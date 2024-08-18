use crate::{util::YoutubeIdExtractor, webserver::AppState};
use actix_web::web;
use anyhow::Context;
use lofigirl_shared_common::{
    track::Track, FAST_TRY_INTERVAL, REGULAR_INTERVAL, STREAM_LAST_READ_TIMEOUT,
};
use lofigirl_sys::image::ImageProcessor;
use tokio::sync::watch::Sender;
use tracing::{info, warn};
use url::Url;

pub struct ServerWorker {
    pub state: web::Data<AppState>,
    video_url: Url,
}

impl ServerWorker {
    pub fn new(video_url: Url, state: web::Data<AppState>) -> anyhow::Result<ServerWorker> {
        Ok(ServerWorker { state, video_url })
    }

    pub async fn work(&mut self, track_tx: Sender<Track>) -> anyhow::Result<()> {
        let state_clone = self.state.clone();
        let mut image_proc = ImageProcessor::new(self.video_url.clone())?;
        info!("New ServerWorker starting for {}", &image_proc.video_url);
        let youtube_video_id = self
            .video_url
            .get_video_id()
            .context("invalid youtube url")?;
        actix_rt::spawn(async move {
            loop {
                // Check last read to check if we should stop
                match state_clone.last_requested.read().get(&youtube_video_id) {
                    Some(instant) => {
                        if instant.elapsed() > *STREAM_LAST_READ_TIMEOUT {
                            info!(
                                "{} is not wanted by any client anymore, stopping",
                                image_proc.video_url
                            );
                            break;
                        }
                    }
                    None => {
                        warn!(
                            "{} is not available in the last requested list, stopping",
                            image_proc.video_url
                        );
                        break;
                    }
                }
                // Snap an image and fetch track info
                // If the track has changed, update state for REST endpoints and update channel for socket
                let next_track = image_proc.next_track().await;
                match next_track {
                    Ok(next_track) => {
                        let track = next_track.clone();
                        let old_track = state_clone
                            .tracks
                            .write()
                            .insert(youtube_video_id.clone(), track);
                        if old_track.filter(|old| *old == next_track).is_none()
                            && track_tx.send(next_track.clone()).is_err()
                        {
                            warn!("Channel problem")
                        }
                        tokio::time::sleep(*REGULAR_INTERVAL).await;
                    }
                    Err(e) => {
                        warn!("Problem with: {}", e);
                        tokio::time::sleep(*FAST_TRY_INTERVAL).await;
                    }
                }
            }
        });
        Ok(())
    }
}
