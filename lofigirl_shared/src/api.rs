use serde::{Deserialize, Serialize};

use crate::{
    config::{LastFMConfig, ListenBrainzConfig},
    track::Track,
};
#[derive(Debug, Serialize, Deserialize)]
pub struct SendInfo {
    pub lastfm: Option<LastFMConfig>,
    pub listenbrainz: Option<ListenBrainzConfig>,
    pub action: Action,
    pub track: Track,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    PlayingNow,
    Listened,
}
