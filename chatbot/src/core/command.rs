#[derive(Debug)]
pub enum ChatBotCommand {
    SendMessage(String),
    LogTextMessage(String),
}
