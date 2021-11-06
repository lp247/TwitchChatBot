use thiserror::Error;

#[derive(Error, Debug)]
pub enum TwitchError {
    #[error("Request failed: {0:?}")]
    ReqwestError(reqwest::Error),
    #[error("Bad request: {0:?}")]
    BadRequest(String),
    #[error("Parsing error: {0:?}")]
    ReqwestParsingError(reqwest::Error),
    #[error("Parsing error: {0:?}")]
    SerdeJSONParsingError(serde_json::Error),
    #[error("Missing response JSON field: {0:?}")]
    MissingResponseJSONField(String, String),
    #[error("Server error: {0:?}")]
    ServerError(String),
    #[error("Could not receive request: {0:?}")]
    TinyHTTPReceiveError(std::io::Error),
}

pub type Result<T> = std::result::Result<T, TwitchError>;
