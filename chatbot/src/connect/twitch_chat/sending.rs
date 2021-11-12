use crate::connect::ConnectorError;
use std::{env, net::TcpStream};
use websocket::{sync::Writer, Message};

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

    pub fn send_raw_message(&mut self, message: String) -> Result<(), ConnectorError> {
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
        self.send_raw_message(format!("JOIN #{}", self.channel))?;
        self.send_raw_message("CAP REQ :twitch.tv/membership".to_owned())
    }
}
