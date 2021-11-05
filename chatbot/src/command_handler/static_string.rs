use super::CommandHandler;

struct StaticString {
    message: String,
}

impl StaticString {
    fn new(message: &str) -> Self {
        Self { message: message.to_owned() }
    }
}

impl CommandHandler for StaticString {
    fn run(&self, input: Option<&str>) -> &str {
        self.message.as_str()
    }
}
