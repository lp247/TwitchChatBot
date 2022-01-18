use super::{text_message::TextMessage, Command};

#[derive(Debug, PartialEq)]
pub enum ChatBotEvent {
    TextMessage(TextMessage),
    Command(Command),
    Part(String),
    Join(String),
}
