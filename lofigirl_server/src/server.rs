use crate::session::TokenDB;
use actix_cors::Cors;
use actix_web::http::StatusCode;

use actix_web::{web, App, HttpServer};
use actix_web::{HttpResponse, Result};
use lofigirl_shared_common::api::{
    ScrobbleRequest, SessionRequest, SessionResponse, TokenRequest, TokenResponse,
};
use lofigirl_shared_common::config::{LastFMApiConfig, LastFMClientConfig};
use lofigirl_shared_common::{track::Track, CHILL_TRACK_API_END_POINT, SLEEP_TRACK_API_END_POINT};
use lofigirl_shared_common::{
    HEALTH_END_POINT, LASTFM_SESSION_END_POINT, SEND_END_POINT, TOKEN_END_POINT, TRACK_END_POINT,
};
use lofigirl_shared_listen::listener::Listener;
use parking_lot::Mutex;
use serde::Serialize;
use thiserror::Error;

pub struct AppState {
    pub lastfm_api: Mutex<Option<LastFMApiConfig>>,
    pub tracks: Mutex<Vec<Option<Track>>>,
    // pub main_track: Mutex<Option<Track>>,
    // pub second_track: Mutex<Option<Track>>,
    pub token_db: Mutex<TokenDB>,
}

impl AppState {
    pub async fn new(
        api: Option<LastFMApiConfig>,
        token_db_file: &str,
        nb_links: usize,
    ) -> anyhow::Result<AppState> {
        Ok(AppState {
            lastfm_api: Mutex::new(api),
            tracks: Mutex::new(vec![None; nb_links]),
            // main_track: Mutex::new(None),
            // second_track: Mutex::new(None),
            token_db: Mutex::new(TokenDB::new(token_db_file).await?),
        })
    }
}
async fn get_track_with_index(data: web::Data<AppState>, idx: usize) -> Result<HttpResponse> {
    let lock = data.tracks.lock();
    let tracks = lock.clone();
    if let Some(track) = tracks.get(idx) {
        Ok(HttpResponse::Ok().json(track))
    } else {
        Ok(HttpResponse::NotFound().json(ServerResponseError::TrackNotAvailable))
    }
}

async fn get_main(data: web::Data<AppState>) -> Result<HttpResponse> {
    get_track_with_index(data, 0).await
}

async fn get_second(data: web::Data<AppState>) -> Result<HttpResponse> {
    get_track_with_index(data, 1).await
}

async fn send(data: web::Data<AppState>, info: web::Json<ScrobbleRequest>) -> Result<HttpResponse> {
    let info = info.into_inner();
    let mut listener = Listener::new();
    let token_db = data.token_db.lock();
    let api = data.lastfm_api.lock();
    let (lfm, lb) = token_db
        .get_info_from_token(&info.token)
        .await
        .map_err(|e| actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    if let Some(lastfm_client_session) = lfm {
        if let Some(api) = &*api {
            listener
                .set_lastfm_listener(api, &LastFMClientConfig::SessionAuth(lastfm_client_session))
                .map_err(|e| {
                    actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
                })?;
        }
    }
    if let Some(listenbrainz) = lb {
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
        let session_config = Listener::convert_client_to_session(
            api,
            &LastFMClientConfig::PasswordAuth(info.password_config),
        )
        .map_err(|e| actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
        Ok(HttpResponse::Ok().json(SessionResponse { session_config }))
    } else {
        Ok(HttpResponse::NotFound().json(ServerResponseError::APINotAvailable))
    }
}

async fn token(data: web::Data<AppState>, info: web::Json<TokenRequest>) -> Result<HttpResponse> {
    let info = info.into_inner();
    let token_db = data.token_db.lock();
    let token = token_db
        .get_or_generate_token(
            info.lastfm_session_key.as_ref(),
            info.listenbrainz_token.as_ref(),
        )
        .await
        .map_err(|e| actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(HttpResponse::Ok().json(TokenResponse { token }))
}

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
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
                    &format!("{}{}", TRACK_END_POINT, CHILL_TRACK_API_END_POINT),
                    web::get().to(get_main),
                )
                .route(
                    &format!("{}{}", TRACK_END_POINT, SLEEP_TRACK_API_END_POINT),
                    web::get().to(get_second),
                )
                .route(SEND_END_POINT, web::post().to(send))
                .route(LASTFM_SESSION_END_POINT, web::post().to(session))
                .route(TOKEN_END_POINT, web::post().to(token))
                .route(HEALTH_END_POINT, web::get().to(health))
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
