use super::sending::TwitchChatSender;
use crate::connect::{ConnectorError, Event, EventContent};

pub struct TwitchChatEvent<'a> {
    content: EventContent,
    sender: &'a TwitchChatSender,
}

impl<'a> TwitchChatEvent<'a> {
    pub fn new(content: EventContent, sender: &'a TwitchChatSender) -> Self {
        Self {
            content,
            sender,
        }
    }
}

impl<'a> Event for TwitchChatEvent<'a> {
    fn content(&self) -> &EventContent {
        &self.content
    }

    fn respond(&self, response: &str) -> Result<(), ConnectorError> {
        self.sender.send_message(response)
    }
}
