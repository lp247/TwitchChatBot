use crate::connect::{ConnectorError, MessageContent};
use std::str::FromStr;

// Example message: :carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :backseating backseating
#[derive(Debug)]
pub struct UserMessage {
    pub content: MessageContent,
    pub user_name: String,
}

impl FromStr for UserMessage {
    type Err = ConnectorError;

    fn from_str(message: &str) -> Result<Self, Self::Err> {
        enum ParsingState {
            UserName,
            AdditionalUserInfo,
            MessageToken,
            Channel,
            MessageText,
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
                // PRIVMSG #captaincallback :backseating backseating
                MessageToken => {
                    if codepoint == ' ' {
                        let token = &message[marker..i];
                        match token {
                            "PRIVMSG" => state = Channel,
                            "JOIN" => {
                                return Self {
                                    content: MessageContent::Command(Command {
                                        commmand_type: Command::JOIN,
                                        options: vec![],
                                    }),
                                }
                            }
                            "PART" => {}
                        }
                        if token == "PRIVMSG" {
                            state = Channel;
                        } else {
                            // we're only interested in PRIVMSG
                            return Err(ConnectorError::MessageReceiveFailed(
                                "Unknown IRC command".to_owned(),
                            ));
                        }
                    }
                }
                Channel => {
                    if codepoint == ' ' {
                        state = MessageText;
                    }
                }
                MessageText => {
                    let text = message[(i + 1)..].trim();
                    let content = MessageContent::from_str(text)?;
                    return Ok(Self {
                        content: content,
                        user_name: user_name.to_owned(),
                    });
                }
            }
        }
        Err(ConnectorError::MessageReceiveFailed(
            "Unknown error".to_owned(),
        ))
    }
}
