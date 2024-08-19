use std::sync::Arc;
use std::time::Instant;

use actix_web::http::header::Header;
use actix_web::http::StatusCode;

use actix_web::{web, HttpRequest, Responder};
use actix_web::{HttpResponse, Result};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use actix_ws::Message;
use futures_util::StreamExt as _;
use lofigirl_shared_common::api::{
    ScrobbleRequest, SessionRequest, SessionResponse, TokenRequest, TokenResponse,
};
use lofigirl_shared_common::config::LastFMClientConfig;
use lofigirl_shared_common::jwt::JWTClaims;
use lofigirl_shared_common::track::Track;
use lofigirl_shared_common::{REGULAR_INTERVAL, SERVER_PING_TIMEOUT_INTERVAL};
use lofigirl_shared_listen::listener::Listener;
use parking_lot::RwLock;
use serde::Serialize;
use thiserror::Error;
use tracing::{info, warn};
use url::Url;

use crate::util::YoutubeIdExtractor;

use super::AppState;

pub(crate) async fn send(
    req: HttpRequest,
    data: web::Data<AppState>,
    info: web::Json<ScrobbleRequest>,
) -> Result<HttpResponse> {
    let auth = Authorization::<Bearer>::parse(&req)?;
    let bearer_token_jwt = auth.as_ref().token().to_owned();
    let token_data = JWTClaims::decode(bearer_token_jwt);
    info!("{:?}", token_data);
    let token_data = token_data
        .map_err(|e| actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    let token: String = token_data.encrypted_token.into();
    let info = info.into_inner();
    let mut listener = Listener::default();
    let (lfm, lb) = data
        .token_db
        .get_info_from_token(&token)
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
    let youtube_id = youtube_url
        .get_video_id()
        .ok_or(actix_web::error::InternalError::new(
            ServerResponseError::InvalidYoutubeLink,
            StatusCode::INTERNAL_SERVER_ERROR,
        ))?;
    // modify the last requested time
    // even with explicit drop, clippy still complains about it so wrap it in a block
    {
        let mut last_requested = data.last_requested.write();
        last_requested.insert(youtube_id.clone(), Instant::now());
    }

    // Check if there is a working image processor
    if let Some(track) = data.tracks.read().get(&youtube_id) {
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
                        let youtube_id = youtube_url.get_video_id();
                        match youtube_id {
                            Some(youtube_id) => {
                                let youtube_id_clone = youtube_id.clone();
                                actix_rt::spawn(async move {
                                    // periodic update
                                    loop {
                                        {
                                            let mut last_requested = state.last_requested.write();
                                            last_requested
                                                .insert(youtube_id_clone.clone(), Instant::now());
                                        }
                                        tokio::time::sleep(*REGULAR_INTERVAL).await;
                                    }
                                });
                                // Check if there is a worker already find its rx channel otherwise create worker and bring its rx channel
                                let state = data.clone();
                                let should_reuse =
                                    data.track_channels.read().contains_key(&youtube_id);
                                let mut rx = if should_reuse {
                                    info!("Found existing worker for given video, reuse worker");
                                    if let Some(rx) = data.track_channels.read().get(&youtube_id) {
                                        rx.clone()
                                    } else {
                                        warn!("reuse failure");
                                        break;
                                    }
                                } else {
                                    let (tx, rx) = tokio::sync::watch::channel(Track::default());
                                    let mut worker = match crate::worker::ServerWorker::new(
                                        youtube_url.clone(),
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
                                                .insert(youtube_id.clone(), rx.clone());
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
                                    loop {
                                        if rx.changed().await.is_err() {
                                            break;
                                        }
                                        let track = rx.borrow_and_update().clone();
                                        // if track is empty, we might have created the channel with default empty track
                                        // so skip this and wait for the next one
                                        if !track.is_empty() {
                                            let ser_track = serde_json::to_string(&track).unwrap();
                                            if session_clone.text(ser_track).await.is_err() {
                                                break;
                                            }
                                        }
                                    }
                                });

                                // if client does not ping for some time - close socket
                                let last_ping = last_ping.clone();
                                let session_clone = session.clone();
                                actix_rt::spawn(async move {
                                    loop {
                                        if last_ping.read().elapsed()
                                            > *SERVER_PING_TIMEOUT_INTERVAL
                                        {
                                            break;
                                        }
                                        tokio::time::sleep(*REGULAR_INTERVAL).await;
                                    }
                                    warn!("Did not receive ping from socket for a while, closing");
                                    let _ = session_clone.close(None).await;
                                });
                            }
                            None => {
                                warn!("Cannot parse url into youtube id");
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        warn!("Cannot parse the string into url");
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
            &LastFMClientConfig::PasswordAuth(
                lofigirl_shared_common::config::LastFMClientPasswordConfig {
                    username: info.username,
                    password: info.secure_password.into(),
                },
            ),
        )
        .map_err(|e| actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
        Ok(HttpResponse::Ok().json(SessionResponse {
            secure_session_key: session_config.session_key.into(),
        }))
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
            info.secure_lastfm_session_key.map(|s| s.into()).as_ref(),
            info.secure_listenbrainz_token.map(|s| s.into()).as_ref(),
        )
        .await
        .map_err(|e| actix_web::error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(HttpResponse::Ok().json(TokenResponse {
        token: token.into(),
    }))
}

pub(crate) async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

#[derive(Error, Debug, Serialize)]
pub enum ServerResponseError {
    #[error("LastFM API is not available")]
    APINotAvailable,
    #[error("Youtube link is not valid")]
    InvalidYoutubeLink,
}
