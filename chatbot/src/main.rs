use crate::{
    connect::ChatBotEvent,
    core::{
        ChatBot,
        ChatBotCommand::{self, *},
    },
};
use app_config::AppConfig;
use connect::TwitchChatConnector;
use std::sync::mpsc;
use std::{error::Error, sync::mpsc::Sender};
use thread_timer::ThreadTimer;

pub mod app_config;
mod connect;
mod core;

fn process_command(
    command: ChatBotCommand,
    connector: &TwitchChatConnector,
    bot_event_sender: Sender<ChatBotEvent>,
) -> Result<(), Box<dyn Error>> {
    match command {
        SendMessage(message) => {
            println!("Sending this message : {}", &message);
            connector.send_message(&message)?;
        }
        LogTextMessage(message) => println!("{}", message),
        TimedCallback { duration, event } => {
            // This timer spawns a thread per invokation, that's bad
            // More serious timers were not a good fit (afaik)
            // It would be a nice exercise to implement such timer with
            // one or 2 threads for all timers
            let timer = ThreadTimer::new();
            let _ = timer.start(duration, move || {
                let _ = bot_event_sender.send(event);
            });
        }
        MultipleCommands(new_commands) => {
            for command in new_commands {
                process_command(command, connector, bot_event_sender.clone())?;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app_config = AppConfig::new()?;

    let (tx, rx) = mpsc::channel();

    let connector = TwitchChatConnector::new(&app_config, tx.clone()).await;
    connector.send_message("Hello, world!")?;

    let mut chat_bot = ChatBot::new();
    while let Ok(message) = rx.recv() {
        if let Some(bot_command) = chat_bot.handle_event(message) {
            process_command(bot_command, &connector, tx.clone())?;
        }
    }
    Ok(())
}
