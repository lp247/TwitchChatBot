use super::ChatBotCommand;
use crate::connect::{ChatBotEvent, Command, CommandType};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ChatBot {
    chatters: HashSet<String>, // NOTE: probably replace String with a User struct when we need it.
    dynamic_commands: HashMap<String, String>,
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

fn str_msg(string: &str) -> Option<ChatBotCommand> {
    Some(ChatBotCommand::SendMessage(string.to_string()))
}

impl ChatBot {
    pub fn new() -> Self {
        Self {
            chatters: HashSet::default(),
            dynamic_commands: HashMap::default(),
        }
    }

    fn handle_command(&mut self, command: Command) -> Option<ChatBotCommand> {
        println!("Executing this command: {:#?}", command);
        use ChatBotCommand::*;
        match command.kind {
            CommandType::Discord => str_msg(DISCORD_MESSAGE),
            CommandType::Help => str_msg(HELP_MESSAGE),
            CommandType::Info => str_msg(INFO_MESSAGE),
            CommandType::Slap => {
                println!("Slapping one of these guys \n{:#?}", self.chatters);
                // Notice how we can now do everything in a single expression
                // because we removed the IO from this place
                let slapping_user = command.user.name;
                println!("This guy specifically : {}", &slapping_user);
                command
                    .options
                    .get(0)
                    .and_then(|slapped_user| self.chatters.get(slapped_user))
                    .map(|slapped_user| {
                        SendMessage(format!(
                            "{} slaps {} around a bit with a large trout",
                            slapping_user, slapped_user
                        ))
                    })
            }
            CommandType::NewCommand => {
                if command.user.has_elevated_rights() {
                    if command.options.len() < 2 {
                        str_msg(NEW_COMMAND_NO_OPTION_MESSAGE)
                    } else {
                        let new_command_name = &command.options[0];
                        let new_command_message = command.options[1..].join(" ");
                        self.dynamic_commands
                            .insert(new_command_name.to_owned(), new_command_message);
                        str_msg(NEW_COMMAND_SUCCESSFUL_MESSAGE)
                    }
                } else {
                    str_msg(DENIED_MESSAGE)
                }
            }
            CommandType::RemoveCommand => {
                if command.user.has_elevated_rights() {
                    if command.options.is_empty() {
                        str_msg(REMOVE_COMMAND_NO_OPTION_MESSAGE)
                    } else {
                        let command_name = &command.options[0];
                        self.dynamic_commands.remove(command_name);
                        str_msg(REMOVE_COMMAND_SUCCESSFUL_MESSAGE)
                    }
                } else {
                    str_msg(DENIED_MESSAGE)
                }
            }
            CommandType::Dynamic(command_name) => self
                .dynamic_commands
                .get(&command_name)
                .map(String::from)
                .map(SendMessage),
        }
    }

    pub fn handle_event(&mut self, event: ChatBotEvent) -> Option<ChatBotCommand> {
        use ChatBotCommand::*;
        match event {
            ChatBotEvent::Command(command) => self.handle_command(command),
            ChatBotEvent::Join(user) => {
                println!("{:?} joined", &user);
                self.chatters.insert(user);
                None
            }
            ChatBotEvent::Part(user) => {
                println!("{:?} parted", &user);
                self.chatters.remove(&user);
                None
            }
            ChatBotEvent::TextMessage(tm) => {
                // this is IO and should perhaps be returned as a command
                // we can choose to do automated moderation here
                Some(LogTextMessage(format!("{}: {}", &tm.user.name, &tm.text)))
            }
        }
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use crate::connect::{Badge, TextMessage, UserInfo};

    // It's now easy to test without connecting
    #[test]
    fn test_join() {
        let mut bot = ChatBot::new();
        let result = bot.handle_event(ChatBotEvent::Join(String::from("Carkhy")));
        assert!(matches!(result, None));
        assert_eq!(bot.chatters.len(), 1);
        assert_eq!(bot.chatters.get("Carkhy").unwrap(), "Carkhy");
    }

    #[test]
    fn test_part() {
        let mut bot = ChatBot::new();
        bot.handle_event(ChatBotEvent::Join(String::from("Carkhy")));
        let result = bot.handle_event(ChatBotEvent::Part(String::from("Carkhy")));
        assert!(matches!(result, None));
        assert_eq!(bot.chatters.len(), 0);
        assert!(matches!(bot.chatters.get("Carkhy"), None));
    }

    #[test]
    fn test_text_message() {
        let mut bot = ChatBot::new();
        let result = bot.handle_event(ChatBotEvent::TextMessage(TextMessage {
            text: "Hello".to_string(),
            user: UserInfo {
                name: "Carkhy".to_owned(),
                badges: HashSet::default(),
            },
        }));
        assert!(
            matches!(result, Some(ChatBotCommand::LogTextMessage(message)) if message == "Carkhy: Hello")
        );
    }

    #[test]
    fn invalid_slapping() {
        let mut bot = ChatBot::new();
        let result = bot.handle_event(ChatBotEvent::Command(Command {
            user: UserInfo {
                name: "CaptainCallback".to_owned(),
                badges: HashSet::default(),
            },
            kind: CommandType::Slap,
            options: vec!["Carkhy".to_string()],
        }));
        assert!(matches!(result, None));
    }

    #[test]
    fn valid_slapping_when_abstraction_detected() {
        let mut bot = ChatBot::new();
        bot.handle_event(ChatBotEvent::Join(String::from("CaptainCallback")));
        let result = bot.handle_event(ChatBotEvent::Command(Command {
            user: UserInfo {
                name: "Carkhy".to_owned(),
                badges: HashSet::default(),
            },
            kind: CommandType::Slap,
            options: vec!["CaptainCallback".to_string()],
        }));
        assert!(matches!(result, Some(ChatBotCommand::SendMessage(message))
                         if message == format!("{} slaps {} around a bit with a large trout", "Carkhy", "CaptainCallback")));
    }

    #[test]
    fn nonmods_cannot_newcommand() {
        let mut bot = ChatBot::new();
        let result = bot.handle_event(ChatBotEvent::Command(Command {
            user: UserInfo {
                name: "CaptainCallback".to_owned(),
                badges: HashSet::default(),
            },
            kind: CommandType::NewCommand,
            options: vec!["test".to_string(), "testing".to_string()],
        }));
        assert!(matches!(result, Some(ChatBotCommand::SendMessage(message))
                         if message == DENIED_MESSAGE));
        assert!(!bot.dynamic_commands.contains_key("test"));
    }

    #[test]
    fn broadcaster_can_newcommand() {
        let mut bot = ChatBot::new();
        let result = bot.handle_event(ChatBotEvent::Command(Command {
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
        assert!(matches!(result, Some(ChatBotCommand::SendMessage(message))
                         if message != DENIED_MESSAGE));
        assert!(bot.dynamic_commands.contains_key("test"));
    }

    #[test]
    fn mods_can_newcommand() {
        let mut bot = ChatBot::new();
        let result = bot.handle_event(ChatBotEvent::Command(Command {
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
        assert!(matches!(result, Some(ChatBotCommand::SendMessage(message))
                         if message != DENIED_MESSAGE));
        assert!(bot.dynamic_commands.contains_key("test2"));
    }
}
