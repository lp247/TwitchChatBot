use crate::{connect::ChatBotEvent, core::ChatBot};
use app_config::AppConfig;
use connect::connect_to_twitch_chat;
use flexi_logger::Logger;
use futures::channel::mpsc::{self};
use futures::future::select;
use futures::pin_mut;
use std::error::Error;
pub mod app_config;
mod connect;
mod core;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Logger::try_with_str("debug")?.start()?;
    let app_config = AppConfig::new()?;

    let (inc_tx, inc_rx) = mpsc::unbounded::<ChatBotEvent>();
    let (out_tx, out_rx) = mpsc::unbounded::<String>();

    let mut chat_bot = ChatBot::new();
    let chat_bot_run = chat_bot.run(inc_rx, out_tx);
    let twitch_connection = connect_to_twitch_chat(out_rx, inc_tx, &app_config);
    pin_mut!(twitch_connection, chat_bot_run);
    select(twitch_connection, chat_bot_run).await;
    Ok(())
}
