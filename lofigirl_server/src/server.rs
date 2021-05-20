use actix_http::http::StatusCode;
use actix_web::{web, App, HttpServer};
use actix_web::{HttpResponse, Result};
use lofigirl_shared_listen::listener::Listener;
use lofigirl_shared_common::api::SendInfo;
use lofigirl_shared_common::{track::Track, CHILL_API_END_POINT, SLEEP_API_END_POINT};
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

async fn send(_data: web::Data<AppState>, info: web::Json<SendInfo>) -> Result<HttpResponse> {
    let info = info.into_inner();
    let mut listener = Listener::new();
    if let Some(lastfm) = info.lastfm {
        listener.set_lastfm_listener(&lastfm).map_err(|e| {
            actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
        })?;
    }
    if let Some(listenbrainz) = info.listenbrainz {
        listener
            .set_listenbrainz_listener(&listenbrainz)
            .map_err(|e| {
                actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
            })?;
    }
    match info.action {
        lofigirl_shared_common::api::Action::PlayingNow => {
            listener.send_now_playing(&info.track).map_err(|e| {
                actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        }
        lofigirl_shared_common::api::Action::Listened => {
            listener.send_listen(&info.track).map_err(|e| {
                actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        }
    }
    Ok(HttpResponse::Ok().finish())
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
                    &format!("/track/{}", CHILL_API_END_POINT),
                    web::get().to(get_main),
                )
                .route(
                    &format!("/track/{}", SLEEP_API_END_POINT),
                    web::get().to(get_second),
                )
                .route("/send", web::post().to(send))
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
