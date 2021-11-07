use super::{auth::AccessTokenDispenser, parsing::TwitchChatMessage, request::TwitchChatRequest};
use crate::connect::{Connector, ConnectorError, Request};
use std::{env, net::TcpStream, str::FromStr};
use websocket::{receiver::Reader, sync::Writer, ClientBuilder, Message, OwnedMessage};

pub struct TwitchChatSender {
    sender: Writer<TcpStream>,
    channel: String,
}

impl TwitchChatSender {
    pub fn new(sender: Writer<TcpStream>, channel: String) -> Self {
        Self {
            sender: sender,
            channel: channel,
        }
    }

    pub fn send_message(&mut self, msg: &str) -> Result<(), ConnectorError> {
        self.send_raw_message(format!("PRIVMSG #{} :{}", self.channel, msg))
    }

    fn send_raw_message(&mut self, message: String) -> Result<(), ConnectorError> {
        let message_obj = Message::text(message);
        self.sender.send_message(&message_obj).map_err(|err| {
            ConnectorError::MessageSendFailed(format!("Could not send message: {:?}", err))
        })
    }

    pub fn login(&mut self, access_token: &str) -> Result<(), ConnectorError> {
        let user_name = env::var("TWITCH_CHAT_USER").map_err(|err| {
            ConnectorError::MessageReceiveFailed(format!(
                "Could not get user name from environment variable: {:?}",
                err
            ))
        })?;
        self.send_raw_message(format!("PASS oauth:{}", access_token))?;
        self.send_raw_message(format!("NICK {}", user_name))?;
        self.send_raw_message(format!("JOIN #{}", self.channel))
    }
}

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
