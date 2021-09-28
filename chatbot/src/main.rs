extern crate websocket;

use reqwest;
use std::{collections::HashMap, env};
use serde::Deserialize;

const TWITCH_CHAT_SERVER: &'static str = "ws://irc-ws.chat.twitch.tv:80";

#[derive(Deserialize)]
struct AuthorizationData {
    access_token: String,
    expires_in: u32,
    token_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to {}", TWITCH_CHAT_SERVER);

    let mut app_authorization = HashMap::new();
    app_authorization.insert("access_token", "rust");
    app_authorization.insert("expires_in", "json");
    app_authorization.insert("refresh_token", "json");
    app_authorization.insert("scope", "json");
    app_authorization.insert("token_type", "json");

    let http_client = reqwest::Client::new();
    let auth_endpoint = format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials", env::var("CLIENT_ID").unwrap(), env::var("CLIENT_SECRET").unwrap());
    let access_token = http_client.post(auth_endpoint).send().await?.json::<AuthorizationData>().await.unwrap().access_token;

    // let chat_client = ClientBuilder::new(CONNECTION)
    //     .unwrap()
    //     .add_protocol("rust-websocket")
    //     .connect_insecure()
    //     .unwrap();

    // println!("Successfully connected");

    // let (_, mut sender) = client.split().unwrap();

    // let message = Message::text("Some message");
    // sender.send_message(&message);

    // println!("Waiting for child threads to exit");

    Ok(())
}
