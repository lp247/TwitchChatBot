use dotenv::dotenv;
use std::env::{self, VarError};
use thiserror::Error;

#[derive(Debug)]
pub struct AppConfig {
    pub channel_name: String,
    pub bot_user_name: String,
    pub twitch_client_id: String,
    pub twitch_client_secret: String,
}

#[derive(Debug, Error)]
pub enum AppConfigError {
    #[error("Environment variable error [{}]", .0)]
    EnvironmentVar(#[from] VarError),
}

impl AppConfig {
    pub fn new() -> Result<AppConfig, AppConfigError> {
        dotenv().ok();
        Ok(AppConfig {
            channel_name: env::var("TWITCH_CHANNEL")
                .unwrap_or_else(|_| "captaincallback".to_string()),
            bot_user_name: env::var("TWITCH_CHAT_USER")?,
            twitch_client_id: env::var("TWITCH_AUTH_CLIENT_ID")?,
            twitch_client_secret: env::var("TWITCH_AUTH_CLIENT_SECRET")?,
        })
    }

    /// Get a reference to the config's channel name.
    /// this value is provided by the TWITCH_CHANNEL environment variable
    pub fn channel_name(&self) -> &str {
        self.channel_name.as_ref()
    }

    /// Get a reference to the config's bot user name.
    /// this value is provided by the TWITCH_CHAR_USER environment variable
    pub fn bot_user_name(&self) -> &str {
        self.bot_user_name.as_ref()
    }

    /// Get a reference to the config's twitch client id.
    /// this value is provided by the TWITCH_AUTH_CLIENT_ID environment variable
    pub fn twitch_client_id(&self) -> &str {
        self.twitch_client_id.as_ref()
    }

    /// Get a reference to the config's twitch client secret.
    /// this value is provided by the TWITCH_AUTH_CLIENT_SECRET environment variable
    pub fn twitch_client_secret(&self) -> &str {
        self.twitch_client_secret.as_ref()
    }
}
