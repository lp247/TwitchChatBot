use super::CommandHandler;

pub struct StaticStringCommandHandler {
    message: String,
}

impl StaticStringCommandHandler {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}

impl CommandHandler for StaticStringCommandHandler {
    fn run(&self) -> &str {
        self.message.as_str()
    }
}
