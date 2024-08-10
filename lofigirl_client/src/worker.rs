use crate::config::Config;
use anyhow::Result;

use futures_util::{SinkExt as _, StreamExt as _, TryStreamExt as _};
use lofigirl_shared_common::config::LastFMClientConfig;
use lofigirl_shared_common::track::Track;
use lofigirl_shared_common::{FAST_TRY_INTERVAL, REGULAR_INTERVAL, TRACK_SOCKET_END_POINT};

use reqwest_websocket::{Message, RequestBuilderExt};
use tracing::{debug, info, warn};
use url::Url;

#[cfg(not(feature = "standalone"))]
use {
    crate::config::TokenConfig, anyhow::bail, lofigirl_shared_common::api::Action,
    lofigirl_shared_common::api::ScrobbleRequest, lofigirl_shared_common::api::SessionRequest,
    lofigirl_shared_common::api::SessionResponse, lofigirl_shared_common::api::TokenRequest,
    lofigirl_shared_common::api::TokenResponse, lofigirl_shared_common::config::ConfigError,
    lofigirl_shared_common::config::LastFMClientPasswordConfig,
    lofigirl_shared_common::config::LastFMClientSessionConfig,
    lofigirl_shared_common::LASTFM_SESSION_END_POINT, lofigirl_shared_common::SEND_END_POINT,
    lofigirl_shared_common::TOKEN_END_POINT, lofigirl_shared_common::TRACK_END_POINT,
    reqwest::Client,
};

#[cfg(feature = "standalone")]
use {lofigirl_shared_listen::listener::Listener, lofigirl_sys::image::ImageProcessor};
#[cfg(feature = "notify")]
use {notify_rust::Notification, notify_rust::Timeout};

#[cfg(not(feature = "standalone"))]
pub struct Worker {
    client: Client,
    requested_url: String,
    track_request_url: String,
    track_send_url: String,
    track_socket_url: String,
    token: String,
    prev_track_with_count: Option<TrackWithCount>,
}

#[cfg(feature = "standalone")]
pub struct Worker {
    listener: Listener,
    image_proc: ImageProcessor,
    prev_track_with_count: Option<TrackWithCount>,
}

impl Worker {
    pub async fn work(&mut self) {
        // loop {
        //     match self.periodic_task().await {
        //         Ok(_) => std::thread::sleep(*REGULAR_INTERVAL),
        //         Err(e) => {
        //             info!("Problem with: {}", e.to_string());
        //             std::thread::sleep(*FAST_TRY_INTERVAL);
        //         }
        //     }
        // }

        self.run_socket_connection().await.unwrap();
    }

    async fn periodic_task(&mut self) -> Result<()> {
        #[cfg(not(feature = "standalone"))]
        let next_track = self.get_track().await?;
        #[cfg(feature = "standalone")]
        let next_track = self.image_proc.next_track().await?;
        let prev = self.prev_track_with_count.take();
        match prev {
            Some(mut t) if t.track == next_track && t.count == 3 => {
                self.send_listen(&t.track).await?;
                #[cfg(not(feature = "standalone"))]
                info!("Sent listen for: \"{} - {}\"", t.track.artist, t.track.song);
                t.count += 1;
                self.prev_track_with_count = Some(t);
            }
            Some(mut t) if t.track == next_track => {
                t.count += 1;
                self.prev_track_with_count = Some(t);
            }
            Some(t) if t.count < 3 => {
                self.send_listen(&t.track).await?;
                #[cfg(not(feature = "standalone"))]
                info!(
                    "Sent listen info for: \"{} - {}\"",
                    t.track.artist, t.track.song
                );
                self.send_now_playing(&next_track).await?;
                #[cfg(not(feature = "standalone"))]
                info!(
                    "Sent now playing info for: \"{} - {}\"",
                    next_track.artist, next_track.song
                );
                self.prev_track_with_count = Some(TrackWithCount::new(next_track));
            }
            _ => {
                self.send_now_playing(&next_track).await?;
                #[cfg(not(feature = "standalone"))]
                info!(
                    "Sent now playing info for: \"{} - {}\"",
                    next_track.artist, next_track.song
                );
                self.prev_track_with_count = Some(TrackWithCount::new(next_track));
            }
        }
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
    pub async fn new(config: &mut Config, requested_url: Url) -> Result<(Worker, bool)> {
        let mut config_changed = false;
        let image_proc = ImageProcessor::new(requested_url)?;
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
        Ok((
            Worker {
                listener,
                image_proc,
                prev_track_with_count: None,
            },
            config_changed,
        ))
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
        let track_request_url = format!(
            "{}{}/{}",
            base_url,
            TRACK_END_POINT,
            percent_encoding::utf8_percent_encode(
                requested_url.as_str(),
                percent_encoding::NON_ALPHANUMERIC
            )
        );
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
                track_request_url,
                track_send_url,
                track_socket_url,
                token,
                prev_track_with_count: None,
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
                                    &client,
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
                let token = Worker::request_token(
                    &client,
                    lastfm_session_key,
                    listenbrainz_token,
                    base_url,
                )
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
                lastfm_session_key,
                listenbrainz_token,
            })
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;
        Ok(token_response.token)
    }

    async fn post_track(&self, track: &Track, action: Action) -> Result<()> {
        self.client
            .post(&self.track_send_url)
            .json(&ScrobbleRequest {
                action,
                track: track.to_owned(),
                token: self.token.to_owned(),
            })
            .send()
            .await?;
        Ok(())
    }

    pub async fn get_track(&self) -> Result<Track> {
        let response = self.client.get(&self.track_request_url).send().await?;
        // if response is 202 bail with different error
        if response.status().as_u16() == 202 {
            bail!("Track is not available yet. Will try again.");
        }
        let content = response.text().await?;
        let track: Track = serde_json::from_str(&content)?;
        Ok(track)
    }

    async fn get_session(
        client: &Client,
        password_config: &LastFMClientPasswordConfig,
        base_url: &str,
    ) -> Result<LastFMClientSessionConfig> {
        let session_response = client
            .post(&format!("{}{}", base_url, LASTFM_SESSION_END_POINT))
            .json(&SessionRequest {
                password_config: password_config.clone(),
            })
            .send()
            .await?
            .json::<SessionResponse>()
            .await?;
        Ok(session_response.session_config)
    }

    async fn run_socket_connection(&self) -> anyhow::Result<()> {
        let response = Client::default()
            .get(&self.track_socket_url)
            .upgrade()
            .send()
            .await?;
        let websocket = response.into_websocket().await?;

        let (mut tx, mut rx) = websocket.split();
        let mut current_track = Track::default();

        // Send initial Url message
        tx.send(Message::Text(self.requested_url.clone().into()))
            .await?;

        // Setup periodic ping message
        tokio::spawn(async move {
            loop {
                if tx.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
                tokio::time::sleep(*REGULAR_INTERVAL).await;
            }
        });

        while let Some(message) = rx.try_next().await? {
            match message {
                Message::Text(text) => {
                    let next_track: Track = serde_json::from_str(&text)?;
                    if !current_track.is_empty() {
                        info!("Sent listen for: \"{}\"", current_track);
                        self.send_listen(&current_track).await?;
                    }
                    info!("Sent now playing info for: \"{}\"", next_track);
                    self.send_now_playing(&next_track).await?;
                    current_track = next_track;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

struct TrackWithCount {
    pub track: Track,
    pub count: usize,
}

impl TrackWithCount {
    fn new(track: Track) -> TrackWithCount {
        TrackWithCount { track, count: 1 }
    }
}
