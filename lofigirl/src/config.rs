use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub lastfm: Option<LastFMConfig>,
    pub listenbrainz: Option<ListenBrainzConfig>,
    pub video: VideoConfig,
}

impl Config {
    pub async fn from_toml(file_name: &Path) -> Result<Config> {
        let file_contents = String::from_utf8(tokio::fs::read(file_name).await?)?;
        let config: Config = toml::from_str(&file_contents)?;
        (config.lastfm.is_some() && config.listenbrainz.is_some())
            .then(|| ())
            .ok_or(ConfigError::EmptyListeners)?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
pub struct LastFMConfig {
    pub api_key: String,
    pub api_secret: String,
    pub username: String,
    pub password: String,
}
#[derive(Debug, Deserialize)]
pub struct ListenBrainzConfig {
    pub token: String,
}
#[derive(Debug, Deserialize)]
pub struct VideoConfig {
    pub link: String,
    pub second_link: Option<String>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Neither LastFM nor Listenbrainz config is given.")]
    EmptyListeners,
}
