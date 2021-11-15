use crate::connect::EventContent;

#[derive(Debug)]
pub enum InternalEventContent {
    Ping(String),
}

#[derive(Debug)]
pub enum TwitchChatInternalEvent {
    Internal(InternalEventContent),
    External(EventContent),
}

impl TwitchChatInternalEvent {
    pub fn new(s: &str) -> Option<Self> {
        if s.starts_with(':') {
            Some(Self::External(EventContent::new(s)?))
        } else {
            s.strip_prefix("PING :")
                .map(String::from)
                .map(InternalEventContent::Ping)
                .map(Self::Internal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ping_messages_correctly() {
        let ping_message = "PING :tmi.twitch.tv";
        let parsed = TwitchChatInternalEvent::new(ping_message);
        assert!(parsed.is_some());
        if let TwitchChatInternalEvent::Internal(InternalEventContent::Ping(server)) =
            parsed.unwrap()
        {
            assert_eq!(server, "tmi.twitch.tv");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn parses_other_messages_starting_with_colon_correctly() {
        let message = ":Some other message";
        let parsed = TwitchChatInternalEvent::new(message);
        assert!(parsed.is_none());
    }

    #[test]
    fn parses_other_messages_starting_without_colon_correctly() {
        let message = "Some other message";
        let parsed = TwitchChatInternalEvent::new(message);
        assert!(parsed.is_none());
    }
}
