use std::path::Path;

use anyhow::Result;
use lofigirl_shared_common::config::{
    ConfigError, LastFMConfig, ListenBrainzConfig, ServerConfig, VideoConfig,
};
use serde::{Deserialize, Serialize};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub lastfm: Option<LastFMConfig>,
    pub listenbrainz: Option<ListenBrainzConfig>,
    pub session: Option<TokenConfig>,
    pub video: Option<VideoConfig>,
    pub server: Option<ServerConfig>,
}

impl Config {
    pub async fn from_toml(file_name: &Path) -> Result<Config> {
        let file_contents = String::from_utf8(tokio::fs::read(file_name).await?)?;
        let config: Config = toml::from_str(&file_contents)?;
        (config.lastfm.is_some() && config.listenbrainz.is_some())
            .then(|| ())
            .ok_or(ConfigError::EmptyListeners)?;
        #[cfg(feature = "standalone")]
        config
            .video
            .is_some()
            .then(|| ())
            .ok_or(ConfigError::EmptyVideoConfig)?;
        #[cfg(not(feature = "standalone"))]
        config
            .server
            .is_some()
            .then(|| ())
            .ok_or(ConfigError::EmptyServerConfig)?;
        Ok(config)
    }

    pub async fn to_toml(&self, filename: &Path) -> Result<()> {
        let contents = toml::to_string(self)?;
        let mut buffer = OpenOptions::new().append(false).open(filename).await?;
        buffer.write_all(contents.as_bytes()).await?;
        Ok(())
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenConfig {
    pub token: String,
}
