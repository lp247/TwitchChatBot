use std::sync::mpsc::SendError;

use thiserror::Error;

use super::connector::twitch_chat::send::SendTask;

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Receiving message failed: {0:?}")]
    MessageReceiveFailed(String),
    #[error("Sending message failed: {0:?}")]
    MessageSendFailed(String),
    #[error("Send error {0:?}")]
    SendFailed(#[from]SendError<SendTask>),
}
