use super::error::{Result, TwitchError};
use serde_json::Value;
use std::env;

static VALIDATION_URL: &str = "https://id.twitch.tv/oauth2/validate";
static REDIRECT_URI: &str = "https://localhost:3030";

fn extract_code_from_url(request_url: &str) -> &str {
    let code_start = request_url.find("code=").unwrap() + 5;
    let code_end = code_start + 30;
    &request_url[code_start..code_end]
}

pub struct AccessTokenDispenser {
    access_token: Option<String>,
    refresh_token: Option<String>,
}

impl AccessTokenDispenser {
    pub fn new() -> Self {
        Self {
            access_token: None,
            refresh_token: None,
        }
    }

    fn retrieve_code(&self) -> Result<String> {
        let ssl_config = tiny_http::SslConfig {
            certificate: include_bytes!("../../../certificates/cert.crt").to_vec(),
            private_key: include_bytes!("../../../certificates/cert.key").to_vec(),
        };
        let server = tiny_http::Server::https("0.0.0.0:3030", ssl_config)
            .map_err(|err| TwitchError::ServerError(format!("{:?}", err)))?;
        println!("Open link https://id.twitch.tv/oauth2/authorize?client_id=3rmzgtjlcc01fup7gjs99ua4uoof3g&redirect_uri=https://localhost:3030&response_type=code&scope=chat:read%20chat:edit");
        let request: tiny_http::Request = server
            .recv()
            .map_err(|err| TwitchError::TinyHTTPReceiveError(err))?;
        let request_url = request.url();
        let code = extract_code_from_url(request_url);
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
            let response_text = validation_response
                .text()
                .await
                .map_err(|err| TwitchError::ReqwestParsingError(err))?;
            let response_json: Value = serde_json::from_str(&response_text)
                .map_err(|err| TwitchError::SerdeJSONParsingError(err))?;
            let response_message =
                response_json["message"]
                    .as_str()
                    .ok_or(TwitchError::MissingResponseJSONField(
                        "message".to_owned(),
                        format!("{:?}", response_json),
                    ))?;
            return Err(TwitchError::BadRequest(response_message.to_owned()));
        }
        Ok(true)
    }

    async fn renew(&mut self) -> Result<()> {
        let code = self.retrieve_code()?;
        let uri = format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri={}", env::var("TWITCH_AUTH_CLIENT_ID").unwrap(), env::var("TWITCH_AUTH_CLIENT_SECRET").unwrap(), code, REDIRECT_URI);
        let client = reqwest::Client::new();
        let response = client
            .post(uri)
            .send()
            .await
            .map_err(|err| TwitchError::ReqwestError(err))?;
        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|err| TwitchError::ReqwestParsingError(err))?;
        let json: Value = serde_json::from_str(&response_text)
            .map_err(|err| TwitchError::SerdeJSONParsingError(err))?;
        if status != 200 {
            let message = json["message"]
                .as_str()
                .ok_or(TwitchError::MissingResponseJSONField(
                    "message".to_owned(),
                    format!("{:?}", json),
                ))?;
            return Err(TwitchError::BadRequest(message.to_owned()));
        }
        let access_token =
            json["access_token"]
                .as_str()
                .ok_or(TwitchError::MissingResponseJSONField(
                    "access_token".to_owned(),
                    format!("{:?}", json),
                ))?;
        let refresh_token =
            json["refresh_token"]
                .as_str()
                .ok_or(TwitchError::MissingResponseJSONField(
                    "refresh_token".to_owned(),
                    format!("{:?}", json),
                ))?;
        self.access_token = Some(access_token.to_owned());
        self.refresh_token = Some(refresh_token.to_owned());
        Ok(())
    }

    pub async fn get(&mut self) -> Result<&str> {
        if !self.access_token_is_valid().await? {
            self.renew().await?;
        }
        Ok(self
            .access_token
            .as_ref()
            .expect("Unexpectedly the access token is not set after renewal!"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getting_code_out_of_url() {
        let url1 = "https://localhost:3030/?code=y6gdw1g6vfi07y1otcpn5abfzb94n9&scope=chat%3Aread+chat%3Aedit";
        let url2 = "/?code=y6gdw1g6vfi07y1otcpn5abfzb94n9&scope=chat%3Aread+chat%3Aedit";
        let code = "y6gdw1g6vfi07y1otcpn5abfzb94n9";
        assert_eq!(extract_code_from_url(url1), code);
        assert_eq!(extract_code_from_url(url2), code);
    }
}
