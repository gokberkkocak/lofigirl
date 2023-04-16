use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LastFMConfig {
    #[serde(flatten)]
    pub client: LastFMClientConfig,
    #[serde(flatten)]
    pub api: Option<LastFMApiConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum LastFMClientConfig {
    PasswordAuth(LastFMClientPasswordConfig),
    SessionAuth(LastFMClientSessionConfig),
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LastFMClientPasswordConfig {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LastFMClientSessionConfig {
    pub session_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LastFMApiConfig {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListenBrainzConfig {
    pub token: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoConfig {
    pub links: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub link: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerSettingsConfig {
    pub port: u32,
    pub token_db: String,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Neither LastFM nor Listenbrainz config is given.")]
    EmptyListeners,
    #[error("Video config not found.")]
    EmptyVideoConfig,
    #[error("Server config not found.")]
    EmptyServerConfig,
}
