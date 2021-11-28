use super::{
    auth::AccessTokenDispenser,
    receive::{handle_receiving_events, ConnectorEvent, ReceiveEvent},
    send::{get_login_tasks, handle_multiple_sending_tasks, handle_sending_task, SendTask},
};
use crate::{
    app_config::AppConfig,
    connect::{error::ConnectorError, event_content::EventContent},
};
use std::net::TcpStream;
use websocket::{receiver::Reader, sync::Writer, ClientBuilder};

pub struct TwitchChatConnector<'a> {
    receiver: Reader<TcpStream>,
    sender: Writer<TcpStream>,
    app_config: &'a AppConfig,
    access_token_dispenser: AccessTokenDispenser<'a>,
}

impl<'a> TwitchChatConnector<'a> {
    pub fn new(app_config: &'a AppConfig) -> Self {
        let chat_client = ClientBuilder::new("ws://irc-ws.chat.twitch.tv:80")
            .unwrap()
            .connect_insecure()
            .unwrap();
        let (receiver, sender) = chat_client.split().unwrap();
        Self {
            receiver,
            sender,
            app_config,
            access_token_dispenser: AccessTokenDispenser::new(app_config),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), ConnectorError> {
        let access_token: String = self
            .access_token_dispenser
            .get()
            .await
            .expect("Could not get access token")
            .to_owned();
        handle_multiple_sending_tasks(
            &mut self.sender,
            get_login_tasks(
                &access_token,
                self.app_config.bot_user_name(),
                self.app_config.channel_name(),
            ),
        )?;
        Ok(())
    }

    pub fn send_message(&mut self, message: &str) -> Result<(), ConnectorError> {
        handle_sending_task(
            &mut self.sender,
            SendTask::PrivateMessage(self.app_config.channel_name(), message),
        )
    }

    pub fn recv_events(&mut self) -> Result<Vec<EventContent>, ConnectorError> {
        let events = handle_receiving_events(&mut self.receiver)?;
        let mut result: Vec<EventContent> = Vec::default();
        for event in events {
            match event {
                ReceiveEvent::ChatBotEvent(event_content) => result.push(event_content),
                ReceiveEvent::ConnectorEvent(ConnectorEvent::Ping) => {
                    handle_sending_task(&mut self.sender, SendTask::Pong)?;
                }
            }
        }
        Ok(result)
    }
}
