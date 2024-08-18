use std::path::Path;

use anyhow::Result;
use lofigirl_shared_common::config::{LastFMApiConfig, ServerSettingsConfig};
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub lastfm_api: Option<LastFMApiConfig>,
    pub server_settings: ServerSettingsConfig,
}

impl ServerConfig {
    pub fn from_toml(file_name: &Path) -> Result<ServerConfig> {
        let file_contents = String::from_utf8(std::fs::read(file_name)?)?;
        let config: ServerConfig = toml::from_str(&file_contents)?;
        info!("Loaded config from {}", file_name.display());
        Ok(config)
    }
}
