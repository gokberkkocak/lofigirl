use std::fmt;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use strsim::jaro;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Track {
    pub artist: String,
    pub song: String,
}

impl Track {
    pub fn from_ocr_text(text: &str) -> Result<Track> {
        let split_text = text.split_once('-').ok_or(TrackError::SplitError)?;
        let artist = split_text.0.to_owned();
        let song = split_text.1.to_owned();
        Ok(Track { artist, song })
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Track) -> bool {
        let sim = jaro(&self.artist, &other.artist) * jaro(&self.song, &other.song);
        #[cfg(debug_assertions)]
        println!("Track similarity: {}", sim);
        sim > 0.95
    }
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.artist, self.song)
    }
}

#[derive(Error, Debug)]
pub enum TrackError {
    #[error("OCR text cannot be split.")]
    SplitError,
}
