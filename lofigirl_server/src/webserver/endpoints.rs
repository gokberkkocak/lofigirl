use std::sync::Arc;
use std::time::Instant;

use actix_web::http::StatusCode;

use actix_web::{web, HttpRequest, Responder};
use actix_web::{HttpResponse, Result};
use actix_ws::Message;
use futures_util::StreamExt as _;
use lofigirl_shared_common::api::{
    ScrobbleRequest, SessionRequest, SessionResponse, TokenRequest, TokenResponse,
};
use lofigirl_shared_common::config::LastFMClientConfig;
use lofigirl_shared_common::track::Track;
use lofigirl_shared_common::{REGULAR_INTERVAL, SERVER_PING_TIMEOUT_INTERVAL};
use lofigirl_shared_listen::listener::Listener;
use parking_lot::RwLock;
use serde::Serialize;
use thiserror::Error;
use tracing::{info, warn};
use url::Url;

use super::AppState;

pub(crate) async fn get_track_with_index(
    data: web::Data<AppState>,
    idx: usize,
) -> Result<HttpResponse> {
    let lock = data.seq_tracks.read();
    if let Some(track) = lock.get(idx) {
        Ok(HttpResponse::Ok().json(track))
    } else {
        Ok(HttpResponse::NotFound().json(ServerResponseError::TrackNotAvailable))
    }
}

pub(crate) async fn get_main(data: web::Data<AppState>) -> Result<HttpResponse> {
    get_track_with_index(data, 0).await
}

pub(crate) async fn get_second(data: web::Data<AppState>) -> Result<HttpResponse> {
    get_track_with_index(data, 1).await
}

pub(crate) async fn send(
    data: web::Data<AppState>,
    info: web::Json<ScrobbleRequest>,
) -> Result<HttpResponse> {
    let info = info.into_inner();
    let mut listener = Listener::default();
    let (lfm, lb) = data
        .token_db
        .get_info_from_token(&info.token)
        .await
        .map_err(|e| actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    if let Some(lastfm_client_session) = lfm {
        if let Some(api) = &data.lastfm_api {
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

pub(crate) async fn dynamic_track(
    data: web::Data<AppState>,
    url: web::Path<String>,
) -> Result<HttpResponse> {
    let youtube_url_string = url.into_inner();
    let youtube_url = Url::parse(&youtube_url_string)
        .map_err(|e| actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    // modify the last requested time
    // even with explicit drop, clippy still complains about it so wrap it in a block
    {
        let mut last_requested = data.last_requested.write();
        last_requested.insert(youtube_url.clone(), Instant::now());
    }

    // Check if there is a working image processor
    if let Some(track) = data.tracks.read().get(&youtube_url) {
        // return track
        return Ok(HttpResponse::Ok().json(track));
    }
    // Create the new worker and send accepted response
    let state = data.clone();
    // Rest API cannot use event based two-way communication, so we ignore tx,rx but we create it anyway for future connections
    let (tx, _rx) = tokio::sync::watch::channel(Track::default());
    let mut worker = crate::worker::ServerWorker::new(youtube_url, state.clone())
        .map_err(|e| actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    worker
        .work(tx)
        .await
        .map_err(|e| actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(HttpResponse::Accepted().finish())
}

pub(crate) async fn track_socket(
    data: web::Data<AppState>,
    req: HttpRequest,
    body: web::Payload,
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
    actix_web::rt::spawn(async move {
        let last_ping = Arc::new(RwLock::new(Instant::now()));
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(msg) => match Url::parse(&msg) {
                    Ok(youtube_url) => {
                        info!("Server received {youtube_url} from socket");
                        // Update last requested state
                        let state = data.clone();
                        let youtube_url_clone = youtube_url.clone();
                        actix_rt::spawn(async move {
                            // periodic update
                            loop {
                                {
                                    let mut last_requested = state.last_requested.write();
                                    last_requested
                                        .insert(youtube_url_clone.clone(), Instant::now());
                                }
                                tokio::time::sleep(*REGULAR_INTERVAL).await;
                            }
                        });
                        // Check if there is a worker already find its rx channel otherwise create worker and bring its rx channel
                        let youtube = youtube_url.clone();
                        let state = data.clone();
                        let should_reuse = data.track_channels.read().contains_key(&youtube_url);
                        let mut rx = if should_reuse {
                            if let Some(rx) = data.track_channels.read().get(&youtube_url) {
                                rx.clone()
                            } else {
                                warn!("reuse failure");
                                break;
                            }
                        } else {
                            let (tx, rx) = tokio::sync::watch::channel(Track::default());
                            let mut worker = match crate::worker::ServerWorker::new(
                                youtube.clone(),
                                state.clone(),
                            ) {
                                Ok(worker) => worker,
                                Err(_) => {
                                    warn!("creation failure");
                                    break;
                                }
                            };
                            match worker.work(tx).await {
                                Ok(_) => {
                                    data.track_channels
                                        .write()
                                        .insert(youtube.clone(), rx.clone());
                                    rx
                                }
                                Err(_) => {
                                    warn!("work failure");
                                    break;
                                }
                            }
                        };

                        // send new message on channel update
                        let mut session_clone = session.clone();
                        actix_rt::spawn(async move {
                            // discard first empty value
                            rx.borrow_and_update();
                            loop {
                                if rx.changed().await.is_err() {
                                    break;
                                }
                                let track = rx.borrow_and_update().clone();
                                let ser_track = serde_json::to_string(&track).unwrap();
                                if session_clone.text(ser_track).await.is_err() {
                                    break;
                                }
                            }
                        });

                        // if client does not ping for some time - close socket
                        let last_ping = last_ping.clone();
                        let session_clone = session.clone();
                        actix_rt::spawn(async move {
                            loop {
                                if last_ping.read().elapsed() > *SERVER_PING_TIMEOUT_INTERVAL {
                                    break;
                                }
                                tokio::time::sleep(*REGULAR_INTERVAL).await;
                            }
                            warn!("Did not receive ping from socket for a while, closing");
                            let _ = session_clone.close(None).await;
                        });
                    }
                    Err(_) => {
                        warn!("warning err");
                        break;
                    }
                },
                Message::Ping(bytes) | Message::Binary(bytes) => {
                    info!("Server received ping from socket");
                    // client ping updates last ping and also last requested for server worker
                    *last_ping.write() = Instant::now();
                    session.pong(&bytes).await.unwrap();
                }
                _ => break,
            }
        }
        let _ = session.close(None).await;
    });

    Ok(response)
}

pub(crate) async fn session(
    data: web::Data<AppState>,
    info: web::Json<SessionRequest>,
) -> Result<HttpResponse> {
    let info = info.into_inner();
    if let Some(api) = &data.lastfm_api {
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

pub(crate) async fn token(
    data: web::Data<AppState>,
    info: web::Json<TokenRequest>,
) -> Result<HttpResponse> {
    let info = info.into_inner();
    let token = data
        .token_db
        .get_or_generate_token(
            info.lastfm_session_key.as_ref(),
            info.listenbrainz_token.as_ref(),
        )
        .await
        .map_err(|e| actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(HttpResponse::Ok().json(TokenResponse { token }))
}

pub(crate) async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

#[derive(Error, Debug, Serialize)]
pub enum ServerResponseError {
    #[error("Track not available.")]
    TrackNotAvailable,
    #[error("LastFM API is not available")]
    APINotAvailable,
}
