use rustube::VideoFetcher;
use anyhow::Result;
use thiserror::Error;
use url::Url;
pub struct YoutubeLinkCapture;

impl YoutubeLinkCapture {
    pub async fn get_raw_link(url: &Url) -> Result<String> {
        let raw_link = VideoFetcher::from_url(url)?
            .fetch()
            .await?
            .descramble()?
            .best_video()
            .ok_or(RustubeError::YoutubeLinkCaptureError)?
            .signature_cipher
            .url
            .to_string();
        #[cfg(debug_assertions)]
        println!("Raw link: {}", raw_link);
        Ok(raw_link)
    }
}

#[derive(Error, Debug)]
pub enum RustubeError {
    #[error("Capturing the raw link has failed.")]
    YoutubeLinkCaptureError,
}