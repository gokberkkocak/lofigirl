use anyhow::Result;
use rand::Rng;
use std::io::Write;
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
    _temp_dir: tempfile::TempDir,
    chunk_path: std::path::PathBuf,
}
#[cfg(not(feature = "alt_yt_backend"))]
impl YoutubeLinkCapturer {
    pub fn new() -> Result<Self> {
        let mut rng = rand::rng();
        let random_suffix = rng.random::<u64>();
        let temp_dir = tempfile::tempdir()?;
        let chunk_path = temp_dir
            .path()
            .join(format!("current_chunk_{}", random_suffix));
        Ok(YoutubeLinkCapturer {
            _temp_dir: temp_dir,
            chunk_path,
        })
    }
    pub async fn get_raw_link(&self, url: &Url) -> Result<String> {
        let video_options = rusty_ytdl::VideoOptions {
            quality: rusty_ytdl::VideoQuality::HighestVideo,
            ..Default::default()
        };
        let video = rusty_ytdl::Video::new_with_options(url.as_str(), video_options)?;
        let stream = video.stream().await?;
        // get one chunk and save to temp
        let mut raw_file = std::fs::File::create(&self.chunk_path)?;
        if let Some(chunk) = stream.chunk().await? {
            raw_file.write_all(&chunk)?;
        }
        let chunk_path_str = self
            .chunk_path
            .to_str()
            .ok_or(CaptureError::YoutubeLinkCaptureError)?;
        info!(
            "Raw stream snapshot is captured using rusty_ytdl to file: {}",
            chunk_path_str
        );
        Ok(chunk_path_str.to_owned())
    }
}

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Capturing the raw link has failed.")]
    YoutubeLinkCaptureError,
}
