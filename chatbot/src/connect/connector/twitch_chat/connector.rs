use super::{
    auth::AccessTokenDispenser,
    receive::{receive, ReceiveEvent},
    send::{get_login_tasks, send, send_multiple, SendTask},
};
use crate::{
    app_config::AppConfig,
    connect::{error::ConnectorError, ChatBotEvent},
};
use std::{
    net::TcpStream,
    sync::mpsc::{self, Sender, SyncSender},
    thread::{self, JoinHandle},
};
use websocket::{receiver::Reader, sync::Writer, ClientBuilder};

pub struct TwitchChatConnector<'a> {
    _receive_thread: ReceiveThread,
    send_thread: SendThread,
    app_config: &'a AppConfig,
}

impl<'a> TwitchChatConnector<'a> {
    pub async fn new(
        app_config: &'a AppConfig,
        chatbot_event_sender: Sender<ChatBotEvent>,
    ) -> TwitchChatConnector<'a> {
        let chat_client = ClientBuilder::new("ws://irc-ws.chat.twitch.tv:80")
            .unwrap()
            .connect_insecure()
            .unwrap();
        let (receiver, mut sender) = chat_client.split().unwrap();
        let mut access_token_dispenser = AccessTokenDispenser::new(app_config);
        let access_token: String = access_token_dispenser
            .get()
            .await
            .expect("Could not get access token")
            .to_owned();
        send_multiple(
            &mut sender,
            get_login_tasks(
                &access_token,
                app_config.bot_user_name(),
                app_config.channel_name(),
            ),
        )
        .expect("Could not log in");
        let send_thread = send_thread(sender);
        let receive_thread = receive_thread(receiver, chatbot_event_sender, send_thread.tx.clone());
        Self {
            send_thread,
            _receive_thread: receive_thread,
            app_config,
        }
    }

    pub fn send_message(&self, message: &'a str) -> Result<(), ConnectorError> {
        Ok(self.send_thread.tx.send(SendTask::PrivateMessage(
            self.app_config.channel_name().to_string(),
            message.to_string(),
        ))?)
    }
}

struct ReceiveThread {
    _handle: JoinHandle<()>,
}

fn receive_thread(
    mut receiver: Reader<TcpStream>,
    send_chat_bot_events: Sender<ChatBotEvent>,
    send_tasks: SyncSender<SendTask>,
) -> ReceiveThread {
    use ReceiveEvent::*;
    let handle = thread::spawn(move || 'outer: loop {
        match receive(&mut receiver) {
            Ok(events) => {
                for event in events {
                    if let ChatBotEvent(event_content) = event {
                        if let Err(error) = send_chat_bot_events.send(event_content) {
                            println!("Reader thread stopped with error {:?}", error);
                            break 'outer;
                        }
                    } else {
                        if let Err(error) = send_tasks.send(SendTask::Pong) {
                            println!("Reader thread stopped with error {:?}", error);
                            break 'outer;
                        }
                    }
                }
            }
            Err(error) => {
                println!("Reader thread stopped with error {:?}", error);
                break 'outer;
            }
        }
    });
    ReceiveThread { _handle: handle }
}

struct SendThread {
    _handle: JoinHandle<()>,
    tx: SyncSender<SendTask>,
}

const SEND_CHAN_CAPACITY: usize = 10;

fn send_thread(mut sender: Writer<TcpStream>) -> SendThread {
    let (tx, rx) = mpsc::sync_channel(SEND_CHAN_CAPACITY);
    let handle = thread::spawn(move || {
        while let Ok(task) = rx.recv() {
            if let Err(error) = send(&mut sender, task) {
                println!("writer thread stopped with error {:?}", error);
                break;
            }
        }
    });
    SendThread {
        _handle: handle,
        tx,
    }
}
