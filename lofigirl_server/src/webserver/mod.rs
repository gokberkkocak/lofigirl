mod endpoints;

use std::collections::HashMap;
use std::time::Instant;

use crate::session::TokenDB;
use actix_cors::Cors;

use actix_web::{web, App, HttpServer};
use endpoints::{dynamic_track, health, send, session, token, track_socket};
use lofigirl_shared_common::config::LastFMApiConfig;
use lofigirl_shared_common::track::Track;
use lofigirl_shared_common::{
    HEALTH_END_POINT, LASTFM_SESSION_END_POINT, SEND_END_POINT, TOKEN_END_POINT, TRACK_END_POINT,
    TRACK_SOCKET_END_POINT,
};
use parking_lot::RwLock;
use tokio::sync::watch::Receiver;

pub struct AppState {
    pub lastfm_api: Option<LastFMApiConfig>,
    pub tracks: RwLock<HashMap<String, Track>>,
    pub last_requested: RwLock<HashMap<String, Instant>>,
    pub track_channels: RwLock<HashMap<String, Receiver<Track>>>,
    pub token_db: TokenDB,
}

impl AppState {
    pub async fn new(
        api: Option<LastFMApiConfig>,
        token_db_file: &str,
    ) -> anyhow::Result<AppState> {
        Ok(AppState {
            lastfm_api: api,
            token_db: TokenDB::new(token_db_file).await?,
            tracks: RwLock::new(HashMap::new()),
            track_channels: RwLock::new(HashMap::new()),
            last_requested: RwLock::new(HashMap::new()),
        })
    }
}
pub struct LofiServer;

impl LofiServer {
    pub async fn start(data: web::Data<AppState>, port: u32) -> std::io::Result<()> {
        HttpServer::new(move || {
            // CORS is pretty relaxed, can change it in real production
            let cors = Cors::permissive();
            App::new()
                .wrap(cors)
                .app_data(data.clone())
                // dynamic track endpoint
                .route(
                    &format!("{}/{{url}}", TRACK_END_POINT),
                    web::get().to(dynamic_track),
                )
                // event based track socket endpoint
                .route(TRACK_SOCKET_END_POINT, web::get().to(track_socket))
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
