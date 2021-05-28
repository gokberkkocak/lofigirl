use std::path::Path;

use anyhow::Result;
use lofigirl_shared_common::config::{LastFMApiConfig, VideoConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub video: VideoConfig,
    pub lastfmapi: Option<LastFMApiConfig>,
}

impl ServerConfig {
    pub fn from_toml(file_name: &Path) -> Result<ServerConfig> {
        let file_contents = String::from_utf8(std::fs::read(file_name)?)?;
        let config: ServerConfig = toml::from_str(&file_contents)?;
        Ok(config)
    }
}
