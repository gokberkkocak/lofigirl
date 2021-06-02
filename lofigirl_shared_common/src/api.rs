use serde::{Deserialize, Serialize};

use crate::{
    config::{LastFMClientPasswordConfig, LastFMClientSessionConfig, ListenBrainzConfig},
    track::Track,
};
#[derive(Debug, Serialize, Deserialize)]
pub struct ScrobbleRequest {
    pub lastfm: Option<LastFMClientSessionConfig>,
    pub listenbrainz: Option<ListenBrainzConfig>,
    pub action: Action,
    pub track: Track,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub password_config: LastFMClientPasswordConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub session_config: LastFMClientSessionConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    PlayingNow,
    Listened,
}
