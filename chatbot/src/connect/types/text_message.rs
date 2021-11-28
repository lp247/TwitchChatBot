use super::UserInfo;

#[derive(Debug)]
pub struct TextMessage {
    pub text: String,
    pub user: UserInfo,
}

// Example text: #channel_name :backseating backseating
impl TextMessage {
    pub fn new(text: &str, user: UserInfo) -> Self {
        Self {
            text: text.to_owned(),
            user,
        }
    }
}
