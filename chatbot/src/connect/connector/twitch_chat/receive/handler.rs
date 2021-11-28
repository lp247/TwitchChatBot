use super::ReceiveEvent;
use crate::connect::error::ConnectorError;
use std::net::TcpStream;
use websocket::{receiver::Reader, OwnedMessage};

pub fn handle_receiving_events(
    receiver: &mut Reader<TcpStream>,
) -> Result<Vec<ReceiveEvent>, ConnectorError> {
    let owned_message = receiver
        .recv_message()
        .map_err(|err| ConnectorError::MessageReceiveFailed(format!("{:?}", err)))?;
    loop {
        match owned_message {
            OwnedMessage::Text(text) => {
                println!("New websocket message: {}", text);
                let events = text.lines().filter_map(ReceiveEvent::new).collect();
                return Ok(events);
            }
            _ => continue,
        }
    }
}
