use anyhow::Result;
use rand::Rng;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};
use thiserror::Error;
use tracing::info;
use url::Url;

#[cfg(feature = "alt_yt_backend")]
pub struct YoutubeLinkCapturer;
#[cfg(feature = "alt_yt_backend")]
impl YoutubeLinkCapturer {
    pub fn new() -> Result<Self> {
        Ok(YoutubeLinkCapturer)
    }
    pub async fn get_raw_link(&self, url: &Url) -> Result<String> {
        let descrambler = rustube::VideoFetcher::from_url(url)?
            .fetch()
            .await?
            .descramble()?;
        let raw_stream = descrambler
            .streams()
            .iter()
            .filter(|stream| stream.codecs.iter().any(|codec| codec.contains("vp9")))
            .max_by_key(|stream| stream.width)
            .ok_or(CaptureError::YoutubeLinkCaptureError)?;
        let raw_link = raw_stream.signature_cipher.url.to_string();
        info!("Raw video link is captured using rustube: {}", raw_link);
        Ok(raw_link)
    }
}

#[cfg(not(feature = "alt_yt_backend"))]
pub struct YoutubeLinkCapturer {
    temp_dir: TempDir,
    last_persistent_fetch_path: std::path::PathBuf,
    last_persistent_fetch_path_str: String,
}
#[cfg(not(feature = "alt_yt_backend"))]
impl YoutubeLinkCapturer {
    pub fn new() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let random_suffix = rng.gen::<u64>();
        let temp_dir = tempfile::tempdir()?;
        let last_persistent_fetch_path = temp_dir
            .path()
            .join(format!("current_chunk_{}", random_suffix));
        let last_persistent_fetch_path_str = last_persistent_fetch_path
            .as_os_str()
            .to_str()
            .ok_or(CaptureError::YoutubeLinkCaptureError)?
            .to_owned();
        Ok(YoutubeLinkCapturer {
            temp_dir,
            last_persistent_fetch_path,
            last_persistent_fetch_path_str,
        })
    }
    pub async fn get_raw_link(&self, url: &Url) -> Result<String> {
        info!("will capture livestream");
        let video_options = rusty_ytdl::VideoOptions {
            quality: rusty_ytdl::VideoQuality::HighestVideo,
            ..Default::default()
        };
        let video = rusty_ytdl::Video::new_with_options(url.as_str(), video_options)?;
        let stream = video.stream().await?;
        info!("livestream is captured");
        // get one chunk and save to temp
        let mut raw_file = NamedTempFile::new_in(&self.temp_dir)?;
        if let Some(chunk) = stream.chunk().await? {
            raw_file.write_all(&chunk)?;
        }
        raw_file.persist(&self.last_persistent_fetch_path)?;
        info!(
            "Raw stream snapshot is captured using rusty_ytdl to file: {}",
            &self.last_persistent_fetch_path_str
        );
        Ok(self.last_persistent_fetch_path_str.to_owned())
    }
}

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Capturing the raw link has failed.")]
    YoutubeLinkCaptureError,
}
