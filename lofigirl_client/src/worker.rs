use crate::config::Config;
use anyhow::Result;

use lofigirl_shared_common::config::LastFMClientConfig;

use lofigirl_shared_common::track::Track;

use url::Url;

#[cfg(not(feature = "standalone"))]
use {
    crate::config::TokenConfig,
    anyhow::bail,
    futures_util::{SinkExt as _, StreamExt as _, TryStreamExt as _},
    lofigirl_shared_common::api::Action,
    lofigirl_shared_common::api::ScrobbleRequest,
    lofigirl_shared_common::api::SessionRequest,
    lofigirl_shared_common::api::SessionResponse,
    lofigirl_shared_common::api::TokenRequest,
    lofigirl_shared_common::api::TokenResponse,
    lofigirl_shared_common::config::ConfigError,
    lofigirl_shared_common::config::LastFMClientPasswordConfig,
    lofigirl_shared_common::config::LastFMClientSessionConfig,
    lofigirl_shared_common::jwt::JWTClaims,
    lofigirl_shared_common::LASTFM_SESSION_END_POINT,
    lofigirl_shared_common::SEND_END_POINT,
    lofigirl_shared_common::TOKEN_END_POINT,
    lofigirl_shared_common::{CLIENT_PING_INTERVAL, TRACK_SOCKET_END_POINT},
    reqwest::Client,
    reqwest_websocket::{Message, RequestBuilderExt},
    tracing::info,
};

#[cfg(feature = "standalone")]
use {
    lofigirl_shared_common::{FAST_TRY_INTERVAL, REGULAR_INTERVAL},
    lofigirl_shared_listen::listener::Listener,
    lofigirl_sys::image::ImageProcessor,
    tracing::warn,
};
#[cfg(feature = "notify")]
use {notify_rust::Notification, notify_rust::Timeout};

#[cfg(not(feature = "standalone"))]
pub struct Worker {
    client: Client,
    requested_url: String,
    track_send_url: String,
    track_socket_url: String,
    token: String,
}

#[cfg(feature = "standalone")]
pub struct Worker {
    listener: Listener,
    url: Url,
}

impl Worker {
    pub async fn work(&mut self) -> anyhow::Result<()> {
        self.work_with_connection().await?;
        Ok(())
    }

    async fn send_listen(&self, track: &Track) -> Result<()> {
        #[cfg(feature = "notify")]
        Notification::new()
            .summary("Scrobbled")
            .body(&format!("{} - {}", &track.artist, &track.song))
            .appname("lofigirl")
            .timeout(Timeout::Milliseconds(6000))
            .show()?;
        #[cfg(feature = "standalone")]
        self.listener.send_listen(track)?;
        #[cfg(not(feature = "standalone"))]
        self.post_track(track, Action::Listened).await?;
        Ok(())
    }

    async fn send_now_playing(&self, track: &Track) -> Result<()> {
        #[cfg(feature = "notify")]
        Notification::new()
            .summary("Now playing")
            .body(&format!("{} - {}", &track.artist, &track.song))
            .appname("lofigirl")
            .timeout(Timeout::Milliseconds(6000))
            .show()?;
        #[cfg(feature = "standalone")]
        self.listener.send_now_playing(track)?;
        #[cfg(not(feature = "standalone"))]
        self.post_track(track, Action::PlayingNow).await?;
        Ok(())
    }
}

#[cfg(feature = "standalone")]
impl Worker {
    pub async fn new(config: &mut Config, url: Url) -> Result<(Worker, bool)> {
        let mut config_changed = false;
        let lastfm_session_config = if let Some(client) = &config.lastfm {
            if let Some(api) = &config.lastfm_api {
                config_changed = true;
                Some(Listener::convert_client_to_session(api, client)?)
            } else {
                None
            }
        } else {
            None
        };
        let mut listener = Listener::default();
        if let Some(session) = lastfm_session_config {
            config.lastfm = Some(LastFMClientConfig::SessionAuth(session.to_owned()));
            if let Some(api) = &config.lastfm_api {
                listener.set_lastfm_listener(api, &LastFMClientConfig::SessionAuth(session))?;
            }
        }
        if let Some(listenbrainz) = &config.listenbrainz {
            listener.set_listenbrainz_listener(listenbrainz)?;
        }
        Ok((Worker { listener, url }, config_changed))
    }

    async fn work_with_connection(&self) -> anyhow::Result<()> {
        let mut image_proc = ImageProcessor::new(self.url.clone())?;
        let mut current_track: Track = Track::default();
        loop {
            match image_proc.next_track().await {
                Ok(next_track) => {
                    if current_track != next_track {
                        if !current_track.is_empty() {
                            self.send_listen(&current_track).await?;
                        }
                        self.send_now_playing(&next_track).await?;
                        current_track = next_track;
                    }
                    tokio::time::sleep(*REGULAR_INTERVAL).await;
                }
                Err(e) => {
                    warn!("Problem with: {}", e);
                    tokio::time::sleep(*FAST_TRY_INTERVAL).await;
                }
            }
        }
    }
}

