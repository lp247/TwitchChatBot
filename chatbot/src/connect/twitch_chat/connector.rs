use super::{auth::AccessTokenDispenser, request::TwitchChatEvent, sending::TwitchChatSender};
use crate::connect::{
    twitch_chat::event::{InternalEventContent, TwitchChatInternalEvent},
    Connector, ConnectorError, Event,
};
use std::net::TcpStream;
use websocket::{receiver::Reader, ClientBuilder, OwnedMessage};

pub struct TwitchChatConnector {
    receiver: Reader<TcpStream>,
    sender: TwitchChatSender,
    access_token_dispenser: AccessTokenDispenser,
}

impl TwitchChatConnector {
    pub fn new(channel: &str) -> Self {
        let chat_client = ClientBuilder::new("ws://irc-ws.chat.twitch.tv:80")
            .unwrap()
            .connect_insecure()
            .unwrap();
        let (receiver, sender) = chat_client.split().unwrap();
        Self {
            receiver,
            sender: TwitchChatSender::new(sender, channel.to_owned()),
            access_token_dispenser: AccessTokenDispenser::new(),
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
}

impl Connector for TwitchChatConnector {
    fn recv_event(&mut self) -> Result<Box<dyn Event + '_>, ConnectorError> {
        let receiver = &mut self.receiver;
        let sender = &mut self.sender;
        loop {
            let owned_message = receiver
                .recv_message()
                .map_err(|err| ConnectorError::MessageReceiveFailed(format!("{:?}", err)))?;
            match owned_message {
                OwnedMessage::Text(text) => {
                    println!("{}", text);
                    if let Some(parsed_message) = TwitchChatInternalEvent::new(&text) {
                        match parsed_message {
                            TwitchChatInternalEvent::External(event_content) => {
                                break Ok(Box::new(TwitchChatEvent::new(
                                    event_content,
                                    &mut self.sender,
                                )));
                            }
                            TwitchChatInternalEvent::Internal(internal_content) => {
                                match internal_content {
                                    InternalEventContent::Ping(server) => {
                                        sender.send_raw_message(format!("PONG :{}", server))?
                                    }
                                }
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
    }
}
