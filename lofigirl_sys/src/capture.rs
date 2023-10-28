use anyhow::Result;
use rusty_ytdl::VideoQuality;
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
    persistent_temp_path: std::path::PathBuf,
}
#[cfg(not(feature = "alt_yt_backend"))]
impl YoutubeLinkCapturer {
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let persistent_temp_path = temp_dir.path().join("current_chunk");
        Ok(YoutubeLinkCapturer {
            temp_dir,
            persistent_temp_path,
        })
    }
    pub async fn get_raw_link(&self, url: &Url) -> Result<String> {
        let video_options = rusty_ytdl::VideoOptions {
            quality: VideoQuality::HighestVideo,
            ..Default::default()
          };
        let video = rusty_ytdl::Video::new_with_options(url.as_str(), video_options)?;
        let stream = video.stream().await?;
        // get one chunk and save to temp
        let mut raw_file = NamedTempFile::new_in(&self.temp_dir)?;
        if let Some(chunk) = stream.chunk().await.unwrap() {
            raw_file.write_all(&chunk)?;
        }
        raw_file.persist(&self.persistent_temp_path)?;
        let raw_file_name = self
            .persistent_temp_path
            .as_os_str()
            .to_str()
            .ok_or(CaptureError::YoutubeLinkCaptureError)?
            .to_owned();
        info!("Raw link is captured using ytextract: {}", &raw_file_name);
        Ok(raw_file_name)
    }
}

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Capturing the raw link has failed.")]
    YoutubeLinkCaptureError,
}
