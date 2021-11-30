use std::time::Duration;

use crate::connect::ChatBotEvent;

#[derive(Debug)]
pub enum ChatBotCommand {
    SendMessage(String),
    LogTextMessage(String),
    // bot registers to be called back with the specified event
    TimedCallback{duration: Duration, event: ChatBotEvent},
    // bot sends more than one command
    MultipleCommands(Vec<ChatBotCommand>),
}
