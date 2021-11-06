#[derive(Debug)]
pub struct MessageInfo {
    pub user: String,
    pub text: String,
}

#[derive(Debug)]
pub enum MessageType {
    UserMessage(MessageInfo),
    PingMessage(String),
}

// Example message: PING :tmi.twitch.tv
fn parse_full_ping_message(raw_message: &str) -> Option<MessageType> {
    if raw_message.starts_with("PING ") {
        return Some(MessageType::PingMessage(
            raw_message.chars().skip(6).collect(),
        ));
    }
    None
}

// Example message: :carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :backseating backseating
fn parse_full_text_message(raw_message: &str) -> Option<MessageType> {
    enum ParsingState {
        UserName,
        AdditionalUserInfo,
        MessageToken,
        Channel,
        MessageText,
    }
    use ParsingState::*;

    let mut state = UserName;
    let mut user_name = String::with_capacity(raw_message.len());
    let mut token = String::with_capacity(raw_message.len());

    for (i, codepoint) in raw_message.char_indices() {
        match state {
            UserName => match codepoint {
                // :carkhy!carkhy@carkhy.tmi.twitch.tv
                ':' => (),
                ' ' => return None,
                '!' => state = AdditionalUserInfo,
                _ => user_name.push(codepoint),
            },
            AdditionalUserInfo => {
                if codepoint == ' ' {
                    state = MessageToken
                }
            }
            MessageToken => {
                match codepoint {
                    // PRIVMSG #captaincallback :backseating backseating
                    ' ' => {
                        if token == "PRIVMSG" {
                            state = Channel;
                        } else {
                            // we're only interested in PRIVMSG
                            return None;
                        }
                    }
                    _ => token.push(codepoint),
                }
            }
            Channel => match codepoint {
                ' ' => {
                    state = MessageText;
                }
                _ => (),
            },
            MessageText => {
                return Some(MessageType::UserMessage(MessageInfo {
                    user: user_name,
                    text: raw_message[(i + 1)..].trim().to_owned(),
                }));
            }
        }
    }
    None
}

pub fn parse_full_message(raw_message: &str) -> Option<MessageType> {
    if raw_message.starts_with(":") {
        return parse_full_text_message(&raw_message);
    }
    parse_full_ping_message(&raw_message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_private_messages() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message";
        let parsed = parse_full_message(raw_message);
        assert!(parsed.is_some());
        if let MessageType::UserMessage(info) = parsed.unwrap() {
            assert_eq!(info.user, "carkhy");
            assert_eq!(
                info.text,
                "a function that takes a string and returns the message"
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn parsing_private_messages_with_trailing_newlines() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message\n";
        let parsed = parse_full_message(raw_message);
        assert!(parsed.is_some());
        if let MessageType::UserMessage(info) = parsed.unwrap() {
            assert_eq!(info.user, "carkhy");
            assert_eq!(
                info.text,
                "a function that takes a string and returns the message"
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn parsing_ping_messages() {
        let ping_message = "PING :tmi.twitch.tv";
        let parsed = parse_full_message(ping_message);
        assert!(parsed.is_some());
        if let MessageType::PingMessage(server) = parsed.unwrap() {
            assert_eq!(server, "tmi.twitch.tv");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn collect_after_skipping_past_the_end() {
        let s = String::from("bleh");
        let iter = s.chars().skip(35);
        let s2: String = iter.collect();
        assert_eq!(s2, "");
    }

    #[test]
    fn slice_starting_at_len() {
        let s = String::from("bleh");
        let slice = &s[s.len()..];
        assert_eq!(slice, "");
    }
}
