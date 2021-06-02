use crate::{config::ServerConfig, server::AppState};
use actix_web::web;
use anyhow::Result;
use lofigirl_shared_common::{FAST_TRY_INTERVAL, REGULAR_INTERVAL};
use lofigirl_sys::image::ImageProcessor;
use thiserror::Error;
use url::Url;

pub struct ServerWorker {
    main_image_proc: ImageProcessor,
    second_image_proc: Option<ImageProcessor>,
    pub state: web::Data<AppState>,
}

impl ServerWorker {
    pub fn new(config: &ServerConfig, only_first: bool) -> Result<ServerWorker> {
        let main_video_url = Url::parse(&config.video.link)?;
        let main_image_proc = ImageProcessor::new(main_video_url)?;
        let second_image_proc = if !only_first {
            let second_video_url = Url::parse(
                &config
                    .video
                    .second_link
                    .as_ref()
                    .ok_or(WorkerError::MissingSecondLink)?,
            )?;
            Some(ImageProcessor::new(second_video_url)?)
        } else {
            None
        };
        let state = web::Data::new(AppState::new(config.lastfmapi.clone()));
        Ok(ServerWorker {
            main_image_proc,
            second_image_proc,
            state,
        })
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
                eprintln!("Problem with: {}", e.to_string());
                false
            }
        }
    }

    async fn fragile_work(&mut self) -> Result<()> {
        let next_track = self.main_image_proc.next_track().await?;
        let mut lock = self.state.main_track.lock();
        *lock = Some(next_track);
        if let Some(second_image_proc) = &mut self.second_image_proc {
            let second_next_track = second_image_proc.next_track().await?;
            let mut lock = self.state.second_track.lock();
            *lock = Some(second_next_track);
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Neither LastFM nor Listenbrainz config is given.")]
    MissingSecondLink,
}
