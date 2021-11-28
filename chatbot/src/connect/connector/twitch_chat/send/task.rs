pub enum SendTask<'a> {
    PrivateMessage(&'a str, &'a str),
    ProvideLoginPassword(&'a str),
    ProvideLoginUserName(&'a str),
    JoinChannel(&'a str),
    RequestCapabilities(&'a str),
    Pong,
}

impl<'a> ToString for SendTask<'a> {
    fn to_string(&self) -> String {
        match self {
            Self::PrivateMessage(channel, message) => format!("PRIVMSG #{} :{}", channel, message),
            Self::ProvideLoginPassword(password) => format!("PASS oauth:{}", password),
            Self::ProvideLoginUserName(user_name) => format!("NICK {}", user_name),
            Self::JoinChannel(channel) => format!("JOIN #{}", channel),
            Self::RequestCapabilities(capability_name) => {
                format!("CAP REQ :twitch.tv/{}", capability_name)
            }
            Self::Pong => format!("PONG :tmi.twitch.tv"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_private_messages_correctly() {
        let task = SendTask::PrivateMessage("channelname", "Message");
        assert_eq!(task.to_string(), "PRIVMSG #channelname :Message");
    }

    #[test]
    fn prints_login_password_messages_correctly() {
        let task = SendTask::ProvideLoginPassword("admin123");
        assert_eq!(task.to_string(), "PASS oauth:admin123");
    }

    #[test]
    fn prints_login_username_messages_correctly() {
        let task = SendTask::ProvideLoginUserName("user123");
        assert_eq!(task.to_string(), "NICK user123");
    }

    #[test]
    fn prints_join_channel_messages_correctly() {
        let task = SendTask::JoinChannel("channel123");
        assert_eq!(task.to_string(), "JOIN #channel123");
    }

    #[test]
    fn prints_request_capabilities_messages_correctly() {
        let task = SendTask::RequestCapabilities("capability123");
        assert_eq!(task.to_string(), "CAP REQ :twitch.tv/capability123");
    }

    #[test]
    fn prints_pong_messages_correctly() {
        let task = SendTask::Pong;
        assert_eq!(task.to_string(), "PONG :tmi.twitch.tv");
    }
}
