use crate::connect::error::ConnectorError;
use crate::connect::{types::CommandType, Badge, ChatBotEvent, Command, TextMessage, UserInfo};
use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use websocket::WebSocketError;
use websocket::{receiver::Reader, OwnedMessage};

pub fn receive(receiver: &mut Reader<TcpStream>) -> Result<Vec<ReceiveEvent>, ConnectorError> {
    loop {
        match receiver.recv_message() {
            Err(WebSocketError::NoDataAvailable) => continue,
            response => match response {
                Ok(owned_message) => match owned_message {
                    OwnedMessage::Text(text) => {
                        println!("New websocket message: {}", text);
                        let events = text
                            .lines()
                            .filter_map(ReceiveEvent::parse_from_message)
                            .collect();
                        return Ok(events);
                    }
                    _ => continue,
                },
                Err(err) => {
                    return Err(ConnectorError::MessageReceiveFailed(format!(
                        "Could not receive message: {:?}",
                        err
                    )))
                }
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ConnectorEvent {
    Ping,
}

#[derive(Debug, PartialEq)]
pub enum ReceiveEvent {
    ChatBotEvent(ChatBotEvent),
    ConnectorEvent(ConnectorEvent),
}

impl ReceiveEvent {
    fn parse_command_kind(command_name: &str) -> CommandType {
        match command_name {
            "help" => CommandType::Help,
            "info" => CommandType::Info,
            "newcommand" => CommandType::NewCommand,
            "removecommand" => CommandType::RemoveCommand,
            "slap" => CommandType::Slap,
            "discord" => CommandType::Discord,
            "newrepeating" => CommandType::NewRepeating,
            "removerepeating" => CommandType::RemoveRepeating,
            _ => CommandType::Dynamic(command_name.to_owned()),
        }
    }

    fn parse_command_from_message(message: &str) -> Option<(CommandType, Vec<String>)> {
        if message == "!" {
            return None;
        }
        let mut words = message.split(' ');
        words.next().map(|name| {
            (
                ReceiveEvent::parse_command_kind(&name[1..]),
                words.map(String::from).collect(),
            )
        })
    }

    pub fn parse_from_message(message: &str) -> Option<Self> {
        enum ParsingState {
            Start,
            Tags,
            UserName,
            AdditionalUserInfo,
            MessageToken,
            Channel,
            MessageBody,
        }
        use ParsingState::*;

        let mut state = Start;
        let mut user_name = &message[0..0];
        let mut marker = 0;
        let mut tags_map = HashMap::<String, String>::new();

        if message.starts_with("PING:") {
            return Some(ReceiveEvent::ConnectorEvent(ConnectorEvent::Ping));
        }

        for (i, codepoint) in message.char_indices() {
            match state {
                Start => match codepoint {
                    '@' => {
                        state = Tags;
                    }
                    ':' => {
                        state = UserName;
                        marker = 1;
                    }
                    _ => return None,
                },
                Tags => {
                    if codepoint == ' ' {
                        state = UserName;
                        tags_map = parse_tags(&message[1..i]);
                        marker = i + 2;
                    }
                }
                UserName => match codepoint {
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
                            "PRIVMSG" => {
                                state = Channel;
                            }
                            "JOIN" => {
                                return Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Join(
                                    user_name.to_string(),
                                )))
                            }
                            "PART" => {
                                return Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Part(
                                    user_name.to_string(),
                                )))
                            }
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
                    let badges = get_badges(tags_map);
                    let user_info = UserInfo {
                        name: user_name.to_owned(),
                        badges,
                    };
                    let user_message = message[i..].trim();
                    if codepoint == '!' {
                        let (command_kind, command_options) =
                            ReceiveEvent::parse_command_from_message(user_message)?;
                        return Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Command(Command {
                            kind: command_kind,
                            options: command_options,
                            user: user_info,
                        })));
                    } else {
                        return Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::TextMessage(
                            TextMessage {
                                text: message[i..].trim().to_owned(),
                                user: user_info,
                            },
                        )));
                    }
                }
            }
        }
        None
    }
}

