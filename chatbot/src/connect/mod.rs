mod twitch_chat;

use thiserror::Error;
pub use twitch_chat::TwitchChatConnector;

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Receiving message failed: {0:?}")]
    MessageReceiveFailed(String),
    #[error("Sending message failed: {0:?}")]
    MessageSendFailed(String),
    #[error("Unknown command: {0:?}")]
    UnknownCommand(String),
}

#[derive(Debug)]
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
    fn new(text: &str, user_name: &str) -> Result<Self, ConnectorError> {
        if !text.starts_with('!') {
            Err(ConnectorError::MessageReceiveFailed(
                "Could not parse command".to_owned(),
            ))
        } else {
            let command_end_index = text.find(' ').unwrap_or(text.len());
            let command_text = &text[1..command_end_index];
            let options: Vec<String> = text[(command_end_index + 1)..]
                .split(' ')
                .map(String::from)
                .collect();
            match command_text {
                "help" => Ok(Self {
                    commmand_type: CommandType::Help,
                    options: options,
                    user_name: user_name.to_owned(),
                }),
                "info" => Ok(Self {
                    commmand_type: CommandType::Info,
                    options: options,
                    user_name: user_name.to_owned(),
                }),
                _ => Err(ConnectorError::UnknownCommand(command_text.to_owned())),
            }
        }
    }
}

#[derive(Debug)]
struct TextMessage {
    text: String,
    user_name: String,
}

// Example text: #channel_name :backseating backseating

impl TextMessage {
    fn new(text: &str, user_name: &str) -> Result<Self, ConnectorError> {
        enum ParsingState {
            Channel,
            MessageText,
        }
        use ParsingState::*;

        let mut state = Channel;

        for (i, codepoint) in text.char_indices() {
            match state {
                Channel => {
                    if codepoint == ' ' {
                        state = MessageText;
                    }
                }
                MessageText => {
                    let text = text[(i + 1)..].trim();
                    return Ok(Self {
                        text: text.to_owned(),
                        user_name: user_name.to_owned(),
                    });
                }
            }
        }
        Err(ConnectorError::MessageReceiveFailed(
            "Could not parse text message".to_owned(),
        ))
    }
}

#[derive(Debug)]
struct Part(String);

impl Part {
    fn new(user_name: &str) -> Self {
        Self(user_name.to_owned())
    }
}

#[derive(Debug)]
struct Join(String);

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
    fn new(message: &str) -> Result<Self, ConnectorError> {
        enum ParsingState {
            UserName,
            AdditionalUserInfo,
            MessageToken,
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
                    ' ' => {
                        return Err(ConnectorError::MessageReceiveFailed(
                            "Unexpected message syntax".to_owned(),
                        ))
                    }
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
                // PRIVMSG #channel_name :backseating backseating
                MessageToken => {
                    if codepoint == ' ' {
                        let token = &message[marker..i];
                        return match token {
                            "PRIVMSG" => Ok(EventContent::TextMessage(TextMessage::new(
                                &message[i..],
                                user_name,
                            )?)),
                            "JOIN" => Ok(EventContent::Join(Join::new(user_name))),
                            "PART" => Ok(EventContent::Part(Part::new(user_name))),
                            _ => Err(ConnectorError::MessageReceiveFailed(
                                "Unknown IRC command".to_owned(),
                            )),
                        };
                    }
                }
            }
        }
        Err(ConnectorError::MessageReceiveFailed(
            "Unknown error".to_owned(),
        ))
    }
}

pub trait Event {
    fn content(&self) -> &EventContent;
    fn respond(&mut self, response: &str) -> Result<(), ConnectorError>;
}

pub trait Connector {
    fn recv_event(&mut self) -> Result<Box<dyn Event + '_>, ConnectorError>;
}
