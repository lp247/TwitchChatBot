use crate::connect::{Command, Connector, MessageContent, TwitchChatConnector};
use std::error::Error;

extern crate websocket;

mod connect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Start");
    let mut proxy = TwitchChatConnector::new("captaincallback");
    proxy.initialize().await?;
    loop {
        let message = proxy.recv_message();
        if let Ok(mut msg) = message {
            let part = match msg.content() {
                MessageContent::Command(info) => {
                    let command_name = match info {
                        Command::Help => "help",
                    };
                    format!("Got command '{}'", command_name)
                }
                MessageContent::Text(info) => {
                    format!("Got message '{}'", info)
                }
            };
            let response = format!("{} from user {}", part, msg.user_name());
            println!("{}", response);
            msg.respond(response.as_str()).unwrap();
        }
    }
}
