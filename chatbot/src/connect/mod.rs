mod twitch_chat;

use thiserror::Error;
pub use twitch_chat::TwitchChatConnector;

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Receiving message failed: {0:?}")]
    MessageReceiveFailed(String),
    #[error("Sending message failed: {0:?}")]
    MessageSendFailed(String),
}

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Help,
    Info,
}

#[derive(Debug)]
pub struct Command {
    pub commmand_type: CommandType,
    pub options: Vec<String>,
    pub user_name: String,
}

impl Command {
    fn new(text: &str, user_name: &str) -> Option<Self> {
        if !text.starts_with('!') {
            None
        } else {
            let mut words = text.split(' ');
            match &words.next()?[1..] {
                "help" => Some(Self {
                    commmand_type: CommandType::Help,
                    options: words.map(String::from).collect(),
                    user_name: user_name.to_owned(),
                }),
                "info" => Some(Self {
                    commmand_type: CommandType::Info,
                    options: words.map(String::from).collect(),
                    user_name: user_name.to_owned(),
                }),
                _ => None,
            }
        }
    }
}

#[derive(Debug)]
pub struct TextMessage {
    pub text: String,
    pub user_name: String,
}

// Example text: #channel_name :backseating backseating
impl TextMessage {
    fn new(text: &str, user_name: &str) -> Self {
        Self {
            text: text.to_owned(),
            user_name: user_name.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct Part(String);

impl Part {
    fn new(user_name: &str) -> Self {
        Self(user_name.to_owned())
    }
}

#[derive(Debug)]
pub struct Join(String);

impl Join {
    fn new(user_name: &str) -> Self {
        Self(user_name.to_owned())
    }
}

#[derive(Debug)]
pub enum EventContent {
    TextMessage(TextMessage),
    Command(Command),
    Part(Part),
    Join(Join),
}

// Example message: :carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :backseating backseating
impl EventContent {
    fn new(message: &str) -> Option<Self> {
        enum ParsingState {
            UserName,
            AdditionalUserInfo,
            MessageToken,
            Channel,
            MessageBody,
        }
        use ParsingState::*;

        let mut state = UserName;
        let mut user_name = &message[0..0];
        let mut marker = 0;

        for (i, codepoint) in message.char_indices() {
            match state {
                // :carkhy!carkhy@carkhy.tmi.twitch.tv
                UserName => match codepoint {
                    ':' => marker = i + 1,
                    ' ' => return None,
                    '!' => {
                        user_name = &message[marker..i];
                        state = AdditionalUserInfo;
                    }
                    _ => (),
                },
                AdditionalUserInfo => {
                    if codepoint == ' ' {
                        marker = i + 1;
                        state = MessageToken
                    }
                }
                MessageToken => {
                    if codepoint == ' ' {
                        let token = &message[marker..i];
                        match token {
                            // (...) PRIVMSG #<channel> :backseating backseating
                            "PRIVMSG" => {
                                state = Channel;
                            }
                            // (...) JOIN #<channel>
                            "JOIN" => return Some(EventContent::Join(Join::new(user_name))),
                            // (...) PART #<channel>
                            "PART" => return Some(EventContent::Part(Part::new(user_name))),
                            // PING :tmi.twitch.tv
                            _ => return None,
                        };
                    }
                }
                Channel => {
                    if codepoint == ':' {
                        state = MessageBody;
                    }
                }
                MessageBody => {
                    // :carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :!help
                    if codepoint == '!' {
                        return Some(EventContent::Command(Command::new(
                            message[i..].trim(),
                            user_name,
                        )?));
                    } else {
                        return Some(EventContent::TextMessage(TextMessage::new(
                            message[i..].trim(),
                            user_name,
                        )));
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::connect::{Command, CommandType, EventContent};

    fn user_message_helper(raw_message: &str, user_name: &str, expected: &str) {
        let parsed = EventContent::new(raw_message);
        assert!(parsed.is_some());
        if let EventContent::TextMessage(user_message) = parsed.unwrap() {
            assert_eq!(user_message.user_name, user_name);
            assert_eq!(user_message.text, expected);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn parsing_user_messages() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message";
        let expected_text = "a function that takes a string and returns the message";
        let expected_user = "carkhy";
        user_message_helper(raw_message, expected_user, expected_text);
    }

    #[test]
    fn parsing_user_messages_with_trailing_newlines() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message\n";
        let expected_text = "a function that takes a string and returns the message";
        let expected_user = "carkhy";
        user_message_helper(raw_message, expected_user, expected_text);
    }

    fn command_helper(
        raw_message: &str,
        expected_command_type: CommandType,
        expected_user_name: &str,
    ) {
        let parsed = EventContent::new(raw_message);
        assert!(parsed.is_some());
        if let EventContent::Command(command) = parsed.unwrap() {
            assert_eq!(command.commmand_type, expected_command_type);
            assert_eq!(command.user_name, expected_user_name);
            assert_eq!(command.options, Vec::<String>::new());
        } else {
            unreachable!();
        }
    }

    #[test]
    fn parsing_help_command_in_event_parser() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :!help";
        let expected_command_type = CommandType::Help;
        let expected_user_name = "carkhy";
        command_helper(raw_message, expected_command_type, expected_user_name)
    }

    #[test]
    fn parsing_info_command_in_event_parser() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :!info";
        let expected_command_type = CommandType::Info;
        let expected_user_name = "carkhy";
        command_helper(raw_message, expected_command_type, expected_user_name)
    }

    #[test]
    fn parsing_help_command_in_command_parser() {
        let raw_command = "!help";
        let expected_command_type = CommandType::Help;
        let parsed = Command::new(raw_command, "testuser");
        assert!(parsed.is_some());
        let unwrapped_parsed = parsed.unwrap();
        assert_eq!(unwrapped_parsed.commmand_type, expected_command_type);
        assert_eq!(unwrapped_parsed.user_name, "testuser");
        assert_eq!(unwrapped_parsed.options, Vec::<String>::new());
    }
}
