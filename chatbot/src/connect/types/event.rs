use uuid::Uuid;

use super::{text_message::TextMessage, Command};

#[derive(Debug, PartialEq)]
pub enum ChatBotEvent {
    TextMessage(TextMessage),
    Command(Command),
    Part(String),
    Join(String),
    // timer sends a message to the bot, String is the name of the message.
    // uuid is the message id, used to deduplicate
    // messages when a command is redefined
    TimedMessage(String, Uuid),
}
