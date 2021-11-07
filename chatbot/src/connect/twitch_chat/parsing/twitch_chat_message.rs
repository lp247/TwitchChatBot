use std::str::FromStr;

use super::{ping_message::PingMessage, user_message::UserMessage};
use crate::connect::ConnectorError;

#[derive(Debug)]
pub enum TwitchChatMessage {
    PingMessage(PingMessage),
    UserMessage(UserMessage),
}

impl FromStr for TwitchChatMessage {
    type Err = ConnectorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with(':') {
            Ok(Self::UserMessage(UserMessage::from_str(s)?))
        } else {
            Ok(Self::PingMessage(PingMessage::from_str(s)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::connect::MessageContent;

    use super::*;

    #[test]
    fn parsing_user_messages() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message";
        let parsed = TwitchChatMessage::from_str(raw_message);
        assert!(parsed.is_ok());
        if let TwitchChatMessage::UserMessage(user_message) = parsed.unwrap() {
            assert_eq!(user_message.user_name, "carkhy");
            if let MessageContent::Text(text) = user_message.content {
                assert_eq!(
                    text,
                    "a function that takes a string and returns the message"
                );
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }

    #[test]
    fn parsing_user_messages_with_trailing_newlines() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message\n";
        let parsed = TwitchChatMessage::from_str(raw_message);
        assert!(parsed.is_ok());
        if let TwitchChatMessage::UserMessage(user_message) = parsed.unwrap() {
            assert_eq!(user_message.user_name, "carkhy");
            if let MessageContent::Text(text) = user_message.content {
                assert_eq!(
                    text,
                    "a function that takes a string and returns the message"
                );
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }

    #[test]
    fn parsing_ping_messages() {
        let ping_message = "PING :tmi.twitch.tv";
        let parsed = TwitchChatMessage::from_str(ping_message);
        assert!(parsed.is_ok());
        if let TwitchChatMessage::PingMessage(server) = parsed.unwrap() {
            assert_eq!(server.0, "tmi.twitch.tv");
        } else {
            unreachable!();
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
