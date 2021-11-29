use super::user_info::UserInfo;

#[derive(Debug, PartialEq, Eq)]
pub enum CommandType {
    Help,
    Info,
    NewCommand,
    RemoveCommand,
    Slap,
    Discord,
    Dynamic(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub kind: CommandType,
    pub options: Vec<String>,
    pub user: UserInfo,
}
