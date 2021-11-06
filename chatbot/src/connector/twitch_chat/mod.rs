mod auth;
mod error;
mod parsing;

use std::env;
use websocket::{client::ClientBuilder, Message, OwnedMessage};

pub struct TwitchChatConnector {
    receiver: websocket::receiver::Reader<std::net::TcpStream>,
    sender: websocket::sender::Writer<std::net::TcpStream>,
    channel: String,
    access_token_dispenser: auth::AccessTokenDispenser,
}

impl<'a> TwitchChatConnector {
    pub fn new(channel: &str) -> Self {
        let chat_client = ClientBuilder::new("ws://irc-ws.chat.twitch.tv:80")
            .unwrap()
            .connect_insecure()
            .unwrap();
        let (receiver, sender) = chat_client.split().unwrap();
        Self {
            receiver,
            sender,
            channel: String::from(channel),
            access_token_dispenser: auth::AccessTokenDispenser::new(),
        }
    }

    fn send_raw_message(
        &mut self,
        message: String,
    ) -> std::result::Result<(), websocket::WebSocketError> {
        let message_obj = Message::text(message);
        self.sender.send_message(&message_obj)
    }

    fn login(&mut self, access_token: &str) -> std::result::Result<(), websocket::WebSocketError> {
        self.send_raw_message(format!("PASS oauth:{}", access_token))?;
        self.send_raw_message(format!("NICK {}", env::var("TWITCH_CHAT_USER").unwrap()))?;
        self.send_raw_message(format!("JOIN #{}", self.channel))
    }

    pub async fn initialize(&mut self) -> std::result::Result<(), websocket::WebSocketError> {
        let access_token = self
            .access_token_dispenser
            .get()
            .await
            .expect("Could not get access token")
            .to_owned();
        self.login(access_token.as_str())?;
        Ok(())
    }

    pub fn send_message(
        &mut self,
        msg: &str,
    ) -> std::result::Result<(), websocket::WebSocketError> {
        self.send_raw_message(format!("PRIVMSG #{} :{}", self.channel, msg))
    }

    pub fn incoming_messages(&mut self) -> impl Iterator<Item = parsing::MessageInfo> + '_ {
        let receiver = &mut self.receiver;
        let sender = &mut self.sender;
        receiver
            .incoming_messages()
            .filter_map(move |message| match message {
                Ok(m) => match m {
                    OwnedMessage::Text(t) => {
                        let parsed_message = parsing::parse_full_message(&t)?;
                        match parsed_message {
                            parsing::MessageType::UserMessage(message_info) => Some(message_info),
                            parsing::MessageType::PingMessage(server) => {
                                let message_obj = Message::text(format!("PONG :{}", server));
                                sender.send_message(&message_obj);
                                // self.send_raw_message("PONG :tmi.twitch.tv".to_owned());
                                None
                            }
                        }
                    }
                    _ => return None,
                },
                Err(_) => return None,
            })
    }
}
