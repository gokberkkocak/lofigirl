use actix_web::{web, App, HttpServer};
use actix_web::{HttpResponse, Result};
use lofigirl_shared::{track::Track, CHILL_API_END_POINT, SLEEP_API_END_POINT};
use parking_lot::Mutex;
use serde::Serialize;
use thiserror::Error;

pub struct AppState {
    pub main_track: Mutex<Option<Track>>,
    pub second_track: Mutex<Option<Track>>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            main_track: Mutex::new(None),
            second_track: Mutex::new(None),
        }
    }
}

async fn get_main(data: web::Data<AppState>) -> Result<HttpResponse> {
    let lock = data.main_track.lock();
    let track = lock.clone();
    if let Some(track) = track {
        Ok(HttpResponse::Ok().json(track))
    } else {
        Ok(HttpResponse::NotFound().json(WebTrackError::CannotGiveTrack))
    }
}

async fn get_second(data: web::Data<AppState>) -> Result<HttpResponse> {
    let lock = data.second_track.lock();
    let track = lock.clone();
    if let Some(track) = track {
        Ok(HttpResponse::Ok().json(track))
    } else {
        Ok(HttpResponse::NotFound().json(WebTrackError::CannotGiveTrack))
    }
}

pub struct LofiServer;

impl LofiServer {
    pub async fn start(data: web::Data<AppState>, port: u32) -> std::io::Result<()> {
        HttpServer::new(move || {
            // move counter into the closure
            App::new()
                // Note: using app_data instead of data
                .app_data(data.clone()) // <- register the created data
                .route(
                    &format!("/{}", CHILL_API_END_POINT),
                    web::get().to(get_main),
                )
                .route(
                    &format!("/{}", SLEEP_API_END_POINT),
                    web::get().to(get_second),
                )
        })
        .bind(format!("0.0.0.0:{}", port))?
        // .bind(format!("127.0.0.1:{}", port))?
        .run()
        .await
    }
}

#[derive(Error, Debug, Serialize)]
pub enum WebTrackError {
    #[error("OCR text cannot be split.")]
    CannotGiveTrack,
}
