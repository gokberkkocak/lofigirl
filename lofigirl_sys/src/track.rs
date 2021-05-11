use std::fmt;

use anyhow::Result;
use strsim::jaro;
use thiserror::Error;

pub struct Track {
    pub artist: String,
    pub song: String,
}

impl Track {
    pub fn from_ocr_text(text: &str) -> Result<Track> {
        let mut it = text.split("-");
        let artist = it.next().ok_or(TrackError::SplitError)?.trim().to_string();
        let song = it.next().ok_or(TrackError::SplitError)?.trim().to_string();
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