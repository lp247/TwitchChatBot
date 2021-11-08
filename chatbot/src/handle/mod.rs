mod static_string;

pub use static_string::StaticStringCommandHandler;

pub trait CommandHandler {
    fn run(&self) -> &str;
}
