use serde_json::Value;
use std::env;
use websocket::{client::ClientBuilder, Message, OwnedMessage};
use reqwest;
use thiserror::Error;

static VALIDATION_URL: &str = "https://id.twitch.tv/oauth2/validate";
static REDIRECT_URI: &str = "https://localhost:3030";

#[derive(Error, Debug)]
pub enum TwitchError {
    #[error("Request failed: {0:?}")] 
    ReqwestError(reqwest::Error),
    #[error("Bad request: {0:?}")]
    BadRequest(String),
    #[error("Parsing error: {0:?}")]
    ReqwestParsingError(reqwest::Error),
    #[error("Parsing error: {0:?}")]
    SerdeJSONParsingError(serde_json::Error),
    #[error("Missing response JSON field: {0:?}")]
    MissingResponseJSONField(String, String),
    #[error("Server error: {0:?}")]
    ServerError(String),
    #[error("Could not receive request: {0:?}")]
    TinyHTTPReceiveError(std::io::Error),
}

pub type Result<T> = std::result::Result<T, TwitchError>;

pub struct AccessTokenDispenser {
    access_token: Option<String>,
    refresh_token: Option<String>,
}

impl AccessTokenDispenser {
    fn new() -> Self {
        Self {access_token: None, refresh_token: None}
    }

    fn extract_code_from_url<'a>(&'a self, request_url: &'a str) -> &'a str {
        let code_start = request_url.find("code=").unwrap() + 5;
        let code_end = code_start + 30;
        &request_url[code_start..code_end]
    }

    fn retrieve_code(&self) -> Result<String> {
        let ssl_config = tiny_http::SslConfig {
            certificate: include_bytes!("../certificates/cert.crt").to_vec(),
            private_key: include_bytes!("../certificates/cert.key").to_vec(),
        };
        let server = tiny_http::Server::https("0.0.0.0:3030", ssl_config).map_err(|err| TwitchError::ServerError(format!("{:?}", err)))?;
        println!("Open link https://id.twitch.tv/oauth2/authorize?client_id=3rmzgtjlcc01fup7gjs99ua4uoof3g&redirect_uri=https://localhost:3030&response_type=code&scope=chat:read%20chat:edit");
        let request: tiny_http::Request = server.recv().map_err(|err| TwitchError::TinyHTTPReceiveError(err))?;
        let request_url = request.url();
        let code = self.extract_code_from_url(request_url);
        Ok(code.to_owned())
    }

    async fn access_token_is_valid(&self) -> Result<bool> {
        if self.access_token.is_none() {
            return Ok(false);
        }
        let client = reqwest::Client::new();
        let access_token = self.access_token.as_ref().unwrap();
        let validation_response = client
            .get(VALIDATION_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|err| TwitchError::ReqwestError(err))?;
        if validation_response.status() != 200 {
            let response_text = validation_response.text().await.map_err(|err| TwitchError::ReqwestParsingError(err))?;
            let response_json: Value = serde_json::from_str(&response_text).map_err(|err| TwitchError::SerdeJSONParsingError(err))?;
            let response_message = response_json["message"].as_str().ok_or(TwitchError::MissingResponseJSONField("message".to_owned(), format!("{:?}", response_json)))?;
            return Err(TwitchError::BadRequest(response_message.to_owned()));
        }
        Ok(true)
    }

    async fn renew(&mut self) -> Result<()> {
        let code = self.retrieve_code()?;
        let uri = format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri={}", env::var("TWITCH_AUTH_CLIENT_ID").unwrap(), env::var("TWITCH_AUTH_CLIENT_SECRET").unwrap(), code, REDIRECT_URI);
        let client = reqwest::Client::new();
        let response = client.post(uri).send().await.map_err(|err| TwitchError::ReqwestError(err))?;
        let status = response.status();
        let response_text = response.text().await.map_err(|err| TwitchError::ReqwestParsingError(err))?;
        let json: Value = serde_json::from_str(&response_text).map_err(|err| TwitchError::SerdeJSONParsingError(err))?;
        if status != 200 {
            let message = json["message"].as_str().ok_or(TwitchError::MissingResponseJSONField("message".to_owned(), format!("{:?}", json)))?;
            return Err(TwitchError::BadRequest(message.to_owned()));
        }
        let access_token = json["access_token"].as_str().ok_or(TwitchError::MissingResponseJSONField("access_token".to_owned(), format!("{:?}", json)))?;
        let refresh_token = json["refresh_token"].as_str().ok_or(TwitchError::MissingResponseJSONField("refresh_token".to_owned(), format!("{:?}", json)))?;
        self.access_token = Some(access_token.to_owned());
        self.refresh_token = Some(refresh_token.to_owned());
        Ok(())
    }

    pub async fn get(&mut self) -> Result<&str> {
        if !self.access_token_is_valid().await? {
            self.renew().await?;
        }
        Ok(self.access_token.as_ref().expect("Unexpectedly the access token is not set after renewal!"))
    }
}

pub struct TwitchChatProxy {
    receiver: websocket::receiver::Reader<std::net::TcpStream>,
    sender: websocket::sender::Writer<std::net::TcpStream>,
    channel: String,
    access_token_dispenser: AccessTokenDispenser,
}

impl<'a> TwitchChatProxy {
    pub fn new(channel: &str) -> Self {
        let chat_client = ClientBuilder::new("ws://irc-ws.chat.twitch.tv:80")
            .unwrap()
            .connect_insecure()
            .unwrap();
        let (receiver, sender) = chat_client.split().unwrap();
        Self {receiver, sender, channel: String::from(channel), access_token_dispenser: AccessTokenDispenser::new()}
    }


    fn send_raw_message(&mut self, message: String) -> std::result::Result<(), websocket::WebSocketError> {
        let message_obj = Message::text(message);
        self.sender.send_message(&message_obj)
    }

    fn login(&mut self, access_token: &str) -> std::result::Result<(), websocket::WebSocketError> {
        self.send_raw_message(format!("PASS oauth:{}", access_token))?;
        self.send_raw_message(format!("NICK {}", env::var("TWITCH_CHAT_USER").unwrap()))?;
        self.send_raw_message(format!("JOIN #{}", self.channel))
    }

    pub async fn initialize(&mut self) -> std::result::Result<(), websocket::WebSocketError> {
        let access_token = self.access_token_dispenser.get().await.expect("Could not get access token").to_owned();
        self.login(access_token.as_str())?;
        Ok(()) 
        // let uri = format!("https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri=https://localhost:3030&response_type=code&scope=chat:read%20chat:edit", env::var("TWITCH_AUTH_CLIENT_ID").unwrap());
    }

    pub fn send_message(&mut self, msg: &str) -> std::result::Result<(), websocket::WebSocketError> {
        self.send_raw_message(format!("PRIVMSG #{} :{}", self.channel, msg))
    }

    pub fn incoming_messages(&mut self) -> impl Iterator<Item = String> + '_ {
        self.receiver.incoming_messages().filter_map(|message| match message { 
            Ok(m) => match m {
                OwnedMessage::Text(t) => t.split(":").nth(2).map(String::from),
                _ => return None,
            },
            Err(_) => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::proxies::AccessTokenDispenser;

    #[test]
    fn getting_code_out_of_url() {
        let atd = AccessTokenDispenser::new();
        let url = "https://localhost:3030/?code=y6gdw1g6vfi07y1otcpn5abfzb94n9&scope=chat%3Aread+chat%3Aedit";
        let code = "y6gdw1g6vfi07y1otcpn5abfzb94n9";
        assert_eq!(atd.extract_code_from_url(url), code);
    }
}
