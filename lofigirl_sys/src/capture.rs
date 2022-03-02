use anyhow::Result;
use thiserror::Error;
use tracing::info;
use url::Url;

#[cfg(feature = "rustube")]
pub struct YoutubeLinkCapturer;
#[cfg(feature = "rustube")]
impl YoutubeLinkCapturer {
    pub fn new() -> Self {
        YoutubeLinkCapturer
    }
    pub async fn get_raw_link(&self, url: &Url) -> Result<String> {
        let raw_link = rustube::VideoFetcher::from_url(url)?
            .fetch()
            .await?
            .descramble()?
            .best_video()
            .ok_or(CaptureError::YoutubeLinkCaptureError)?
            .signature_cipher
            .url
            .to_string();
        info!("Raw video link is captured using rustube: {}", raw_link);
        Ok(raw_link)
    }
}

#[cfg(not(feature = "rustube"))]
pub struct YoutubeLinkCapturer {
    client: ytextract::Client,
}
#[cfg(not(feature = "rustube"))]
impl YoutubeLinkCapturer {
    pub fn new() -> YoutubeLinkCapturer {
        YoutubeLinkCapturer {
            client: ytextract::Client::new(),
        }
    }

    pub async fn get_raw_link(&self, url: &Url) -> Result<String> {
        let video = self.client.video(url.as_str().parse()?).await?;
        let raw_link = video
            .streams()
            .await?
            .filter_map(|stream| match stream {
                ytextract::Stream::Audio(_) => None,
                ytextract::Stream::Video(v) => Some(v),
            })
            .max_by_key(|stream| stream.width())
            .ok_or(CaptureError::YoutubeLinkCaptureError)?
            .url()
            .to_string();
        info!("Raw link is captured using ytextract: {}", raw_link);
        Ok(raw_link)
    }
}

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Capturing the raw link has failed.")]
    YoutubeLinkCaptureError,
}
