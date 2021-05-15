use actix_web::{App, HttpServer, web};
use actix_web::{HttpResponse, Result};
use lofigirl_shared::track::Track;
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
    if let Some(track) = track{       
        Ok(HttpResponse::Ok().json(track))
    }
    else {
        Ok(HttpResponse::NotFound().json(WebTrackError::CannotGiveTrack))
    }
}

async fn get_second(data: web::Data<AppState>) -> Result<HttpResponse> {
    let lock = data.second_track.lock();
    let track = lock.clone();
    if let Some(track) = track{       
        Ok(HttpResponse::Ok().json(track))
    }
    else {
        Ok(HttpResponse::NotFound().json(WebTrackError::CannotGiveTrack))
    }
}

pub struct LofiServer;

impl LofiServer {

    pub async fn start(data: web::Data<AppState>) -> std::io::Result<()> {
        HttpServer::new(move || {
            // move counter into the closure
            App::new()
                // Note: using app_data instead of data
                .app_data(data.clone()) // <- register the created data
                .route("/chill", web::get().to(get_main))
                .route("/sleep", web::get().to(get_second))
        })
        .bind("0.0.0.0:8080")?
        // .bind("127.0.0.1:8080")?
        .run()
        .await
    }
}


#[derive(Error, Debug, Serialize)]
pub enum WebTrackError {
    #[error("OCR text cannot be split.")]
    CannotGiveTrack,
}
