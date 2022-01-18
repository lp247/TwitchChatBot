use super::connector::twitch_chat::send::SendTask;
use std::sync::mpsc;
use thiserror::Error;
use tokio_tungstenite::tungstenite;
use websocket::websocket_base;

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Receiving message failed: {0:?}")]
    MessageReceiveFailed(String),
    #[error("External server error: {0:?}")]
    ExternalServerError(String),
    #[error("Http status 400: bad request: {0:?}")]
    HTTP400(String),
    #[error("Http status 404: not found")]
    HTTP404,
    #[error("Http status 403: forbidden: {0:?}")]
    HTTP403(String),
    #[error("No stored value available: {0}")]
    StoredValueNotAvailable(String),
    // Errors for other crates
    #[error("Error in crate 'mpsc': {0:?}")]
    MPSCSendError(#[from] mpsc::SendError<SendTask>),
    #[error("Error in crate 'reqwest': {0:?}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Error in crate 'serde_json': {0:?}")]
    SerdeJSONError(#[from] serde_json::Error),
    #[error("Error in crate 'kv': {0:?}")]
    KVError(#[from] kv::Error),
    #[error("Error in crate 'websocket': {0:?}")]
    WebsocketError(#[from] websocket_base::result::WebSocketError),
    #[error("Error in crate 'tungstenite': {0:?}")]
    TungsteniteError(#[from] tungstenite::Error),
    #[error("Error in crate 'futures::channel::mpsc', {0:?}")]
    FuturesChannelError(#[from] futures::channel::mpsc::SendError),
}
