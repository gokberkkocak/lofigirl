use crate::{config::ServerConfig, server::AppState};
use actix_web::web;
use anyhow::{bail, Result};
use lofigirl_shared_common::{FAST_TRY_INTERVAL, REGULAR_INTERVAL};
use lofigirl_sys::image::ImageProcessor;
use thiserror::Error;
use tracing::{info, warn};
use url::Url;

pub struct ServerWorker {
    pub state: web::Data<AppState>,
    image_procs_queue: Option<Vec<ImageProcessor>>,
}

impl ServerWorker {
    pub async fn new(config: &ServerConfig) -> Result<ServerWorker> {
        if config.video.links.is_empty() {
            bail!(WorkerError::NoVideoLink);
        }
        let image_procs_queue = Some(
            config
                .video
                .links
                .iter()
                .filter_map(|link| {
                    let video_url = Url::parse(link).ok()?;
                    let image_proc = ImageProcessor::new(video_url).ok()?;
                    Some(image_proc)
                })
                .collect::<Vec<_>>(),
        );
        let state = web::Data::new(
            AppState::new(
                config.lastfm_api.clone(),
                &config.server_settings.token_db,
                config.video.links.len(),
            )
            .await?,
        );
        info!("Server Worker initialized");
        Ok(ServerWorker {
            state,
            image_procs_queue,
        })
    }

    pub async fn work(&mut self) -> Result<()> {
        let image_procs = self.image_procs_queue.take();
        match image_procs {
            Some(image_procs) => {
                let state = self.state.clone();
                ServerWorker::start_workers(state, image_procs).await?;
                Ok(())
            }
            None => bail!("No image processors found"),
        }
    }

    async fn start_workers(
        state: web::Data<AppState>,
        image_procs: Vec<ImageProcessor>,
    ) -> Result<()> {
        let mut handles = vec![];
        let nb_processors = image_procs.len();
        let artificial_delay = *REGULAR_INTERVAL / nb_processors as u32;
        for (idx, mut image_proc) in image_procs.into_iter().enumerate() {
            let state_clone = state.clone();
            let handle = actix_rt::spawn(async move {
                loop {
                    let next_track = image_proc.next_track().await;
                    match next_track {
                        Ok(track) => {
                            let mut lock = state_clone.tracks.lock();
                            lock[idx] = Some(track);
                            std::thread::sleep(*REGULAR_INTERVAL);
                        }
                        Err(e) => {
                            warn!("Problem with: {}", e);
                            std::thread::sleep(*FAST_TRY_INTERVAL);
                        }
                    }
                }
            });
            handles.push(handle);
            info!("image processor worker {} started", idx);
            std::thread::sleep(artificial_delay);
        }
        for handle in handles {
            handle.await?;
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("There are no video links.")]
    NoVideoLink,
}
