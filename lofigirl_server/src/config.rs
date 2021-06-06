use std::path::Path;

use anyhow::Result;
use lofigirl_shared_common::config::{LastFMApiConfig, VideoConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub video: VideoConfig,
    pub lastfm_api: Option<LastFMApiConfig>,
    #[serde(default = "default_port")]
    pub port: u32,
    #[serde(default = "default_token_db")]
    pub token_db: String,
}

fn default_port() -> u32 {
    8888
}
fn default_token_db() -> String {
    String::from("token.db")
}

impl ServerConfig {
    pub fn from_toml(file_name: &Path) -> Result<ServerConfig> {
        let file_contents = String::from_utf8(std::fs::read(file_name)?)?;
        let config: ServerConfig = toml::from_str(&file_contents)?;
        Ok(config)
    }
}
