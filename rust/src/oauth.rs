use axum::{Extension, Json, extract::Query, response::Response};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use tokio;
use uuid::Uuid;
use urlencoding;
use aide::axum::ApiRouter;
use anyhow::Result;

use crate::Config;

/// Discord OAuth configuration extracted from the main Config
#[derive(Debug, Clone)]
pub struct DiscordOAuthConfig {
    pub client_id: String,
    pub secret: String,
    pub redirect_uri: String,
}

impl DiscordOAuthConfig {
    /// Create DiscordOAuthConfig from the main application Config
    pub fn from_env(config: &Config) -> Self {
        Self {
            client_id: config.discord_client_id
                .clone()
                .expect("Discord client ID must be configured"),
            secret: config.discord_client_secret
                .clone()
                .expect("Discord client secret must be configured"),
            redirect_uri: config.discord_redirect_uri.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub email: Option<String>,
}

pub fn authorization_url(state: &str, cfg: &DiscordOAuthConfig) -> String {
    format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify%20email&state={}",
        cfg.client_id, 
        urlencoding::encode(&cfg.redirect_uri), 
        urlencoding::encode(state)
    )
}

pub async fn exchange_code(code: &str, cfg: &DiscordOAuthConfig) -> Result<TokenResponse> {
    let params = [
        ("client_id", cfg.client_id.as_str()),
        ("client_secret", cfg.secret.as_str()),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", cfg.redirect_uri.as_str()),
    ];

    let client = Client::new();
    let res = client
        .post("https://discord.com/api/oauth2/token")
        .form(&params)
        .send()
        .await?;

    let token_response: TokenResponse = res.json().await?;
    Ok(token_response)
}

pub async fn fetch_user(token: &str) -> Result<DiscordUser> {
    let client = Client::new();
    let res = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(token)
        .send()
        .await?;

    let user: DiscordUser = res.json().await?;
    Ok(user)
}

/// Creates and returns the OAuth API router
pub fn routes() -> ApiRouter {
    ApiRouter::new()
        // OAuth routes will be added here
}
