mod twitch_chat;

use std::str::FromStr;
use thiserror::Error;
pub use twitch_chat::TwitchChatConnector;

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Receiving message failed: {0:?}")]
    MessageReceiveFailed(String),
    #[error("Sending message failed: {0:?}")]
    MessageSendFailed(String),
    #[error("Unknown command: {0:?}")]
    UnknownCommand(String),
}

#[derive(Debug)]
pub enum Command {
    Help,
}

impl FromStr for Command {
    type Err = ConnectorError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if !text.starts_with('!') {
            Err(Self::Err::MessageReceiveFailed(
                "Could not parse command".to_owned(),
            ))
        } else {
            let command_end_index = text.find(' ').unwrap_or(text.len());
            let command_text = &text[1..command_end_index];
            match command_text {
                "help" => Ok(Command::Help),
                _ => Err(Self::Err::UnknownCommand(command_text.to_owned())),
            }
        }
    }
}

#[derive(Debug)]
pub enum MessageContent {
    Command(Command),
    Text(String),
}

impl FromStr for MessageContent {
    type Err = ConnectorError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.starts_with('!') {
            Ok(Self::Command(Command::from_str(text)?))
        } else {
            Ok(Self::Text(text.to_owned()))
        }
    }
}

pub trait Request {
    fn content(&self) -> &MessageContent;
    fn user_name(&self) -> &str;
    fn respond(&mut self, response: &str) -> Result<(), ConnectorError>;
}

pub trait Connector {
    fn recv_message(&mut self) -> Result<Box<dyn Request + '_>, ConnectorError>;
}
