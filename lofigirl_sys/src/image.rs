use std::sync::LazyLock;

use anyhow::Result;
use leptess::LepTess;
use opencv::boxed_ref::BoxedRef;
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
static LOW_BOUNDS: LazyLock<BoxedRef<Mat>> =
    LazyLock::new(|| Mat::from_slice(&[200, 200, 200]).unwrap());
static HIGH_BOUNDS: LazyLock<BoxedRef<Mat>> =
    LazyLock::new(|| Mat::from_slice(&[255, 255, 255]).unwrap());

pub struct ImageProcessor {
    link_capturer: YoutubeLinkCapturer,
    video_url: Url,
    ocr: LepTess,
}

impl ImageProcessor {
    pub fn new(video_url: Url) -> Result<ImageProcessor> {
        let ocr = LepTess::new(None, "eng")?;
        let link_capturer = YoutubeLinkCapturer::new()?;
        Ok(ImageProcessor {
            link_capturer,
            video_url,
            ocr,
        })
    }

    pub async fn next_track(&mut self) -> Result<Track> {
        // CAPTURE
        let raw_link = self.link_capturer.get_raw_link(&self.video_url).await?;
        let mut capturer = VideoCapture::from_file(&raw_link, opencv::videoio::CAP_FFMPEG)?;
        let mut full_image = Mat::default();
        let params = Vector::new();
        capturer
            .read(&mut full_image)?
            .then_some(())
            .ok_or(ImageProcessingError::ImageReadError)?;
        #[cfg(debug_assertions)]
        opencv::imgcodecs::imwrite("debug_full.jpg", &full_image, &params)?
            .then_some(())
            .ok_or(ImageProcessingError::ImageWriteError)?;
        // CROP
        let image_dimensions = full_image.mat_size();
        (image_dimensions.len() == 2)
            .then_some(())
            .ok_or(ImageProcessingError::ImageDimensionsError)?;
        let roi = Rect_::new(0, 0, image_dimensions[1], image_dimensions[0] / 10);
        let cropped_image = Mat::roi(&full_image, roi)?;
        #[cfg(debug_assertions)]
        opencv::imgcodecs::imwrite("debug_cropped.jpg", &cropped_image, &params)?
            .then_some(())
            .ok_or(ImageProcessingError::ImageWriteError)?;
        // MASK
        let mut masked_image = Mat::default();
        opencv::core::in_range(
            &cropped_image,
            &*LOW_BOUNDS,
            &*HIGH_BOUNDS,
            &mut masked_image,
        )?;
        #[cfg(debug_assertions)]
        opencv::imgcodecs::imwrite("debug_masked.jpg", &masked_image, &params)?
            .then_some(())
            .ok_or(ImageProcessingError::ImageWriteError)?;
        // ENCODE
        let mut buf = Vector::new();
        opencv::imgcodecs::imencode(".jpg", &masked_image, &mut buf, &params)?
            .then_some(())
            .ok_or(ImageProcessingError::ImageEncodeError)?;
        // OCR
        self.ocr.set_image_from_mem(buf.as_slice())?;
        self.ocr.set_source_resolution(DPI);
        let ocr_text = self.ocr.get_utf8_text()?.trim().to_string();
        info!("Track read using Tesseract OCR: {}", ocr_text);
        let track = Track::from_ocr_text(&ocr_text)?;
        Ok(track)
    }
}

#[derive(Error, Debug)]
pub enum ImageProcessingError {
    #[error("Reading the frame has failed.")]
    ImageReadError,
    #[error("Writing the image for debug has failed.")]
    ImageWriteError,
    #[error("Encoding the image has failed.")]
    ImageEncodeError,
    #[error("There is a problem with the frame dimensions.")]
    ImageDimensionsError,
    #[error("Masking the frame has failed.")]
    ImageMaskError,
}
