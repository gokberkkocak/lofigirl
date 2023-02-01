use anyhow::Result;
use thiserror::Error;
use tracing::info;
use url::Url;

#[cfg(not(feature = "use_ytextract"))]
pub struct YoutubeLinkCapturer;
#[cfg(not(feature = "use_ytextract"))]
impl YoutubeLinkCapturer {
    pub fn new() -> Self {
        YoutubeLinkCapturer
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

#[cfg(feature = "use_ytextract")]
pub struct YoutubeLinkCapturer {
    client: ytextract::Client,
}
#[cfg(feature = "use_ytextract")]
impl YoutubeLinkCapturer {
    pub fn new() -> YoutubeLinkCapturer {
        YoutubeLinkCapturer {
            client: ytextract::Client::new(),
        }
    }

    pub async fn get_raw_link(&self, url: &Url) -> Result<String> {
        let video = self.client.video(url.as_str().parse()?).await?;

        let raw_stream = video
            .streams()
            .await?
            .filter_map(|stream| match stream {
                ytextract::Stream::Audio(_) => None,
                ytextract::Stream::Video(v) => Some(v),
            })
            .max_by_key(|stream| stream.width())
            .ok_or(CaptureError::YoutubeLinkCaptureError)?;
        let raw_link = raw_stream.url().to_string();
        info!("Raw link is captured using ytextract: {}", raw_link);
        Ok(raw_link)
    }
}

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Capturing the raw link has failed.")]
    YoutubeLinkCaptureError,
}
