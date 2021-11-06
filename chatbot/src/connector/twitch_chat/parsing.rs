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

pub fn parse_message(raw_message: &str) -> Option<MessageType> {
    enum State { Starting, ParsingUserName, SkipAdditionalUserInfo, ParsePing, ParseUserMessage }
    use State::*;

    let mut state = Starting;
    let mut user_name = String::with_capacity(raw_message.len());
    let mut token = String::with_capacity(raw_message.len());

    for (i, codepoint) in raw_message.char_indices() {
        match state {
            Starting => if codepoint == ':' { state = ParsingUserName } else {
                token.push(codepoint);
                state = ParsePing;
            },
            ParsingUserName => match codepoint { // :carkhy!carkhy@carkhy.tmi.twitch.tv
                ' ' => return None,
                '!' => state = SkipAdditionalUserInfo,
                _  => user_name.push(codepoint),
            },
            SkipAdditionalUserInfo => if codepoint == ' ' { state = ParseUserMessage },
            ParsePing => match codepoint { // PING :tmi.twitch.tv
                ' ' => if token == "PING" {
                    return Some(MessageType::PingMessage(
                        raw_message[i..].chars().skip(2).collect()
                    ))
                }
                _ => token.push(codepoint),
            },            
            ParseUserMessage => {
                match codepoint { // PRIVMSG #captaincallback :backseating backseating
                    ' ' => if token == "PRIVMSG" {
                        return Some(MessageType::UserMessage(
                            MessageInfo{
                                user: user_name,
                                text: raw_message[i..].chars()
                                    .skip(1) // skip space
                                    .skip_while(|c| *c != ' ') // skip chan name
                                    .skip(2) //skip space and colon
                                    .collect(), 
                            }));
                    } else { // we're only interested in PRIVMSG
                        return None;
                    },
                    _ => token.push(codepoint),
                }
            },
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_private_messages() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message";
        let parsed = parse_message(raw_message);
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
        let parsed = parse_message(raw_message);
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
        let parsed = parse_message(ping_message);
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
