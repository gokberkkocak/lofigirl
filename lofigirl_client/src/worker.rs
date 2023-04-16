use crate::config::Config;
use anyhow::Result;
use lofigirl_shared_common::config::LastFMClientConfig;
use lofigirl_shared_common::{config::ConfigError, track::Track};
use tracing::info;

#[cfg(not(feature = "standalone"))]
use {
    crate::config::TokenConfig,
    lofigirl_shared_common::api::Action,
    lofigirl_shared_common::api::ScrobbleRequest,
    lofigirl_shared_common::api::SessionRequest,
    lofigirl_shared_common::api::SessionResponse,
    lofigirl_shared_common::api::TokenRequest,
    lofigirl_shared_common::api::TokenResponse,
    lofigirl_shared_common::config::LastFMClientPasswordConfig,
    lofigirl_shared_common::config::LastFMClientSessionConfig,
    lofigirl_shared_common::LASTFM_SESSION_END_POINT,
    lofigirl_shared_common::SEND_END_POINT,
    lofigirl_shared_common::TOKEN_END_POINT,
    lofigirl_shared_common::TRACK_END_POINT,
    lofigirl_shared_common::{CHILL_TRACK_API_END_POINT, SLEEP_TRACK_API_END_POINT},
    reqwest::Client,
};

#[cfg(feature = "standalone")]
use {
    lofigirl_shared_listen::listener::Listener, lofigirl_sys::image::ImageProcessor,
    thiserror::Error, url::Url,
};
#[cfg(feature = "notify")]
use {notify_rust::Notification, notify_rust::Timeout};

#[cfg(not(feature = "standalone"))]
pub struct Worker {
    client: Client,
    track_request_url: String,
    track_send_url: String,
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
    pub async fn work(&mut self) -> bool {
        match self.fragile_work().await {
            Ok(_) => true,
            Err(e) => {
                info!("Problem with: {}", e.to_string());
                false
            }
        }
    }

    async fn fragile_work(&mut self) -> Result<()> {
        #[cfg(not(feature = "standalone"))]
        let next_track = self.get_track().await?;
        #[cfg(feature = "standalone")]
        let next_track = self.image_proc.next_track().await?;
        let prev = self.prev_track_with_count.take();
        match prev {
            Some(mut t) if t.track == next_track && t.count == 3 => {
                self.send_listen(&t.track).await?;
                t.count += 1;
                self.prev_track_with_count = Some(t);
            }
            Some(mut t) if t.track == next_track => {
                t.count += 1;
                self.prev_track_with_count = Some(t);
            }
            Some(t) if t.count < 3 => {
                self.send_listen(&t.track).await?;
                self.send_now_playing(&next_track).await?;
                self.prev_track_with_count = Some(TrackWithCount::new(next_track));
            }
            _ => {
                self.send_now_playing(&next_track).await?;
                self.prev_track_with_count = Some(TrackWithCount::new(next_track));
            }
        }
        Ok(())
    }

    async fn send_listen(&mut self, track: &Track) -> Result<()> {
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

    async fn send_now_playing(&mut self, track: &Track) -> Result<()> {
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
    pub async fn new(config: &mut Config, second: bool) -> Result<(Worker, bool)> {
        let mut config_changed = false;
        let video_url = if second {
            Url::parse(
                &config
                    .video
                    .as_ref()
                    .ok_or(ConfigError::EmptyVideoConfig)?
                    .links
                    .get(1)
                    .as_ref()
                    .ok_or(WorkerError::MissingSecondVideoLink)?,
            )?
        } else {
            Url::parse(
                &config
                    .video
                    .as_ref()
                    .ok_or(ConfigError::EmptyVideoConfig)?
                    .links
                    .get(0)
                    .ok_or(ConfigError::EmptyVideoConfig)?,
            )?
        };
        let image_proc = ImageProcessor::new(video_url)?;
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
        let mut listener = Listener::new();
        if let Some(session) = lastfm_session_config {
            config.lastfm = Some(LastFMClientConfig::SessionAuth(session.to_owned()));
            if let Some(api) = &config.lastfm_api {
                listener.set_lastfm_listener(&api, &LastFMClientConfig::SessionAuth(session))?;
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
    pub async fn new(config: &mut Config, second: bool) -> Result<(Worker, bool)> {
        let mut config_changed = false;
        let client = Client::new();
        let base_url = config
            .server
            .as_ref()
            .ok_or(ConfigError::EmptyServerConfig)?
            .link
            .as_str();
        let token_config = config.session.take();
        let token = match token_config {
            Some(token) => token.token,
            None => {
                let lastfm_session_config = if let Some(lastfm) = &config.lastfm {
                    match &lastfm {
                        lofigirl_shared_common::config::LastFMClientConfig::PasswordAuth(p) => {
                            config_changed = config_changed || true;
                            Some(
                                Worker::get_session(
                                    &client,
                                    p,
                                    &config
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
                let listenbrainz_token = if let Some(l) = &config.listenbrainz {
                    Some(l.token.to_owned())
                } else {
                    None
                };
                let token =
                    Worker::get_token(&client, lastfm_session_key, listenbrainz_token, base_url)
                        .await?;
                {
                    config_changed = config_changed || true;
                    config.session = Some(TokenConfig {
                        token: token.clone(),
                    });
                }
                token
            }
        };
        let track_request_url = format!(
            "{}{}{}",
            base_url,
            TRACK_END_POINT,
            if second {
                &SLEEP_TRACK_API_END_POINT
            } else {
                &CHILL_TRACK_API_END_POINT
            }
        );
        let track_send_url = format!("{}{}", base_url, SEND_END_POINT);
        info!("Client worker initialized");
        Ok((
            Worker {
                client,
                track_request_url,
                track_send_url,
                token,
                prev_track_with_count: None,
            },
            config_changed,
        ))
    }

    async fn get_token(
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

#[cfg(feature = "standalone")]
#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Second video link cannot be found on config file.")]
    MissingSecondVideoLink,
}
