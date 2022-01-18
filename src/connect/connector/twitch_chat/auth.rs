use std::collections::HashMap;

use super::retry_manager::ExponentialRetryManager;
use crate::{app_config::AppConfig, connect::error::ConnectorError};
use async_trait::async_trait;
use futures::future::TryFutureExt;
use futures_retry::FutureRetry;
use kv::*;
use serde_json::{from_str, Value};

const VALIDATION_URL: &str = "https://id.twitch.tv/oauth2/validate";
const REDIRECT_URI: &str = "https://localhost:3030";
const AUTH_CONFIG_FILE: &str = "./auth_store";
const AUTH_BUCKET_NAME: &str = "auth_config";
const ACCESS_TOKEN_PERSISTENCE_KEY: &str = "access_token";
const REFRESH_TOKEN_PERSISTENCE_KEY: &str = "refresh_token";

fn get_json_from_response_text(response_text: String) -> Result<Value, ConnectorError> {
    let val: Value = from_str(&response_text)?;
    Ok(val)
}

fn extract_code_from_url(request_url: &str) -> &str {
    let code_start = request_url.find("code=").unwrap() + 5;
    let code_end = code_start + 30;
    &request_url[code_start..code_end]
}

fn create_url_with_query_params(base: &str, query_params: &HashMap<&str, &str>) -> String {
    let query_params_options_strings: Vec<String> = query_params
        .iter()
        .map(|entry| format!("{}={}", entry.0, entry.1))
        .collect();
    format!("{}?{}", base, query_params_options_strings.join("&"))
}

async fn access_token_is_valid(access_token: &str) -> Result<bool, ConnectorError> {
    log::debug!("Checking validity of access token");
    let client = reqwest::Client::new();
    let validation_response = client
        .get(VALIDATION_URL)
        .bearer_auth(access_token)
        .send()
        .await?;
    let status_code = validation_response.status();
    let response_text = validation_response.text().await?;
    log::debug!("Got validity check server response {}", &response_text);
    match status_code.as_u16() {
        401 => Ok(false),
        200 => Ok(true),
        status_code => Err(ConnectorError::ExternalServerError(format!(
            "Access token validation server sent bad response with http status code {}",
            status_code
        ))),
    }
}

async fn access_token_is_valid_retrying(access_token: &str) -> Result<bool, ConnectorError> {
    FutureRetry::new(
        || access_token_is_valid(access_token),
        ExponentialRetryManager::new(Some(1), Some(3)),
    )
    .await
    .map(|val| val.0)
    .map_err(|err| err.0)
}

fn retrieve_code(client_id: &str) -> Result<String, ConnectorError> {
    let ssl_config = tiny_http::SslConfig {
        certificate: include_bytes!("../../../../certificates/cert.crt").to_vec(),
        // TODO: security: must not include private keys in a binary !
        private_key: include_bytes!("../../../../certificates/cert.key").to_vec(),
    };
    let server = tiny_http::Server::https("0.0.0.0:3030", ssl_config).map_err(|err| {
        ConnectorError::MessageReceiveFailed(format!(
            "Could not start server to get code: {:?}",
            err
        ))
    })?;
    println!(
            "Open link https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri=https://localhost:3030&response_type=code&scope=chat:read%20chat:edit",
            client_id,
        );
    let request: tiny_http::Request = server.recv().map_err(|err| {
        ConnectorError::MessageReceiveFailed(format!(
            "Could not receive request with code: {:?}",
            err
        ))
    })?;
    let request_url = request.url();
    let code = extract_code_from_url(request_url);
    Ok(code.to_owned())
}

fn load_saved_access_token() -> Result<(String, String), ConnectorError> {
    let cfg = Config::new(AUTH_CONFIG_FILE);
    let store = Store::new(cfg)?;
    let bucket = store.bucket::<String, String>(Some(AUTH_BUCKET_NAME))?;
    let access_token = bucket.get("access_token")?;
    let refresh_token = bucket.get("refresh_token")?;
    if access_token.is_some() && refresh_token.is_some() {
        Ok((access_token.unwrap(), refresh_token.unwrap()))
    } else {
        Err(ConnectorError::StoredValueNotAvailable(
            "access_token or refresh_token".to_owned(),
        ))
    }
}

async fn request_access_token(uri: &str) -> Result<(String, String), ConnectorError> {
    let client = reqwest::Client::new();
    let response = client.post(uri).send().await?;
    let status_code = response.status();
    let response_text = response.text().await?;
    log::debug!("Got token request server response {}", &response_text);
    match status_code.as_u16() {
        200 => {
            let json = get_json_from_response_text(response_text)?;
            let access_token = json["access_token"].as_str();
            let refresh_token = json["refresh_token"].as_str();
            if access_token.is_some() && refresh_token.is_some() {
                Ok((
                    access_token.unwrap().to_owned(),
                    refresh_token.unwrap().to_owned(),
                ))
            } else {
                Err(ConnectorError::ExternalServerError(
                    "Server did not provide access token or refresh token in response".to_owned(),
                ))
            }
        }
        // Invalid refresh token
        400 => {
            let json = get_json_from_response_text(response_text)?;
            let error_message = json["message"].as_str();
            Err(ConnectorError::HTTP400(
                error_message.unwrap_or_default().to_owned(),
            ))
        }
        403 => {
            let json = get_json_from_response_text(response_text)?;
            let error_message = json["message"].as_str();
            Err(ConnectorError::HTTP403(
                error_message.unwrap_or_default().to_owned(),
            ))
        }
        404 => Err(ConnectorError::HTTP404),
        status_code => Err(ConnectorError::ExternalServerError(format!(
            "Access token request server sent bad response with http status code {}",
            status_code
        ))),
    }
}

