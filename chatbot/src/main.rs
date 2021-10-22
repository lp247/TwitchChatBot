extern crate websocket;

use reqwest;
use serde_json::Value;
use std::env;
use tiny_http;
use websocket::{client::ClientBuilder, Message};

struct TwitchChatProxy {
    receiver: websocket::receiver::Reader<std::net::TcpStream>,
    sender: websocket::sender::Writer<std::net::TcpStream>,
    channel: String,
}

impl TwitchChatProxy {
    fn new(channel: &str) -> Self {
        let chat_client = ClientBuilder::new("ws://irc-ws.chat.twitch.tv:80")
            .unwrap()
            .connect_insecure()
            .unwrap();
        let (receiver, sender) = chat_client.split().unwrap();
        Self {
            receiver,
            sender,
            channel: String::from(channel),
        }
    }

    fn get_code(&self) -> String {
        let ssl_config = tiny_http::SslConfig {
            certificate: include_bytes!("../certificates/cert.crt").to_vec(),
            private_key: include_bytes!("../certificates/cert.key").to_vec(),
        };
        let server = tiny_http::Server::https("0.0.0.0:3030", ssl_config).unwrap();
        let request: tiny_http::Request = server.recv().unwrap();
        let request_url = request.url();
        String::from(
            request_url
                .split("=")
                .nth(1)
                .unwrap()
                .split("&")
                .nth(0)
                .unwrap(),
        )
    }

    async fn get_access_token(&self, code: &str) -> String {
        let uri = format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri=https://localhost:3030", env::var("TWITCH_AUTH_CLIENT_ID").unwrap(), env::var("TWITCH_AUTH_CLIENT_SECRET").unwrap(), code);
        let client = reqwest::Client::new();
        let response = client.post(uri).send().await.unwrap();
        let response_text = response.text().await.unwrap();
        let json: Value = serde_json::from_str(&response_text).unwrap();
        String::from(json["access_token"].as_str().unwrap())
    }

    fn send_raw_message(&mut self, message: String) -> Result<(), websocket::WebSocketError> {
        let message_obj = Message::text(message);
        self.sender.send_message(&message_obj)
    }

    fn login(&mut self, access_token: &str) -> Result<(), websocket::WebSocketError> {
        self.send_raw_message(format!("PASS oauth:{}", access_token))?;
        self.send_raw_message(format!("NICK {}", env::var("TWITCH_CHAT_USER").unwrap()))?;
        self.send_raw_message(format!("JOIN #{}", self.channel))
    }

    async fn initialize(&mut self) -> Result<(), websocket::WebSocketError> {
        let code = self.get_code();
        let access_token = self.get_access_token(code.as_str()).await;
        self.login(access_token.as_str())?;
        Ok(()) 
        // let uri = format!("https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri=https://localhost:3030&response_type=code&scope=chat:read%20chat:edit", env::var("TWITCH_AUTH_CLIENT_ID").unwrap());
    }

    fn send_message(&mut self, msg: &str) -> Result<(), websocket::WebSocketError> {
        self.send_raw_message(format!("PRIVMSG #{} :{}", self.channel, msg))
    }

    fn get_receiver(&self) -> &websocket::receiver::Reader<std::net::TcpStream> {
        &self.receiver
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut proxy = TwitchChatProxy::new("captaincallback");
    proxy.initialize().await;
    proxy.send_message("Hello, World!");
    Ok(())
}
