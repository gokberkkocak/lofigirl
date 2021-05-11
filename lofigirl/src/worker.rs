use lofigirl_sys::{image::ImageProcessor, track::Track};
use url::Url;

use crate::{config::Config, listener::Listener};
use anyhow::Result;
use thiserror::Error;

pub struct Worker {
    listener: Listener,
    image_proc: ImageProcessor,
    prev_track_with_count: Option<TrackWithCount>,
}

impl Worker {
    pub fn new(config: &Config, second: bool) -> Result<Worker> {
        let listener = Listener::new(&config)?;
        let video_url = if second {
            Url::parse(
                &config
                    .video
                    .second_link
                    .as_ref()
                    .ok_or(WorkerError::MissingSecondVideoLink)?,
            )?
        } else {
            Url::parse(&config.video.link)?
        };
        let image_proc = ImageProcessor::new(video_url)?;
        Ok(Worker {
            listener,
            image_proc,
            prev_track_with_count: None,
        })
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

    pub async fn fragile_work(&mut self) -> Result<()> {
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
            },
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
    #[error("Neither LastFM nor Listenbrainz config is given.")]
    MissingSecondVideoLink,
}