// https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-authorization-code-flow
async fn request_new_access_token(
    client_id: &str,
    client_secret: &str,
) -> Result<(String, String), ConnectorError> {
    // TODO: don't include the code retrieval in retrying
    let code = retrieve_code(client_id)?;
    let query_params: HashMap<&str, &str> = HashMap::from([
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", &code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", REDIRECT_URI),
    ]);
    let uri = create_url_with_query_params("https://id.twitch.tv/oauth2/token", &query_params);
    request_access_token(&uri).await
}

async fn request_new_access_token_retrying(
    client_id: &str,
    client_secret: &str,
) -> Result<(String, String), ConnectorError> {
    FutureRetry::new(
        || request_new_access_token(client_id, client_secret),
        ExponentialRetryManager::new(Some(1), Some(3)),
    )
    .await
    .map(|val| val.0)
    .map_err(|err| err.0)
}

// https://dev.twitch.tv/docs/authentication#refreshing-access-tokens
async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<(String, String), ConnectorError> {
    log::debug!("Refreshing access token");
    let query_params: HashMap<&str, &str> = HashMap::from([
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", &refresh_token),
        ("grant_type", "refresh_token"),
    ]);
    let uri = create_url_with_query_params("https://id.twitch.tv/oauth2/token", &query_params);
    request_access_token(&uri).await
}

async fn refresh_access_token_retrying(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<(String, String), ConnectorError> {
    FutureRetry::new(
        || refresh_access_token(client_id, client_secret, refresh_token),
        ExponentialRetryManager::new(Some(1), Some(3)),
    )
    .await
    .map(|val| val.0)
    .map_err(|err| err.0)
}

fn store_tokens(access_token: &str, refresh_token: &str) {
    let cfg = Config::new(AUTH_CONFIG_FILE);
    if let Err(_) = Store::new(cfg)
        .and_then(|store| store.bucket::<String, String>(Some(AUTH_BUCKET_NAME)))
        .and_then(|bucket| {
            let access_token_saving = bucket.set(ACCESS_TOKEN_PERSISTENCE_KEY, access_token);
            let refresh_token_saving = bucket.set(REFRESH_TOKEN_PERSISTENCE_KEY, refresh_token);
            access_token_saving.and(refresh_token_saving)
        })
    {
        log::warn!("Could not store access token or refresh token");
    }
}

#[async_trait]
pub trait AccessTokenDispenser {
    async fn get(&mut self) -> Result<&str, ConnectorError>;
}

pub struct TwitchAccessTokenDispenser<'a> {
    app_config: &'a AppConfig,
    access_token: String,
    refresh_token: String,
}

impl<'a> TwitchAccessTokenDispenser<'a> {
    pub async fn new(
        app_config: &'a AppConfig,
    ) -> Result<TwitchAccessTokenDispenser<'a>, ConnectorError> {
        let (access_token, refresh_token) = match load_saved_access_token() {
            Ok(val) => val,
            Err(_) => {
                let (access_token, refresh_token) = request_new_access_token_retrying(
                    app_config.twitch_client_id(),
                    app_config.twitch_client_secret(),
                )
                .await?;
                store_tokens(&access_token, &refresh_token);
                (access_token, refresh_token)
            }
        };
        Ok(Self {
            app_config,
            access_token,
            refresh_token,
        })
    }
}

#[async_trait]
impl<'a> AccessTokenDispenser for TwitchAccessTokenDispenser<'a> {
    async fn get(&mut self) -> Result<&str, ConnectorError> {
        log::debug!("Requesting access token");
        let access_token_valid = access_token_is_valid_retrying(&self.access_token).await?;
        let (access_token, refresh_token) = match access_token_valid {
            true => {
                log::debug!("Access token is valid");
                (self.access_token.to_owned(), self.refresh_token.to_owned())
            }
            false => {
                log::debug!("Access token is invalid");
                refresh_access_token_retrying(
                    &self.app_config.twitch_client_id(),
                    &self.app_config.twitch_client_secret(),
                    &self.refresh_token,
                )
                .or_else(|_| async {
                    request_new_access_token_retrying(
                        self.app_config.twitch_client_id(),
                        self.app_config.twitch_client_secret(),
                    )
                    .await
                })
                .await?
            }
        };
        store_tokens(&access_token, &refresh_token);
        self.access_token = access_token;
        self.refresh_token = refresh_token;
        Ok(self.access_token.as_ref())
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

    #[test]
    fn creating_url_with_query_params() {
        let query_params: HashMap<&str, &str> =
            HashMap::from([("param1", "firstvalue"), ("param2", "secondvalue")]);
        let base = "https://testserver.com";
        // https://testserver.com?param1=firstvalue&param2=secondvalue
        let generated_uri = create_url_with_query_params(base, &query_params);
        assert!(generated_uri.starts_with(base));
        assert!(generated_uri.contains("param1=firstvalue"));
        assert!(generated_uri.contains("param2=secondvalue"));
        assert_eq!(generated_uri.len(), base.len() + 37);
    }
}