fn parse_tags(tags_string: &str) -> HashMap<String, String> {
    tags_string
        .split(';')
        .map(|key_val_pair| {
            let mut key_val_split = key_val_pair.split('=');
            (
                key_val_split.next().unwrap_or_default().to_owned(),
                key_val_split.next().unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

fn get_badges(tags: HashMap<String, String>) -> HashSet<Badge> {
    if let Some(badges) = tags.get("badges") {
        if badges.is_empty() {
            return HashSet::default();
        }
        badges
            .split(',')
            .map(|s| {
                // TODO: use filter_map here to avoid potential panics
                let mut splt = s.split('/');
                Badge {
                    name: splt.next().map(String::from).unwrap(),
                    level: splt.next().and_then(|s| s.parse().ok()).unwrap_or(0),
                }
            })
            .collect()
    } else {
        HashSet::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_user_messages() {
        let message = "@badge-info=;badges=;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :chatter!chatter@chatter.tmi.twitch.tv PRIVMSG #channel123 :This is a test message";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::TextMessage(
            TextMessage {
                text: "This is a test message".to_owned(),
                user: UserInfo {
                    name: "chatter".to_owned(),
                    badges: HashSet::default(),
                },
            },
        )));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_user_messages_with_trailing_newlines() {
        let message = "@badge-info=;badges=;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :chatter!chatter@chatter.tmi.twitch.tv PRIVMSG #channel123 :This is a test message\n";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::TextMessage(
            TextMessage {
                text: "This is a test message".to_owned(),
                user: UserInfo {
                    name: "chatter".to_owned(),
                    badges: HashSet::default(),
                },
            },
        )));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_badges_list() {
        let message = "@badge-info=;badges=badge1/2,badge2/10;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :chatter!chatter@chatter.tmi.twitch.tv PRIVMSG #channel123 :This is a test message";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::TextMessage(
            TextMessage {
                text: "This is a test message".to_owned(),
                user: UserInfo {
                    name: "chatter".to_owned(),
                    badges: HashSet::from([
                        Badge {
                            name: "badge1".to_owned(),
                            level: 2,
                        },
                        Badge {
                            name: "badge2".to_owned(),
                            level: 10,
                        },
                    ]),
                },
            },
        )));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_help_command() {
        let message = "@badge-info=;badges=;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :chatter!chatter@chatter.tmi.twitch.tv PRIVMSG #channel123 :!help";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Command(Command {
            kind: CommandType::Help,
            options: Vec::default(),
            user: UserInfo {
                name: "chatter".to_owned(),
                badges: HashSet::default(),
            },
        })));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_info_command() {
        let message = "@badge-info=;badges=;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :chatter!chatter@chatter.tmi.twitch.tv PRIVMSG #channel123 :!info";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Command(Command {
            kind: CommandType::Info,
            options: Vec::default(),
            user: UserInfo {
                name: "chatter".to_owned(),
                badges: HashSet::default(),
            },
        })));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_join() {
        let message = ":carkhy!carkhy@carkhy.tmi.twitch.tv JOIN #captaincallback";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Join(
            "carkhy".to_owned(),
        )));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_part() {
        let message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PART #captaincallback";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Part(
            "carkhy".to_owned(),
        )));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_slap_command() {
        let message = "@badge-info=;badges=;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :chatter!chatter@chatter.tmi.twitch.tv PRIVMSG #channel123 :!slap anotheruser";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Command(Command {
            kind: CommandType::Slap,
            options: vec!["anotheruser".to_owned()],
            user: UserInfo {
                name: "chatter".to_owned(),
                badges: HashSet::default(),
            },
        })));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_newcommand_command() {
        let message = "@badge-info=;badges=;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :chatter!chatter@chatter.tmi.twitch.tv PRIVMSG #channel123 :!newcommand command Text to output";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Command(Command {
            kind: CommandType::NewCommand,
            options: vec![
                "command".to_owned(),
                "Text".to_owned(),
                "to".to_owned(),
                "output".to_owned(),
            ],
            user: UserInfo {
                name: "chatter".to_owned(),
                badges: HashSet::default(),
            },
        })));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_removecommand_command() {
        let message = "@badge-info=;badges=;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :chatter!chatter@chatter.tmi.twitch.tv PRIVMSG #channel123 :!removecommand command";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Command(Command {
            kind: CommandType::RemoveCommand,
            options: vec!["command".to_owned()],
            user: UserInfo {
                name: "chatter".to_owned(),
                badges: HashSet::default(),
            },
        })));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_discord_command() {
        let message = "@badge-info=;badges=;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :chatter!chatter@chatter.tmi.twitch.tv PRIVMSG #channel123 :!discord";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Command(Command {
            kind: CommandType::Discord,
            options: Vec::default(),
            user: UserInfo {
                name: "chatter".to_owned(),
                badges: HashSet::default(),
            },
        })));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_dynamic_command() {
        let message = "@badge-info=;badges=;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :chatter!chatter@chatter.tmi.twitch.tv PRIVMSG #channel123 :!unknown command";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Command(Command {
            kind: CommandType::Dynamic("unknown".to_owned()),
            options: vec!["command".to_owned()],
            user: UserInfo {
                name: "chatter".to_owned(),
                badges: HashSet::default(),
            },
        })));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }

    #[test]
    fn parsing_newrepeating_command() {
        let message = "@badge-info=;badges=;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :chatter!chatter@chatter.tmi.twitch.tv PRIVMSG #channel123 :!newrepeating command 60 Text to output";
        let expected = Some(ReceiveEvent::ChatBotEvent(ChatBotEvent::Command(Command {
            kind: CommandType::NewRepeating,
            options: vec![
                "command".to_owned(),
                "60".to_owned(),
                "Text".to_owned(),
                "to".to_owned(),
                "output".to_owned(),
            ],
            user: UserInfo {
                name: "chatter".to_owned(),
                badges: HashSet::default(),
            },
        })));
        assert_eq!(ReceiveEvent::parse_from_message(message), expected);
    }
}
