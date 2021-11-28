use crate::connect::ChatBotEvent;

#[derive(Debug)]
pub enum ConnectorEvent {
    Ping,
}

impl ConnectorEvent {
    pub fn new(s: &str) -> Option<Self> {
        if !s.starts_with("PING :") {
            return None;
        }
        Some(Self::Ping)
    }
}

#[derive(Debug)]
pub enum ReceiveEvent {
    ChatBotEvent(ChatBotEvent),
    ConnectorEvent(ConnectorEvent),
}

impl ReceiveEvent {
    pub fn new(s: &str) -> Option<Self> {
        if let Some(event_content) = ChatBotEvent::new(s) {
            Some(Self::ChatBotEvent(event_content))
        } else {
            Some(Self::ConnectorEvent(ConnectorEvent::new(s)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ping_messages_correctly() {
        let ping_message = "PING :tmi.twitch.tv";
        let parsed = ConnectorEvent::new(ping_message);
        assert!(parsed.is_some());
        assert!(matches!(parsed.unwrap(), ConnectorEvent::Ping))
    }

    #[test]
    fn skips_non_ping_messages() {
        let message = "Some other message";
        let parsed = ConnectorEvent::new(message);
        assert!(parsed.is_none());
    }
}
