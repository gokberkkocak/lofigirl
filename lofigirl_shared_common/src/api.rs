use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{encrypt::SecureString, track::Track};
#[derive(Debug, Serialize, Deserialize)]
pub struct ScrobbleRequest {
    pub action: Action,
    pub track: Track,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRequest {
    pub username: String,
    pub secure_password: SecureString,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub secure_session_key: SecureString,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRequest {
    pub secure_lastfm_session_key: Option<SecureString>,
    pub secure_listenbrainz_token: Option<SecureString>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub secure_token: SecureString,
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
