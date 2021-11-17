use std::collections::HashSet;

use crate::connect::{CommandType, ConnectorError, EventContent, TwitchChatConnector};

pub struct ChatBot {
    chatters: HashSet<String>,
    help_message: String,
    info_message: String,
    connector: TwitchChatConnector,
}

impl ChatBot {
    pub fn new() -> Self {
        Self {chatters: HashSet::<String>::new(), help_message: "!help: Show this help | !info: Show some information about the chat bot".to_owned(), info_message: "Hello, my name is TwitchBotanist. I am a twitch chat bot written in Rust. If you want to know what you can ask me, write '!help' into the chat!".to_owned(), connector: TwitchChatConnector::new("captaincallback")}
    }

    fn handle_event(&mut self, event: EventContent) {
        match event {
            EventContent::Command(info) => {
                match info.commmand_type {
                    CommandType::Help => {
                        self.connector.send_message(&self.help_message).unwrap();
                    }
                    CommandType::Info => {
                        self.connector.send_message(&self.info_message).unwrap();
                    }
                    CommandType::Slap => {
                        println!("{:?}", self.chatters);
                        let chatters = &self.chatters;
                        let slapping_user = info.user_name;
                        let slapped_user = info
                            .options
                            .get(0)
                            .and_then(|slapped_user| chatters.get(slapped_user));
                        if let Some(slapped_user) = slapped_user {
                            self.connector
                                .send_message(
                                    format!(
                                        "{} slaps {} around a bit with a large trout",
                                        slapping_user, slapped_user
                                    )
                                    .as_str(),
                                )
                                .ok();
                        }
                    }
                };
            }
            EventContent::Join(user) => {
                println!("{:?}", &user.0);
                self.chatters.insert(user.0);
            }
            EventContent::Part(user) => {
                self.chatters.remove(&user.0);
            }
            EventContent::TextMessage(tm) => println!("{}: {}", &tm.user_name, &tm.text),
        };
    }

    pub async fn run(&mut self) -> Result<(), ConnectorError> {
        self.connector.initialize().await?;
        self.connector.send_message("Hello, world!");
        self.connector.send_message("/followers")?;
        loop {
            let message = self.connector.recv_event();
            if let Ok(event) = message {
                self.handle_event(event);
            };
        }
    }
}
