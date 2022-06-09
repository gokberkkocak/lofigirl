use anyhow::Result;
use leptess::LepTess;
use opencv::core::Vector;
use opencv::core::{Mat, MatTraitConst, Rect_};
use opencv::videoio::VideoCapture;
use opencv::videoio::VideoCaptureTrait;
use thiserror::Error;
use tracing::info;
use url::Url;

use crate::capture::YoutubeLinkCapturer;

use lofigirl_shared_common::track::Track;

const DPI: i32 = 70;

pub struct ImageProcessor {
    link_capturer: YoutubeLinkCapturer,
    video_url: Url,
    ocr: LepTess,
    low_bounds: Mat,
    high_bounds: Mat,
}

impl ImageProcessor {
    pub fn new(video_url: Url) -> Result<ImageProcessor> {
        let low_bounds = Mat::from_slice(&[200, 200, 200])?;
        let high_bounds = Mat::from_slice(&[255, 255, 255])?;
        let ocr = LepTess::new(None, "eng")?;
        let link_capturer = YoutubeLinkCapturer::new();
        Ok(ImageProcessor {
            link_capturer,
            video_url,
            ocr,
            low_bounds,
            high_bounds,
        })
    }

    pub async fn next_track(&mut self) -> Result<Track> {
        // CAPTURE
        let raw_link = self.link_capturer.get_raw_link(&self.video_url).await?;
        let mut capturer = VideoCapture::from_file(&raw_link, opencv::videoio::CAP_FFMPEG)?;
        capturer.set(opencv::videoio::CAP_PROP_FRAME_WIDTH, 1920.0)?;
        capturer.set(opencv::videoio::CAP_PROP_FRAME_HEIGHT, 1080.0)?;
        let mut full_image = Mat::default();
        let params = Vector::new();
        capturer
            .read(&mut full_image)?;
        #[cfg(debug_assertions)]
        opencv::imgcodecs::imwrite("debug_full.jpg", &full_image, &params)?;
        // CROP
        let image_dimensions = full_image.mat_size();
        (image_dimensions.len() == 2)
           .then(|| ())
           .ok_or(ImageProcessingError::ImageDimensionsError)?;
        let roi = Rect_::new(0, 0, 1920, 108);
        let cropped_image = Mat::roi(&full_image, roi)?;
        #[cfg(debug_assertions)]
        opencv::imgcodecs::imwrite("debug_cropped.jpg", &cropped_image, &params)?;
        // MASK
        let mut masked_image = Mat::default();
        opencv::core::in_range(
            &cropped_image,
            &self.low_bounds,
            &self.high_bounds,
            &mut masked_image,
        )?;
        #[cfg(debug_assertions)]
        opencv::imgcodecs::imwrite("debug_masked.jpg", &masked_image, &params)?;
        // ENCODE
        let mut buf = Vector::new();
        opencv::imgcodecs::imencode(".jpg", &masked_image, &mut buf, &params)?;
        // OCR
        self.ocr.set_image_from_mem(buf.as_slice())?;
        self.ocr.set_source_resolution(DPI);
        let ocr_text = self.ocr.get_utf8_text()?;
        info!("Track read using Tesseract OCR: {}", ocr_text);
        let track = Track::from_ocr_text(&ocr_text)?;
        Ok(track)
    }
}

#[derive(Error, Debug)]
pub enum ImageProcessingError {
    #[error("Read frame has no dimensions. There is a problem with the reading the stream.")]
    ImageDimensionsError,
}