#[cfg(not(feature = "standalone"))]
impl Worker {
    pub async fn new(config: &mut Config, requested_url: Url) -> Result<(Worker, bool)> {
        let mut config_changed = false;
        let client = Client::new();
        let base_url = config
            .server
            .as_ref()
            .ok_or(ConfigError::EmptyServerConfig)?
            .link
            .as_str()
            .to_owned();

        let token = Worker::get_token(config, &client, &base_url, &mut config_changed).await?;
        let track_send_url = format!("{}{}", base_url, SEND_END_POINT);
        info!("Client worker initialized");

        // ws socket url
        // parse base url to detect http or https
        let mut url = Url::parse(&base_url)?;
        if url.scheme() == "https" {
            url.set_scheme("wss")
                .map_err(|_| anyhow::Error::msg("url scheme error"))?;
        } else if url.scheme() == "http" {
            url.set_scheme("ws")
                .map_err(|_| anyhow::Error::msg("url scheme error"))?;
        } else {
            bail!("Cannot work on outside http/https")
        }
        let track_socket_url = url.join(TRACK_SOCKET_END_POINT)?.into();

        Ok((
            Worker {
                client,
                requested_url: requested_url.into(),
                track_send_url,
                track_socket_url,
                token,
            },
            config_changed,
        ))
    }

    async fn get_token(
        config: &mut Config,
        client: &Client,
        base_url: &str,
        config_changed: &mut bool,
    ) -> anyhow::Result<String> {
        let token_config = config.session.take();
        let token = match token_config {
            Some(token) => token.token,
            None => {
                let lastfm_session_config = if let Some(lastfm) = &config.lastfm {
                    match &lastfm {
                        lofigirl_shared_common::config::LastFMClientConfig::PasswordAuth(p) => {
                            Some(
                                Worker::get_session(
                                    client,
                                    p,
                                    config
                                        .server
                                        .as_ref()
                                        .ok_or(ConfigError::EmptyServerConfig)?
                                        .link
                                        .as_str(),
                                )
                                .await?,
                            )
                        }
                        lofigirl_shared_common::config::LastFMClientConfig::SessionAuth(s) => {
                            Some(s.clone())
                        }
                    }
                } else {
                    None
                };
                let lastfm_session_key = if let Some(s) = lastfm_session_config {
                    if let Some(l) = &mut config.lastfm {
                        *l = LastFMClientConfig::SessionAuth(s.to_owned());
                    }
                    Some(s.session_key)
                } else {
                    None
                };
                let listenbrainz_token = config.listenbrainz.as_ref().map(|l| l.token.to_owned());
                let token =
                    Worker::request_token(client, lastfm_session_key, listenbrainz_token, base_url)
                        .await?;
                *config_changed = true;
                config.session = Some(TokenConfig {
                    token: token.clone(),
                });
                token
            }
        };
        Ok(token)
    }

    async fn request_token(
        client: &Client,
        lastfm_session_key: Option<String>,
        listenbrainz_token: Option<String>,
        base_url: &str,
    ) -> Result<String> {
        let token_response = client
            .post(&format!("{}{}", base_url, TOKEN_END_POINT))
            .json(&TokenRequest {
                secure_lastfm_session_key: lastfm_session_key.map(|s| s.into()),
                secure_listenbrainz_token: listenbrainz_token.map(|s| s.into()),
            })
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;
        Ok(token_response.secure_token.into())
    }

    async fn post_track(&self, track: &Track, action: Action) -> Result<()> {
        let jwt_token = JWTClaims::encode(self.token.to_owned())?;
        self.client
            .post(&self.track_send_url)
            .header("Authorization", "Bearer ".to_owned() + &jwt_token)
            .json(&ScrobbleRequest {
                action,
                track: track.to_owned(),
            })
            .send()
            .await?;
        Ok(())
    }

    async fn get_session(
        client: &Client,
        password_config: &LastFMClientPasswordConfig,
        base_url: &str,
    ) -> Result<LastFMClientSessionConfig> {
        let session_response = client
            .post(&format!("{}{}", base_url, LASTFM_SESSION_END_POINT))
            .json(&SessionRequest {
                username: password_config.username.clone(),
                secure_password: password_config.password.clone().into(),
            })
            .send()
            .await?
            .json::<SessionResponse>()
            .await?;
        Ok(LastFMClientSessionConfig {
            session_key: session_response.secure_session_key.into(),
        })
    }

    async fn work_with_connection(&self) -> anyhow::Result<()> {
        let response = Client::default()
            .get(&self.track_socket_url)
            .upgrade()
            .send()
            .await?;
        let websocket = response.into_websocket().await?;

        let (mut tx, mut rx) = websocket.split();
        let mut current_track = Track::default();

        // Send initial Url message
        tx.send(Message::Text(self.requested_url.clone())).await?;

        // Setup periodic ping message
        tokio::spawn(async move {
            loop {
                if tx.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
                tokio::time::sleep(*CLIENT_PING_INTERVAL).await;
            }
        });

        while let Some(message) = rx.try_next().await? {
            if let Message::Text(text) = message {
                let next_track: Track = serde_json::from_str(&text)?;
                if !current_track.is_empty() {
                    info!("Sent listen for: \"{}\"", current_track);
                    self.send_listen(&current_track).await?;
                }
                info!("Sent now playing info for: \"{}\"", next_track);
                self.send_now_playing(&next_track).await?;
                current_track = next_track;
            }
        }
        Ok(())
    }
}
