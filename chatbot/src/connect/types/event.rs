use super::{text_message::TextMessage, Badge, Command, UserInfo};
use std::collections::{HashMap, HashSet};

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
        badges
            .split(',')
            .map(|s| {
                let mut splt = s.split('/');
                return Badge {
                    name: splt.next().map(String::from).unwrap(),
                    level: splt.next().and_then(|s| s.parse().ok()).unwrap_or(0),
                };
            })
            .collect()
    } else {
        HashSet::default()
    }
}

#[derive(Debug)]
pub enum ChatBotEvent {
    TextMessage(TextMessage),
    Command(Command),
    Part(String),
    Join(String),
}

// Example message: :carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :backseating backseating
impl ChatBotEvent {
    pub fn new(message: &str) -> Option<Self> {
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
                // @badge-info=;badges=;client-nonce=1e51cee7513a4516545bbc36a22f27eb;color=;display-name=carkhy;emotes=;first-msg=0;flags=;id=60904094-3684-4871-9e8c-1400648a804d;mod=0;room-id=120630112;subscriber=0;tmi-sent-ts=1637614002702;turbo=0;user-id=70346833;user-type= :carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :copy/paste that in your code to keep that valuable test case
                Tags => {
                    if codepoint == ' ' {
                        state = UserName;
                        tags_map = parse_tags(&message[1..i]);
                        marker = i + 2;
                    }
                }
                // :carkhy!carkhy@carkhy.tmi.twitch.tv
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
                            // (...) PRIVMSG #<channel> :backseating backseating
                            "PRIVMSG" => {
                                state = Channel;
                            }
                            // (...) JOIN #<channel>
                            "JOIN" => return Some(ChatBotEvent::Join(user_name.to_string())),
                            // (...) PART #<channel>
                            "PART" => return Some(ChatBotEvent::Part(user_name.to_string())),
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
                    let badges = get_badges(tags_map);
                    let user_info = UserInfo {
                        name: user_name.to_owned(),
                        badges,
                    };
                    if codepoint == '!' {
                        return Command::new(message[i..].trim(), user_info)
                            .map(ChatBotEvent::Command);
                    } else {
                        return Some(ChatBotEvent::TextMessage(TextMessage::new(
                            message[i..].trim(),
                            user_info,
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
    use super::*;

    fn user_message_helper(raw_message: &str, expected_text: &str, expected_user: &UserInfo) {
        let parsed = ChatBotEvent::new(raw_message);
        assert!(parsed.is_some());
        if let ChatBotEvent::TextMessage(user_message) = parsed.unwrap() {
            assert_eq!(user_message.user.name, expected_user.name);
            assert_eq!(user_message.user.badges, expected_user.badges);
            assert_eq!(user_message.text, expected_text);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn parsing_user_messages() {
        let raw_message = "@tag1=something;tag2= :carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message";
        let expected_text = "a function that takes a string and returns the message";
        let expected_user_info = UserInfo {
            name: "carkhy".to_owned(),
            badges: HashSet::default(),
        };
        user_message_helper(raw_message, expected_text, &expected_user_info);
    }

    #[test]
    fn parsing_user_messages_with_trailing_newlines() {
        let raw_message = "@tag1=something;tag2= :carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message\n";
        let expected_text = "a function that takes a string and returns the message";
        let expected_user_info = UserInfo {
            name: "carkhy".to_owned(),
            badges: HashSet::default(),
        };
        user_message_helper(raw_message, expected_text, &expected_user_info);
    }

    fn command_helper(raw_message: &str, expected_command: &str, expected_user: &UserInfo) {
        let parsed = ChatBotEvent::new(raw_message);
        assert!(parsed.is_some());
        if let ChatBotEvent::Command(command) = parsed.unwrap() {
            assert_eq!(command.name, expected_command);
            assert_eq!(command.user.name, expected_user.name);
            assert_eq!(command.user.badges, expected_user.badges);
            assert_eq!(command.options, Vec::<String>::new());
        } else {
            unreachable!();
        }
    }

    #[test]
    fn parsing_help_command_in_event_parser() {
        let raw_message = "@badges=badgename/10,other/20;tag2= :carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :!help";
        let expected_command = "help";
        let expected_user_info = UserInfo {
            name: "carkhy".to_owned(),
            badges: HashSet::from([
                Badge {
                    name: "badgename".to_owned(),
                    level: 10,
                },
                Badge {
                    name: "other".to_owned(),
                    level: 20,
                },
            ]),
        };
        command_helper(raw_message, expected_command, &expected_user_info);
    }

    #[test]
    fn parsing_info_command_in_event_parser() {
        let raw_message = "@badges=badgename/10,other/20;tag2= :carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :!info";
        let expected_command = "info";
        let expected_user_info = UserInfo {
            name: "carkhy".to_owned(),
            badges: HashSet::from([
                Badge {
                    name: "badgename".to_owned(),
                    level: 10,
                },
                Badge {
                    name: "other".to_owned(),
                    level: 20,
                },
            ]),
        };
        command_helper(raw_message, expected_command, &expected_user_info);
    }

    #[test]
    fn parsing_join_message() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv JOIN #captaincallback";
        let expected_user = "carkhy".to_owned();
        let parsed = ChatBotEvent::new(raw_message);
        assert!(parsed.is_some());
        if let ChatBotEvent::Join(user) = parsed.unwrap() {
            assert_eq!(user, expected_user);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn parsing_part_message() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PART #captaincallback";
        let expected_user = "carkhy".to_owned();
        let parsed = ChatBotEvent::new(raw_message);
        assert!(parsed.is_some());
        if let ChatBotEvent::Part(user) = parsed.unwrap() {
            assert_eq!(user, expected_user);
        } else {
            unreachable!();
        }
    }
}
