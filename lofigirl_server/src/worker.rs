use crate::{config::ServerConfig, server::AppState};
use actix_web::web;
use anyhow::{bail, Result};
use lofigirl_shared_common::{FAST_TRY_INTERVAL, REGULAR_INTERVAL};
use lofigirl_sys::image::ImageProcessor;
use thiserror::Error;
use tracing::{info, warn};
use url::Url;

pub struct ServerWorker {
    image_procs: Vec<ImageProcessor>,
    pub state: web::Data<AppState>,
}

impl ServerWorker {
    pub async fn new(config: &ServerConfig) -> Result<ServerWorker> {
        if config.video.links.is_empty() {
            bail!(WorkerError::NoVideoLink);
        }
        let image_procs = config
            .video
            .links
            .iter()
            .filter_map(|link| {
                let video_url = Url::parse(&link).ok()?;
                let image_proc = ImageProcessor::new(video_url).ok()?;
                Some(image_proc)
            })
            .collect::<Vec<_>>();
        let state = web::Data::new(
            AppState::new(
                config.lastfm_api.clone(),
                &config.server_settings.token_db,
                config.video.links.len(),
            )
            .await?,
        );
        info!("Server Worker initialized");
        Ok(ServerWorker { image_procs, state })
    }

    pub async fn loop_work(&mut self) {
        loop {
            let wait_duration = match self.work().await {
                true => &REGULAR_INTERVAL,
                false => &FAST_TRY_INTERVAL,
            };
            std::thread::sleep(**wait_duration);
        }
    }

    pub async fn work(&mut self) -> bool {
        match self.fragile_work().await {
            Ok(_) => true,
            Err(e) => {
                warn!("Problem with: {}", e.to_string());
                false
            }
        }
    }

    async fn fragile_work(&mut self) -> Result<()> {
        let mut lock = self.state.tracks.lock();
        for (idx, image_proc) in self.image_procs.iter_mut().enumerate() {
            let next_track = image_proc.next_track().await;
            match next_track {
                Ok(track) => {
                    lock[idx] = Some(track);
                }
                Err(e) => {
                    warn!("Problem with: {}", e);
                }
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("There are no video links.")]
    NoVideoLink,
}
