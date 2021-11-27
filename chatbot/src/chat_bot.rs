use std::collections::{HashMap, HashSet};

use crate::connect::{Command, EventContent};

#[derive(Debug)]
pub struct ChatBot {
    chatters: HashSet<String>, // NOTE: probably replace String with a User struct when we need it.
    dynamic_commands: HashMap<String, String>,
}

// This will later probably have commands to do http requests to retreive
// user information and who knows what else
#[derive(Debug)]
pub enum ChatBotCommand {
    SendMessage(String),
    LogTextMessage(String),
}

const HELP_MESSAGE: &str =
    "!help: Show this help | !info: Show some information about the chat bot";
const INFO_MESSAGE: &str =
    "Hello, my name is TwitchBotanist. I am a twitch chat bot written in Rust. If you want to know what you can ask me, write '!help' into the chat!";
const NEW_COMMAND_SUCCESSFUL_MESSAGE: &str = "The new command has been defined successfully.";
const NEW_COMMAND_NO_OPTION_MESSAGE: &str =
    "newcommand requires at least two options but less were given.";
const REMOVE_COMMAND_NO_OPTION_MESSAGE: &str =
    "removecommand requires at least one option but none was given.";
const REMOVE_COMMAND_SUCCESSFUL_MESSAGE: &str = "The command has been removed successfully.";

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
        match command.name.as_str() {
            "help" => Some(SendMessage(HELP_MESSAGE.to_string())),
            "info" => Some(SendMessage(INFO_MESSAGE.to_string())),
            "slap" => {
                println!("Slapping one of these guys \n{:#?}", self.chatters);
                // Notice how we can now do everything in a single expression
                // because we removed the IO from this place
                let slapping_user = command.user_name;
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
            "newcommand" => {
                if command.options.len() < 2 {
                    Some(SendMessage(NEW_COMMAND_NO_OPTION_MESSAGE.to_string()))
                } else {
                    let new_command_name = &command.options[0];
                    let new_command_message = command.options[1..].join(" ");
                    self.dynamic_commands
                        .insert(new_command_name.to_owned(), new_command_message);
                    Some(SendMessage(NEW_COMMAND_SUCCESSFUL_MESSAGE.to_string()))
                }
            }
            "removecommand" => {
                if command.options.is_empty() {
                    Some(SendMessage(REMOVE_COMMAND_NO_OPTION_MESSAGE.to_string()))
                } else {
                    let command_name = &command.options[0];
                    self.dynamic_commands.remove(command_name);
                    Some(SendMessage(REMOVE_COMMAND_SUCCESSFUL_MESSAGE.to_string()))
                }
            }
            _ => command
                .tags
                .get("mod")
                .map(|val| val == "1")
                .and(self.dynamic_commands.get(command.name.as_str()))
                .map(String::from)
                .map(SendMessage),
        }
    }

    pub fn handle_event(&mut self, event: EventContent) -> Option<ChatBotCommand> {
        use ChatBotCommand::*;
        match event {
            EventContent::Command(command) => self.handle_command(command),
            EventContent::Join(user) => {
                println!("{:?} joined", &user);
                self.chatters.insert(user);
                None
            }
            EventContent::Part(user) => {
                println!("{:?} parted", &user);
                self.chatters.remove(&user);
                None
            }
            EventContent::TextMessage(tm) => {
                // this is IO and should perhaps be returned as a command
                // we can choose to do automated moderation here
                Some(LogTextMessage(format!("{}: {}", &tm.user_name, &tm.text)))
            }
        }
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use crate::connect::TextMessage;
    use std::collections::HashMap;

    // It's now easy to test without connecting
    #[test]
    fn test_join() {
        let mut bot = ChatBot::new();
        let result = bot.handle_event(EventContent::Join(String::from("Carkhy")));
        assert!(matches!(result, None));
        assert_eq!(bot.chatters.len(), 1);
        assert_eq!(bot.chatters.get("Carkhy").unwrap(), "Carkhy");
    }

    #[test]
    fn test_part() {
        let mut bot = ChatBot::new();
        bot.handle_event(EventContent::Join(String::from("Carkhy")));
        let result = bot.handle_event(EventContent::Part(String::from("Carkhy")));
        assert!(matches!(result, None));
        assert_eq!(bot.chatters.len(), 0);
        assert!(matches!(bot.chatters.get("Carkhy"), None));
    }

    #[test]
    fn test_text_message() {
        let mut bot = ChatBot::new();
        let result = bot.handle_event(EventContent::TextMessage(TextMessage {
            text: "Hello".to_string(),
            user_name: "Carkhy".to_string(),
            tags: HashMap::<String, String>::new(),
        }));
        assert!(
            matches!(result, Some(ChatBotCommand::LogTextMessage(message)) if message == "Carkhy: Hello")
        );
    }

    #[test]
    fn invalid_slapping() {
        let mut bot = ChatBot::new();
        let result = bot.handle_event(EventContent::Command(Command {
            user_name: "CaptainCallback".to_string(),
            name: "slap".to_owned(),
            options: vec!["Carkhy".to_string()],
            tags: HashMap::<String, String>::new(),
        }));
        assert!(matches!(result, None));
    }

    #[test]
    fn valid_slapping_when_abstraction_detected() {
        let mut bot = ChatBot::new();
        bot.handle_event(EventContent::Join(String::from("CaptainCallback")));
        let result = bot.handle_event(EventContent::Command(Command {
            user_name: "Carkhy".to_string(),
            name: "slap".to_owned(),
            options: vec!["CaptainCallback".to_string()],
            tags: HashMap::<String, String>::new(),
        }));
        println!("{:#?}", result);
        println!(
            "{}",
            format!(
                "{} slaps {} around a bit with a large trout",
                "Carkhy", "CaptainCallback"
            )
        );
        assert!(matches!(result, Some(ChatBotCommand::SendMessage(message))
                         if message == format!("{} slaps {} around a bit with a large trout", "Carkhy", "CaptainCallback")));
    }
}
