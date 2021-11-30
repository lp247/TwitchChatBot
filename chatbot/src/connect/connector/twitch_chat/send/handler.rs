use super::task::SendTask;
use crate::connect::error::ConnectorError;
use std::net::TcpStream;
use websocket::{sync::Writer, Message};

pub fn handle_sending_task(
    sender: &mut Writer<TcpStream>,
    task: SendTask,
) -> Result<(), ConnectorError> {
    let message = Message::text(task.to_string());
    sender.send_message(&message).map_err(|err| {
        ConnectorError::MessageSendFailed(format!("Could not send message: {:?}", err))
    })
}

pub fn handle_multiple_sending_tasks(
    sender: &mut Writer<TcpStream>,
    tasks: Vec<SendTask>,
) -> Result<(), ConnectorError> {
    for task in tasks {
        handle_sending_task(sender, task)?;
    }
    Ok(())
}
