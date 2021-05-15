use lofigirl_shared::{config::ConfigError, track::Track};
#[cfg(not(feature = "standalone"))]
use lofigirl_shared::{CHILL_API_END_POINT, SLEEP_API_END_POINT};
#[cfg(feature = "standalone")]
use lofigirl_sys::image::ImageProcessor;
#[cfg(feature = "standalone")]
use url::Url;
use crate::{config::Config, listener::Listener};
use anyhow::Result;
#[cfg(not(feature = "standalone"))]
use reqwest::Client;
use thiserror::Error;

pub struct Worker {
    listener: Listener,
    #[cfg(feature = "standalone")]
    image_proc: ImageProcessor,
    #[cfg(not(feature = "standalone"))]
    client: Client,
    #[cfg(not(feature = "standalone"))]
    server_url: String,
    prev_track_with_count: Option<TrackWithCount>,
}

impl Worker {
    pub fn new(config: &Config, second: bool) -> Result<Worker> {
        let listener = Listener::new(&config)?;
        #[cfg(feature = "standalone")]
        let video_url = if second {
            Url::parse(
                &config
                    .video
                    .as_ref()
                    .ok_or(ConfigError::EmptyVideoConfig)?
                    .second_link
                    .as_ref()
                    .ok_or(WorkerError::_MissingSecondVideoLink)?,
            )?
        } else {
            Url::parse(
                &config
                    .video
                    .as_ref()
                    .ok_or(ConfigError::EmptyVideoConfig)?
                    .link,
            )?
        };
        #[cfg(feature = "standalone")]
        let image_proc = ImageProcessor::new(video_url)?;
        #[cfg(not(feature = "standalone"))]
        let client = Client::new();
        #[cfg(not(feature = "standalone"))]
        let server_url = format!(
            "{}/{}",
            &config
                .server
                .as_ref()
                .ok_or(ConfigError::EmptyServerConfig)?
                .link
                .as_str(),
            if second {
                &SLEEP_API_END_POINT
            } else {
                &CHILL_API_END_POINT
            }
        );
        Ok(Worker {
            listener,
            #[cfg(feature = "standalone")]
            image_proc,
            #[cfg(not(feature = "standalone"))]
            client,
            #[cfg(not(feature = "standalone"))]
            server_url,
            prev_track_with_count: None,
        })
    }

    #[cfg(not(feature = "standalone"))]
    pub async fn post(&self) -> Result<Track> {
        let response = self.client.get(&self.server_url).send().await?;
        let content = response.text().await?;
        let track: Track = serde_json::from_str(&content)?;
        Ok(track)
    }

    pub async fn work(&mut self) -> bool {
        match self.fragile_work().await {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Problem with: {}", e.to_string());
                false
            }
        }
    }

    async fn fragile_work(&mut self) -> Result<()> {
        #[cfg(not(feature = "standalone"))]
        let next_track = self.post().await?;
        #[cfg(feature = "standalone")]
        let next_track = self.image_proc.next_track().await?;
        let prev = self.prev_track_with_count.take();
        match prev {
            Some(mut t) if t.track == next_track && t.count == 3 => {
                self.listener.send_listen(&t.track)?;
                t.count += 1;
                self.prev_track_with_count = Some(t);
            }
            Some(mut t) if t.track == next_track => {
                t.count += 1;
                self.prev_track_with_count = Some(t);
            }
            Some(t) if t.count < 3 => {
                self.listener.send_listen(&t.track)?;
                self.listener.send_now_playing(&next_track)?;
                self.prev_track_with_count = Some(TrackWithCount::new(next_track));
            }
            _ => {
                self.listener.send_now_playing(&next_track)?;
                self.prev_track_with_count = Some(TrackWithCount::new(next_track));
            }
        }
        Ok(())
    }
}

struct TrackWithCount {
    pub track: Track,
    pub count: usize,
}

impl TrackWithCount {
    fn new(track: Track) -> TrackWithCount {
        TrackWithCount { track, count: 1 }
    }
}

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Second video link cannot be found on config file.")]
    _MissingSecondVideoLink,
}
