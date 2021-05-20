use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LastFMConfig {
    pub api_key: String,
    pub api_secret: String,
    pub username: String,
    pub password: String,
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
