use std::str::FromStr;

use crate::connect::ConnectorError;

// Example message: PING :tmi.twitch.tv
#[derive(Debug)]
pub struct PingMessage(pub String);

impl FromStr for PingMessage {
    type Err = ConnectorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(server) = s.strip_prefix("PING :") {
            Ok(Self(server.to_owned()))
        } else {
            Err(ConnectorError::MessageReceiveFailed(
                "Bad ping message syntax".to_owned(),
            ))
        }
    }
}
