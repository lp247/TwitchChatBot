use connect::EventContent;

use crate::{
    connect::{CommandType, Connector, TwitchChatConnector},
    handle::{CommandHandler, StaticStringCommandHandler},
};
use std::error::Error;

extern crate websocket;

mod connect;
mod handle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut connector = TwitchChatConnector::new("captaincallback");
    connector.initialize().await?;
    connector.send_message("Starting Chat Bot")?;
    connector.send_message("/followers")?;
    let help_command_message =
        "!help: Show this help | !info: Show some information about the chat bot";
    let help_command_handler = StaticStringCommandHandler::new(help_command_message);
    let info_command_message = "Hello, my name is TwitchBotanist. I am a twitch chat bot written in Rust. If you want to know what you can ask me, write '!help' into the chat!";
    let info_command_handler = StaticStringCommandHandler::new(info_command_message);
    loop {
        let message = connector.recv_event();
        if let Ok(mut msg) = message {
            match msg.content() {
                EventContent::Command(info) => {
                    match info.commmand_type {
                        CommandType::Help => {
                            msg.respond(help_command_handler.run(&info.options))
                                .unwrap();
                        }
                        CommandType::Info => {
                            msg.respond(info_command_handler.run(&info.options))
                                .unwrap();
                        }
                    };
                }
                EventContent::Join(_) => (),
                EventContent::Part(_) => (),
                EventContent::TextMessage(_) => (),
            };
        };
    }
}
