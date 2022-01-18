use crate::connect::{ChatBotEvent, Command, CommandType};
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    StreamExt,
};
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct ChatBot {
    chatters: HashSet<String>, // NOTE: probably replace String with a User struct when we need it.
    dynamic_commands: HashMap<String, String>,
    repeating_messages: HashMap<String, RepeatingMessage>,
}

#[derive(Debug)]
struct RepeatingMessage {
    name: String,
    text: String,
    interval: Duration,
    timer_id: Uuid,
}

const HELP_MESSAGE: &str =
    "!help: Show this help | !info: Show some information about the chat bot";
const INFO_MESSAGE: &str =
    "Hello, my name is TwitchBotanist. I am a twitch chat bot written in Rust. My source code is on GitHub (https://github.com/CaptainCallback/TwitchBotanist). If you want to know what you can ask me, write '!help' into the chat!";
const NEW_COMMAND_SUCCESSFUL_MESSAGE: &str = "The new command has been defined successfully.";
const NEW_COMMAND_NO_OPTION_MESSAGE: &str =
    "newcommand requires at least two options but less were given.";
const REMOVE_COMMAND_NO_OPTION_MESSAGE: &str =
    "removecommand requires at least one option but none was given.";
const REMOVE_COMMAND_SUCCESSFUL_MESSAGE: &str = "The command has been removed successfully.";
const DENIED_MESSAGE: &str = "Denied: i ought to !slap you...";
const DISCORD_MESSAGE: &str =
    "You can join me on discord for news and updates here: https://discord.gg/qM6DTTQxDV";

impl ChatBot {
    pub fn new() -> Self {
        Self {
            chatters: HashSet::default(),
            dynamic_commands: HashMap::default(),
            repeating_messages: HashMap::default(),
        }
    }

    fn handle_command(&mut self, command: Command) -> Option<String> {
        log::info!("Executing this command: {:#?}", command);
        match command.kind {
            CommandType::Discord => Some(DISCORD_MESSAGE.to_owned()),
            CommandType::Help => Some(HELP_MESSAGE.to_owned()),
            CommandType::Info => Some(INFO_MESSAGE.to_owned()),
            CommandType::Slap => {
                log::debug!("Slapping one of these guys \n{:#?}", self.chatters);
                // Notice how we can now do everything in a single expression
                // because we removed the IO from this place
                let slapping_user = command.user.name;
                log::debug!("This guy specifically : {}", &slapping_user);
                command
                    .options
                    .get(0)
                    .and_then(|slapped_user| self.chatters.get(slapped_user))
                    .map(|slapped_user| {
                        format!(
                            "{} slaps {} around a bit with a large trout",
                            slapping_user, slapped_user
                        )
                    })
            }
            CommandType::NewCommand => {
                if command.user.has_elevated_rights() {
                    if command.options.len() < 2 {
                        Some(NEW_COMMAND_NO_OPTION_MESSAGE.to_owned())
                    } else {
                        let new_command_name = &command.options[0];
                        let new_command_message = command.options[1..].join(" ");
                        self.dynamic_commands
                            .insert(new_command_name.to_owned(), new_command_message);
                        Some(NEW_COMMAND_SUCCESSFUL_MESSAGE.to_owned())
                    }
                } else {
                    Some(DENIED_MESSAGE.to_owned())
                }
            }
            CommandType::RemoveCommand => {
                if command.user.has_elevated_rights() {
                    if command.options.is_empty() {
                        Some(REMOVE_COMMAND_NO_OPTION_MESSAGE.to_owned())
                    } else {
                        let command_name = &command.options[0];
                        self.dynamic_commands.remove(command_name);
                        Some(REMOVE_COMMAND_SUCCESSFUL_MESSAGE.to_owned())
                    }
                } else {
                    Some(DENIED_MESSAGE.to_owned())
                }
            }
            CommandType::NewRepeating => {
                if command.user.has_elevated_rights() {
                    if command.options.len() < 2 {
                        // TODO: set the correct message here
                        Some(NEW_COMMAND_NO_OPTION_MESSAGE.to_owned())
                    } else {
                        let message_name = &command.options[0];
                        if let Ok(seconds) = &command.options[1].parse() {
                            let interval = Duration::from_secs(*seconds);
                            let id = Uuid::new_v4();
                            self.repeating_messages.insert(
                                message_name.to_string(),
                                RepeatingMessage {
                                    name: message_name.to_string(),
                                    text: command.options[2..].join(" "),
                                    interval,
                                    timer_id: id,
                                },
                            );
                            // TODO: set the correct message here
                            Some(NEW_COMMAND_SUCCESSFUL_MESSAGE.to_owned())
                        } else {
                            // TODO: set the correct message here
                            Some(NEW_COMMAND_NO_OPTION_MESSAGE.to_owned())
                        }
                    }
                } else {
                    Some(DENIED_MESSAGE.to_owned())
                }
            }
            CommandType::RemoveRepeating => {
                if command.user.has_elevated_rights() {
                    if command.options.is_empty() {
                        // TODO: set the correct message here
                        Some(REMOVE_COMMAND_NO_OPTION_MESSAGE.to_owned())
                    } else {
                        let command_name = &command.options[0];
                        self.repeating_messages.remove(command_name);
                        // TODO: set the correct message here
                        Some(REMOVE_COMMAND_SUCCESSFUL_MESSAGE.to_owned())
                    }
                } else {
                    Some(DENIED_MESSAGE.to_owned())
                }
            }
            CommandType::Dynamic(command_name) => {
                self.dynamic_commands.get(&command_name).map(String::from)
            }
        }
    }

