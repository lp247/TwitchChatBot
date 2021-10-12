extern crate websocket;

use reqwest;
use serde::Deserialize;
use std::convert::Infallible;
use std::{env, thread};
use warp::filters::query::query;
use warp::Filter;
use websocket::client::ClientBuilder;
use websocket::Message;
use serde_json::Value;

const TWITCH_CHAT_SERVER: &'static str = "ws://irc-ws.chat.twitch.tv:80";

#[derive(Deserialize)]
struct AuthorizationData {
    access_token: String,
    expires_in: u32,
    token_type: String,
}

#[derive(Deserialize)]
struct RequestQueryParams {
    code: String,
}

async fn request_handler(params: RequestQueryParams) -> Result<impl warp::Reply, Infallible> {
    let uri = format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri=https://localhost:3030", env::var("TWITCH_AUTH_CLIENT_ID").unwrap(), env::var("TWITCH_AUTH_CLIENT_SECRET").unwrap(), params.code);
    let client = reqwest::Client::new();
    let response = client.post(uri).send().await.unwrap();
    let response_text = response.text().await.unwrap();
    let json: Value = serde_json::from_str(&response_text).unwrap();
    thread::spawn(move || {
        let chat_client = ClientBuilder::new(TWITCH_CHAT_SERVER)
            .unwrap()
            .connect_insecure()
            .unwrap();
        let (mut receiver, mut sender) = chat_client.split().unwrap();
        let password_msg = Message::text(format!("PASS oauth:{}", json["access_token"].as_str().unwrap()));
        let username_msg = Message::text(format!("NICK {}", env::var("TWITCH_CHAT_USER").unwrap()));
        sender.send_message(&password_msg);
        sender.send_message(&username_msg);
        for message in receiver.incoming_messages() {
            println!("Recv: {:?}", message.unwrap());
        }
    });

    Ok(String::from("OK"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let routes = warp::get()
        .and(query::<RequestQueryParams>())
        .and_then(request_handler);

    warp::serve(routes)
        .tls()
        .cert_path("certificates/cert.crt")
        .key_path("certificates/cert.key")
        .run(([127, 0, 0, 1], 3030))
        .await;
    // let routes = warp::get().and(warp::path("/")).and(query::<RequestQueryParams>()).map(request_handler);

    Ok(())
}
