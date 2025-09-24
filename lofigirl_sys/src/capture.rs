use anyhow::Result;
use rand::Rng;
use thiserror::Error;
use tracing::info;
use url::Url;

// Compile-time check to ensure exactly one backend is selected
#[cfg(not(any(
    all(feature = "rustube_backend", not(feature = "rusty_ytdl_backend"), not(feature = "native_yt_dlp")),
    all(feature = "rusty_ytdl_backend", not(feature = "rustube_backend"), not(feature = "native_yt_dlp")),
    all(feature = "native_yt_dlp", not(feature = "rustube_backend"), not(feature = "rusty_ytdl_backend"))
)))]
compile_error!("Exactly one YouTube backend must be enabled: 'rustube_backend', 'rusty_ytdl_backend', or 'native_yt_dlp'");

#[cfg(all(feature = "rustube_backend", not(feature = "rusty_ytdl_backend"), not(feature = "native_yt_dlp")))]
pub struct YoutubeLinkCapturer;
#[cfg(all(feature = "rustube_backend", not(feature = "rusty_ytdl_backend"), not(feature = "native_yt_dlp")))]
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

#[cfg(all(feature = "rusty_ytdl_backend", not(feature = "rustube_backend"), not(feature = "native_yt_dlp")))]
use std::io::Write;

#[cfg(all(feature = "rusty_ytdl_backend", not(feature = "rustube_backend"), not(feature = "native_yt_dlp")))]
pub struct YoutubeLinkCapturer {
    _temp_dir: tempfile::TempDir,
}
#[cfg(all(feature = "rusty_ytdl_backend", not(feature = "rustube_backend"), not(feature = "native_yt_dlp")))]
impl YoutubeLinkCapturer {
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        Ok(YoutubeLinkCapturer {
            _temp_dir: temp_dir,
        })
    }
    pub async fn get_raw_link(&self, url: &Url) -> Result<String> {
        // Generate random filename for this download
        let mut rng = rand::rng();
        let random_suffix = rng.random::<u64>();
        let chunk_path = self._temp_dir
            .path()
            .join(format!("current_chunk_{}", random_suffix));
        
        let video_options = rusty_ytdl::VideoOptions {
            quality: rusty_ytdl::VideoQuality::HighestVideo,
            ..Default::default()
        };
        let video = rusty_ytdl::Video::new_with_options(url.as_str(), video_options)?;
        let stream = video.stream().await?;
        // get one chunk and save to temp
        let mut raw_file = std::fs::File::create(&chunk_path)?;
        if let Some(chunk) = stream.chunk().await? {
            raw_file.write_all(&chunk)?;
        }
        let chunk_path_str = chunk_path
            .to_str()
            .ok_or(CaptureError::YoutubeLinkCaptureError)?;
        info!(
            "Raw stream snapshot is captured using rusty_ytdl to file: {}",
            chunk_path_str
        );
        Ok(chunk_path_str.to_owned())
    }
}

#[cfg(all(feature = "native_yt_dlp", not(feature = "rustube_backend"), not(feature = "rusty_ytdl_backend")))]
use std::process::Command;

#[cfg(all(feature = "native_yt_dlp", not(feature = "rustube_backend"), not(feature = "rusty_ytdl_backend")))]
pub struct YoutubeLinkCapturer {
    _temp_dir: tempfile::TempDir,
}
#[cfg(all(feature = "native_yt_dlp", not(feature = "rustube_backend"), not(feature = "rusty_ytdl_backend")))]
impl YoutubeLinkCapturer {
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        Ok(YoutubeLinkCapturer {
            _temp_dir: temp_dir,
        })
    }
    pub async fn get_raw_link(&self, url: &Url) -> Result<String> {
        info!("Raw video link capture using native yt-dlp for URL: {}", url);
        
        // Generate random filename for this download
        let mut rng = rand::rng();
        let random_suffix = rng.random::<u64>();
        let output_path = self._temp_dir.path().join(format!("yt_{}.mp4", random_suffix));
        
        // Run yt-dlp command to download 1-second segment
        let output = Command::new("yt-dlp")
            .arg(url.as_str())
            .arg("-o")
            .arg(&output_path)
            .arg("--download-sections")
            .arg("*00:00-00:01")
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("yt-dlp command failed: {}", stderr));
        }
        
        // Verify the file was created
        if !output_path.exists() {
            return Err(anyhow::anyhow!("yt-dlp did not create the expected output file"));
        }
        
        let output_path_str = output_path
            .to_str()
            .ok_or(CaptureError::YoutubeLinkCaptureError)?;
        
        info!(
            "Raw video segment captured using native yt-dlp to file: {}",
            output_path_str
        );
        
        Ok(output_path_str.to_owned())
    }
}

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Capturing the raw link has failed.")]
    YoutubeLinkCaptureError,
}
