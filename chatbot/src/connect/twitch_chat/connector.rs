use super::{
    auth::AccessTokenDispenser, parsing::TwitchChatMessage, request::TwitchChatRequest,
    sending::TwitchChatSender,
};
use crate::connect::{Connector, ConnectorError, Request};
use std::{net::TcpStream, str::FromStr};
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
    fn recv_message(&mut self) -> Result<Box<dyn Request + '_>, ConnectorError> {
        let receiver = &mut self.receiver;
        loop {
            let owned_message = receiver
                .recv_message()
                .map_err(|err| ConnectorError::MessageReceiveFailed(format!("{:?}", err)))?;
            match owned_message {
                OwnedMessage::Text(text) => {
                    println!("{}", text);
                    let parsed_message = TwitchChatMessage::from_str(&text)?;
                    match parsed_message {
                        TwitchChatMessage::UserMessage(user_message) => {
                            break Ok(Box::new(TwitchChatRequest::new(
                                user_message,
                                &mut self.sender,
                            )));
                        }
                        TwitchChatMessage::PingMessage(..) => continue,
                    }
                }
                _ => continue,
            }
        }
    }
}
