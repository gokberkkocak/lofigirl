use crate::{config::ServerConfig, server::AppState};
use actix_web::web;
use anyhow::{bail, Context, Result};
use lofigirl_shared_common::{FAST_TRY_INTERVAL, REGULAR_INTERVAL, STREAM_LAST_READ_TIMEOUT};
use lofigirl_sys::image::ImageProcessor;
use tracing::{info, warn};
use url::Url;

pub struct InitServerWorker {
    pub state: web::Data<AppState>,
    image_procs_queue: Option<Vec<ImageProcessor>>,
}

impl InitServerWorker {
    pub async fn new(config: &ServerConfig) -> Result<InitServerWorker> {
        let image_procs_queue = config
            .video
            .iter()
            .flat_map(|v| {
                v.links.iter().map(|link| {
                    let video_url = Url::parse(link).ok()?;
                    let image_proc = ImageProcessor::new(video_url).ok()?;
                    Some(image_proc)
                })
            })
            .collect::<Option<Vec<_>>>();

        let len = if let Some(v) = &image_procs_queue {
            v.len()
        } else {
            0
        };

        let state = web::Data::new(
            AppState::new(
                config.lastfm_api.clone(),
                &config.server_settings.token_db,
                len,
            )
            .await?,
        );
        info!(
            "SeqServerWorker Worker initialized with {} image processors",
            len
        );
        Ok(InitServerWorker {
            state,
            image_procs_queue,
        })
    }

    pub async fn work(&mut self) -> Result<()> {
        let image_procs = self.image_procs_queue.take();
        match image_procs {
            Some(image_procs) => {
                let state = self.state.clone();
                InitServerWorker::start_workers(state, image_procs).await?;
            }
            None => bail!("Init image processors failed to start"),
        }
        Ok(())
    }

    async fn start_workers(
        state: web::Data<AppState>,
        image_procs: Vec<ImageProcessor>,
    ) -> Result<()> {
        // If no image processors, return
        if image_procs.is_empty() {
            info!("No image processors to start");
            return Ok(());
        }
        // Start the image processors
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
                            let mut lock = state_clone.seq_tracks.write();
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
            info!("Image processor worker {} started", idx);
            std::thread::sleep(artificial_delay);
        }
        for handle in handles {
            handle.await?;
        }
        Ok(())
    }
}

pub struct ServerWorker {
    pub state: web::Data<AppState>,
    image_proc: Option<ImageProcessor>,
}

impl ServerWorker {
    pub fn new(video_url: Url, state: web::Data<AppState>) -> Result<ServerWorker> {
        let image_proc = Some(ImageProcessor::new(video_url)?);
        Ok(ServerWorker { state, image_proc })
    }

    pub async fn work(&mut self) -> anyhow::Result<()> {
        let state_clone = self.state.clone();
        let mut image_proc = self
            .image_proc
            .take()
            .context("image processor not found")?;
        info!("ServerWorker starting for {}", &image_proc.video_url);
        actix_rt::spawn(async move {
            loop {
                // Check last read to check if we should stop
                match state_clone.last_requested.read().get(&image_proc.video_url) {
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
                // Set the track in the state
                let next_track = image_proc.next_track().await;
                match next_track {
                    Ok(track) => {
                        let mut lock = state_clone.tracks.write();
                        lock.insert(image_proc.video_url.clone(), track);
                        std::thread::sleep(*REGULAR_INTERVAL);
                    }
                    Err(e) => {
                        warn!("Problem with: {}", e);
                        std::thread::sleep(*FAST_TRY_INTERVAL);
                    }
                }
            }
        });
        Ok(())
    }
}
