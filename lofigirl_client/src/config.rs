use std::path::Path;

use anyhow::Result;
use lofigirl_shared_common::config::{
    ConfigError, LastFMApiConfig, LastFMClientConfig, ListenBrainzConfig, ServerConfig,
    ServerSettingsConfig, VideoConfig,
};
use serde::{Deserialize, Serialize};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub lastfm: Option<LastFMClientConfig>,
    pub lastfm_api: Option<LastFMApiConfig>,
    pub listenbrainz: Option<ListenBrainzConfig>,
    pub session: Option<TokenConfig>,
    pub video: Option<VideoConfig>,
    pub server: Option<ServerConfig>,
    #[allow(dead_code)]
    pub server_settings: Option<ServerSettingsConfig>,
}

impl Config {
    pub async fn from_toml(file_name: &Path) -> Result<Config> {
        let file_contents = String::from_utf8(tokio::fs::read(file_name).await?)?;
        let config: Config = toml::from_str(&file_contents)?;
        (config.lastfm.is_some() || config.listenbrainz.is_some())
            .then_some(())
            .ok_or(ConfigError::EmptyListeners)?;
        #[cfg(feature = "standalone")]
        config
            .video
            .is_some()
            .then_some(())
            .ok_or(ConfigError::EmptyVideoConfig)?;
        #[cfg(not(feature = "standalone"))]
        config
            .server
            .is_some()
            .then_some(())
            .ok_or(ConfigError::EmptyServerConfig)?;
        info!("Loaded config from {}", file_name.display());
        Ok(config)
    }

    pub async fn to_toml(&self, filename: &Path) -> Result<()> {
        let contents = toml::to_string(self)?;
        let mut buffer = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(filename)
            .await?;
        buffer.write_all(contents.as_bytes()).await?;
        Ok(())
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenConfig {
    pub token: String,
}