    fn handle_event(&mut self, event: ChatBotEvent) -> Option<String> {
        match event {
            ChatBotEvent::Command(command) => self.handle_command(command),
            ChatBotEvent::Join(user) => {
                log::info!("{:?} joined", &user);
                self.chatters.insert(user);
                None
            }
            ChatBotEvent::Part(user) => {
                log::info!("{:?} parted", &user);
                self.chatters.remove(&user);
                None
            }
            _ => None,
        }
    }

    pub async fn run(
        &mut self,
        mut receiver: UnboundedReceiver<ChatBotEvent>,
        sender: UnboundedSender<String>,
    ) {
        sender
            .unbounded_send("Hello, world!".to_owned())
            .expect("Could not send message");
        while let Some(event) = receiver.next().await {
            if let Some(response) = self.handle_event(event) {
                sender
                    .unbounded_send(response)
                    .expect("Could not send message");
            }
        }
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use crate::connect::{Badge, TextMessage, UserInfo};

    #[test]
    fn test_join() {
        let mut bot = ChatBot::new();
        let response = bot.handle_event(ChatBotEvent::Join(String::from("Carkhy")));
        assert!(response.is_none());
        assert_eq!(bot.chatters.len(), 1);
        assert!(bot.chatters.contains("Carkhy"));
    }

    #[test]
    fn test_part() {
        let mut bot = ChatBot::new();
        bot.handle_event(ChatBotEvent::Join(String::from("Carkhy")));
        let response = bot.handle_event(ChatBotEvent::Part(String::from("Carkhy")));
        assert!(response.is_none());
        assert_eq!(bot.chatters.len(), 0);
        assert!(!bot.chatters.contains("Carkhy"));
    }

    #[test]
    fn test_text_message() {
        let mut bot = ChatBot::new();
        let response = bot.handle_event(ChatBotEvent::TextMessage(TextMessage {
            text: "Hello".to_string(),
            user: UserInfo {
                name: "Carkhy".to_owned(),
                badges: HashSet::default(),
            },
        }));
        assert!(response.is_none());
    }

    #[test]
    fn invalid_slapping() {
        let mut bot = ChatBot::new();
        let response = bot.handle_event(ChatBotEvent::Command(Command {
            user: UserInfo {
                name: "CaptainCallback".to_owned(),
                badges: HashSet::default(),
            },
            kind: CommandType::Slap,
            options: vec!["Carkhy".to_string()],
        }));
        assert!(response.is_none());
    }

    #[test]
    fn valid_slapping_when_abstraction_detected() {
        let mut bot = ChatBot::new();
        bot.handle_event(ChatBotEvent::Join(String::from("CaptainCallback")));
        let response = bot.handle_event(ChatBotEvent::Command(Command {
            user: UserInfo {
                name: "Carkhy".to_owned(),
                badges: HashSet::default(),
            },
            kind: CommandType::Slap,
            options: vec!["CaptainCallback".to_string()],
        }));
        if let Some(text) = response {
            assert_eq!(
                text,
                format!("Carkhy slaps CaptainCallback around a bit with a large trout")
            );
        } else {
            unreachable!();
        }
    }

    #[test]
    fn nonmods_cannot_newcommand() {
        let mut bot = ChatBot::new();
        let response = bot.handle_event(ChatBotEvent::Command(Command {
            user: UserInfo {
                name: "CaptainCallback".to_owned(),
                badges: HashSet::default(),
            },
            kind: CommandType::NewCommand,
            options: vec!["test".to_string(), "testing".to_string()],
        }));
        if let Some(s) = response {
            assert_eq!(s, DENIED_MESSAGE);
        } else {
            unreachable!();
        }
        assert!(!bot.dynamic_commands.contains_key("test"));
    }

    #[test]
    fn broadcaster_can_newcommand() {
        let mut bot = ChatBot::new();
        let response = bot.handle_event(ChatBotEvent::Command(Command {
            user: UserInfo {
                name: "CaptainCallback".to_owned(),
                badges: HashSet::from([Badge {
                    name: "broadcaster".to_owned(),
                    level: 1,
                }]),
            },
            kind: CommandType::NewCommand,
            options: vec!["test".to_string(), "testing".to_string()],
        }));
        if let Some(s) = response {
            assert_ne!(s, DENIED_MESSAGE);
        } else {
            unreachable!();
        }
        assert!(bot.dynamic_commands.contains_key("test"));
    }

    #[test]
    fn mods_can_newcommand() {
        let mut bot = ChatBot::new();
        let response = bot.handle_event(ChatBotEvent::Command(Command {
            user: UserInfo {
                name: "CaptainCallback".to_owned(),
                badges: HashSet::from([Badge {
                    name: "moderator".to_owned(),
                    level: 1,
                }]),
            },
            kind: CommandType::NewCommand,
            options: vec!["test2".to_string(), "testing2".to_string()],
        }));
        if let Some(s) = response {
            assert_ne!(s, DENIED_MESSAGE);
        } else {
            unreachable!();
        }
        assert!(bot.dynamic_commands.contains_key("test2"));
    }
}
