use std::collections::HashMap;

#[derive(Debug)]
pub struct TextMessage {
    pub text: String,
    pub user_name: String,
    pub tags: HashMap<String, String>,
}

// Example text: #channel_name :backseating backseating
impl TextMessage {
    pub fn new(text: &str, user_name: &str, tags: HashMap<String, String>) -> Self {
        Self {
            text: text.to_owned(),
            user_name: user_name.to_owned(),
            tags,
        }
    }
}
