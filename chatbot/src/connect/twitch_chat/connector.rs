use super::{auth::AccessTokenDispenser, sending::TwitchChatSender};
use crate::{app_config::AppConfig, connect::{
    twitch_chat::event::{InternalEventContent, TwitchChatInternalEvent},
    ConnectorError, EventContent,
}};
use std::{net::TcpStream, sync::Arc};
use websocket::{receiver::Reader, ClientBuilder, OwnedMessage};

pub struct TwitchChatConnector {
    receiver: Reader<TcpStream>,
    sender: TwitchChatSender,
    access_token_dispenser: AccessTokenDispenser,
}

impl TwitchChatConnector {
    pub fn new(app_config: Arc<AppConfig>) -> Self {
        let chat_client = ClientBuilder::new("ws://irc-ws.chat.twitch.tv:80")
            .unwrap()
            .connect_insecure()
            .unwrap();
        let (receiver, sender) = chat_client.split().unwrap();
        Self {
            receiver,
            sender: TwitchChatSender::new(sender, app_config.clone()),
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
        self.sender.login(access_token.as_str())?;
        Ok(())
    }

    pub fn send_message(&mut self, message: &str) -> Result<(), ConnectorError> {
        self.sender.send_message(message)
    }

    pub fn recv_events(&mut self) -> Result<Vec<EventContent>, ConnectorError> {
        let receiver = &mut self.receiver;
        let sender = &mut self.sender;
        loop {
            let owned_message = receiver
                .recv_message()
                .map_err(|err| ConnectorError::MessageReceiveFailed(format!("{:?}", err)))?;
            match owned_message {
                OwnedMessage::Text(text) => {
                    println!("{}", text);
                    let events = text.lines().filter_map(TwitchChatInternalEvent::new);
                    let mut result : Vec<EventContent> = Vec::default();                    
                    for event in events {
                        match event {
                            TwitchChatInternalEvent::External(event_content) => 
                                result.push(event_content),
                            TwitchChatInternalEvent::Internal(InternalEventContent::Ping(server)) => 
                                sender.send_raw_message(format!("PONG :{}", server))?,
                        }
                    }
                    return Ok(result);
                }
                _ => continue,
            }
        }
    }
}
