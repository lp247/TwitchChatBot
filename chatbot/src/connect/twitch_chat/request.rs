use super::{connector::TwitchChatSender, parsing::UserMessage};
use crate::connect::{ConnectorError, MessageContent, Request};

pub struct TwitchChatRequest<'a> {
    message: UserMessage,
    sender: &'a mut TwitchChatSender,
}

impl<'a> TwitchChatRequest<'a> {
    pub fn new(message: UserMessage, sender: &'a mut TwitchChatSender) -> Self {
        Self {
            message: message,
            sender: sender,
        }
    }
}

impl<'a> Request for TwitchChatRequest<'a> {
    fn content(&self) -> &MessageContent {
        &self.message.content
    }

    fn user_name(&self) -> &str {
        self.message.user_name.as_str()
    }

    fn respond(&mut self, response: &str) -> Result<(), ConnectorError> {
        self.sender.send_message(response)
    }
}
