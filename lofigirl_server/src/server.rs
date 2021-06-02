use crate::session::TokenDB;
use actix_cors::Cors;
use actix_http::http::StatusCode;
use actix_web::{web, App, HttpServer};
use actix_web::{HttpResponse, Result};
use lofigirl_shared_common::api::{ScrobbleRequest, SessionRequest, SessionResponse, TokenRequest};
use lofigirl_shared_common::config::{LastFMApiConfig, LastFMClientConfig, LastFMConfig};
use lofigirl_shared_common::{track::Track, CHILL_API_END_POINT, SLEEP_API_END_POINT};
use lofigirl_shared_common::{SEND_END_POINT, SESSION_END_POINT, TRACK_END_POINT};
use lofigirl_shared_listen::listener::Listener;
use parking_lot::Mutex;
use serde::Serialize;
use thiserror::Error;

pub struct AppState {
    pub lastfm_api: Mutex<Option<LastFMApiConfig>>,
    pub main_track: Mutex<Option<Track>>,
    pub second_track: Mutex<Option<Track>>,
    pub token_db: Mutex<TokenDB>,
}

impl AppState {
    pub async fn new(api: Option<LastFMApiConfig>, token_db_file: &str) -> Result<AppState> {
        Ok(AppState {
            lastfm_api: Mutex::new(api),
            main_track: Mutex::new(None),
            second_track: Mutex::new(None),
            token_db: Mutex::new(TokenDB::new(token_db_file).await?),
        })
    }
}

async fn get_main(data: web::Data<AppState>) -> Result<HttpResponse> {
    let lock = data.main_track.lock();
    let track = lock.clone();
    if let Some(track) = track {
        Ok(HttpResponse::Ok().json(track))
    } else {
        Ok(HttpResponse::NotFound().json(ServerResponseError::TrackNotAvailable))
    }
}

async fn get_second(data: web::Data<AppState>) -> Result<HttpResponse> {
    let lock = data.second_track.lock();
    let track = lock.clone();
    if let Some(track) = track {
        Ok(HttpResponse::Ok().json(track))
    } else {
        Ok(HttpResponse::NotFound().json(ServerResponseError::TrackNotAvailable))
    }
}

async fn send(data: web::Data<AppState>, info: web::Json<ScrobbleRequest>) -> Result<HttpResponse> {
    let info = info.into_inner();
    let mut listener = Listener::new();
    let api = data.lastfm_api.lock();
    if let Some(lastfm_client_session) = info.lastfm {
        if let Some(api) = &*api {
            let l = LastFMConfig {
                client: LastFMClientConfig::SessionAuth(lastfm_client_session),
                api: Some(api.clone()),
            };
            listener.set_lastfm_listener(&l).map_err(|e| {
                actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        }
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

async fn session(
    data: web::Data<AppState>,
    info: web::Json<SessionRequest>,
) -> Result<HttpResponse> {
    let info = info.into_inner();
    let api = data.lastfm_api.lock();
    if let Some(api) = &*api {
        let l = LastFMConfig {
            client: LastFMClientConfig::PasswordAuth(info.password_config),
            api: Some(api.clone()),
        };
        let session_config = Listener::convert_client_to_session(&l).map_err(|e| {
            actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(HttpResponse::Ok().json(SessionResponse { session_config }))
    } else {
        Ok(HttpResponse::NotFound().json(ServerResponseError::APINotAvailable))
    }
}

async fn token(data: web::Data<AppState>, info: web::Json<TokenRequest>) {
    let info = info.into_inner();
}

pub struct LofiServer;

impl LofiServer {
    pub async fn start(data: web::Data<AppState>, port: u32) -> std::io::Result<()> {
        HttpServer::new(move || {
            // CORS is pretty relaxed, can change it in real production
            let cors = Cors::permissive();
            App::new()
                .wrap(cors)
                .app_data(data.clone()) // <- register the created data
                .route(
                    &format!("{}/{}", TRACK_END_POINT, CHILL_API_END_POINT),
                    web::get().to(get_main),
                )
                .route(
                    &format!("{}/{}", TRACK_END_POINT, SLEEP_API_END_POINT),
                    web::get().to(get_second),
                )
                .route(SEND_END_POINT, web::post().to(send))
                .route(SESSION_END_POINT, web::post().to(session))
        })
        .bind(format!("0.0.0.0:{}", port))?
        // .bind(format!("127.0.0.1:{}", port))?
        .run()
        .await
    }
}

#[derive(Error, Debug, Serialize)]
pub enum ServerResponseError {
    #[error("Track not available.")]
    TrackNotAvailable,
    #[error("LastFM API is not available")]
    APINotAvailable,
}
