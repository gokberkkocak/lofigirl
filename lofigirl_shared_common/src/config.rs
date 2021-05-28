use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LastFMConfig {
    #[serde(flatten)]
    pub client: LastFMClientConfig,
    #[serde(flatten)]
    pub api: LastFMApiConfig,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
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
#[derive(Debug, Deserialize)]
pub struct VideoConfig {
    pub link: String,
    pub second_link: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub link: String,
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
