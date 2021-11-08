use super::CommandHandler;

pub struct StringInterpolationCommandHandler {
    message: String,
}

impl StringInterpolationCommandHandler {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}

impl CommandHandler for StringInterpolationCommandHandler {
    fn run(&self, options: Vec<String>) -> &str {
        self.message.as_str()
    }
}
