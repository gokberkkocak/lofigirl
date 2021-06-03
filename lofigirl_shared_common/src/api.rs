use std::fmt;

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
pub struct SessionRequest {
    pub password_config: LastFMClientPasswordConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_config: LastFMClientSessionConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRequest {
    pub lastfm_session_key: Option<String>,
    pub listenbrainz_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    PlayingNow,
    Listened,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Listened => write!(f, "Listened"),
            Action::PlayingNow => write!(f, "Playing Now"),
        }
    }
}
