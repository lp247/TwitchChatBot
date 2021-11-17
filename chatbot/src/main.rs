use chat_bot::ChatBot;
use std::error::Error;

extern crate websocket;

mod chat_bot;
mod connect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut chat_bot = ChatBot::new();
    chat_bot.run().await?;
    Ok(())
}
