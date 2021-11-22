use crate::{app_config::AppConfig, connect::ConnectorError};
use std::{cell::RefCell, net::TcpStream, sync::Arc};
use websocket::{sync::Writer, Message};

pub struct TwitchChatSender {
    sender: RefCell<Writer<TcpStream>>,
    app_config: Arc<AppConfig>,
}

impl TwitchChatSender {
    pub fn new(sender: Writer<TcpStream>, app_config: Arc<AppConfig>) -> Self {
        Self {
            sender: RefCell::new(sender),
            app_config,
        }
    }

    pub fn send_message(&self, msg: &str) -> Result<(), ConnectorError> {
        self.send_raw_message(format!(
            "PRIVMSG #{} :{}",
            self.app_config.channel_name(),
            msg
        ))
    }

    pub fn send_raw_message(&self, message: String) -> Result<(), ConnectorError> {
        let message_obj = Message::text(message);
        self.sender
            .borrow_mut()
            .send_message(&message_obj)
            .map_err(|err| {
                ConnectorError::MessageSendFailed(format!("Could not send message: {:?}", err))
            })
    }

    pub fn login(&self, access_token: &str) -> Result<(), ConnectorError> {
        self.send_raw_message(format!("PASS oauth:{}", access_token))?;
        self.send_raw_message(format!("NICK {}", self.app_config.bot_user_name()))?;
        self.send_raw_message(format!("JOIN #{}", self.app_config.channel_name()))?;
        self.send_raw_message("CAP REQ :twitch.tv/membership".to_owned())?;
        self.send_raw_message("CAP REQ :twitch.tv/tags".to_owned())
    }
}
