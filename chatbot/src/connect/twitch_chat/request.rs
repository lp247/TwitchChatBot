use super::sending::TwitchChatSender;
use crate::connect::{ConnectorError, EventContent};

pub struct TwitchChatEvent<'a> {
    pub content: EventContent,
    sender: &'a TwitchChatSender,
}

impl<'a> TwitchChatEvent<'a> {
    pub fn new(content: EventContent, sender: &'a TwitchChatSender) -> Self {
        Self { content, sender }
    }

    pub fn respond(&self, response: &str) -> Result<(), ConnectorError> {
        self.sender.send_message(response)
    }
}
