pub trait CommandHandler {
    fn run(&self, input: Option<&str>) -> &str;
}
