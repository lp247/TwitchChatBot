#[derive(Debug)]
pub struct MessageInfo {
    pub user: String,
    pub text: String,
}

#[derive(Debug)]
pub enum MessageType {
    PrivateMessage(MessageInfo),
    PingMessage,
}

pub fn parse_message(raw_message: &str) -> Option<MessageType> {
    let words = raw_message.split(" ").collect::<Vec<&str>>();
    if *words.get(1)? == "PRIVMSG" {
        let user = extract_user(raw_message)?;
        let text = extract_text(raw_message)?;
        let message_info = MessageInfo {
            user: user.to_owned(),
            text: text.to_owned(),
        };
        return Some(MessageType::PrivateMessage(message_info));
    } else if *words.get(0)? == "PING" {
        return Some(MessageType::PingMessage);
    }
    None
}

fn extract_user(raw_message: &str) -> Option<&str> {
    raw_message.split("!").nth(0).map(|s| &s[1..])
}

fn extract_text(raw_message: &str) -> Option<&str> {
    raw_message.split(":").nth(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_private_messages() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message";
        assert!(matches!(parse_message(raw_message),
                         Some(MessageType::PrivateMessage(MessageInfo{user, text}))
                         if user == "carkhy" && text == "a function that takes a string and returns the message"
        ));
    }

    #[test]
    fn parsing_ping_messages() {
        let ping_message = "PING :tmi.twitch.tv";
        assert!(matches!(
            parse_message(ping_message),
            Some(MessageType::PingMessage)
        ));
    }
}
