use super::UserInfo;

#[derive(Debug, PartialEq)]
pub struct TextMessage {
    pub text: String,
    pub user: UserInfo,
}
