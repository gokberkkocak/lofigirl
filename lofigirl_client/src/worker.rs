use crate::config::Config;
use crate::config::TokenConfig;
use anyhow::Result;
use lofigirl_shared_common::api::Action;
use lofigirl_shared_common::api::ScrobbleRequest;
use lofigirl_shared_common::api::SessionRequest;
use lofigirl_shared_common::api::SessionResponse;
use lofigirl_shared_common::api::TokenRequest;
use lofigirl_shared_common::api::TokenResponse;
use lofigirl_shared_common::config::LastFMClientConfig;
use lofigirl_shared_common::config::LastFMClientPasswordConfig;
use lofigirl_shared_common::config::LastFMClientSessionConfig;
use lofigirl_shared_common::SEND_END_POINT;
use lofigirl_shared_common::SESSION_END_POINT;
use lofigirl_shared_common::TOKEN_END_POINT;
use lofigirl_shared_common::TRACK_END_POINT;
use lofigirl_shared_common::{config::ConfigError, track::Track};
#[cfg(not(feature = "standalone"))]
use lofigirl_shared_common::{CHILL_API_END_POINT, SLEEP_API_END_POINT};
#[cfg(feature = "standalone")]
use lofigirl_shared_listen::listener::Listener;
#[cfg(feature = "standalone")]
use lofigirl_sys::image::ImageProcessor;
#[cfg(feature = "notify")]
use notify_rust::Notification;
#[cfg(feature = "notify")]
use notify_rust::Timeout;
#[cfg(not(feature = "standalone"))]
use reqwest::Client;
use thiserror::Error;
#[cfg(feature = "standalone")]
use url::Url;

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
    pub async fn new(config: &mut Config, second: bool) -> Result<Worker> {
        #[cfg(feature = "standalone")]
        let video_url = if second {
            Url::parse(
                &config
                    .video
                    .as_ref()
                    .ok_or(ConfigError::EmptyVideoConfig)?
                    .second_link
                    .as_ref()
                    .ok_or(WorkerError::_MissingSecondVideoLink)?,
            )?
        } else {
            Url::parse(
                &config
                    .video
                    .as_ref()
                    .ok_or(ConfigError::EmptyVideoConfig)?
                    .link,
            )?
        };
        #[cfg(feature = "standalone")]
        let image_proc = ImageProcessor::new(video_url)?;
        #[cfg(not(feature = "standalone"))]
        let client = Client::new();
        let lastfm_session_config = if let Some(lastfm) = &config.lastfm {
            #[cfg(not(feature = "standalone"))]
            match &lastfm.client {
                lofigirl_shared_common::config::LastFMClientConfig::PasswordAuth(p) => Some(
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
                ),
                lofigirl_shared_common::config::LastFMClientConfig::SessionAuth(s) => {
                    Some(s.clone())
                }
            }
            #[cfg(feature = "standalone")]
            if let Some(api) = &lastfm.api {
                Some(Listener::convert_client_to_session(api, &lastfm.client)?)
            } else {
                None
            }
        } else {
            None
        };
        #[cfg(feature = "standalone")]
        let mut listener = Listener::new();
        #[cfg(feature = "standalone")]
        if let Some(lastfm) = &config.lastfm {
            if let Some(api) = &lastfm.api {
                if let Some(session) = lastfm_session_config {
                    listener.set_lastfm_listener(&api, &LastFMClientConfig::SessionAuth(session))?;
                }
            }
        }
        #[cfg(feature = "standalone")]
        if let Some(listenbrainz) = &config.listenbrainz {
            listener.set_listenbrainz_listener(listenbrainz)?;
        }
        #[cfg(not(feature = "standalone"))]
        let base_url = config
            .server
            .as_ref()
            .ok_or(ConfigError::EmptyServerConfig)?
            .link
            .as_str();
        #[cfg(not(feature = "standalone"))]
        let track_request_url = format!(
            "{}{}/{}",
            base_url,
            TRACK_END_POINT,
            if second {
                &SLEEP_API_END_POINT
            } else {
                &CHILL_API_END_POINT
            }
        );
        #[cfg(not(feature = "standalone"))]
        let track_send_url = format!("{}{}", base_url, SEND_END_POINT);
        #[cfg(not(feature = "standalone"))]
        let lastfm_session_key = if let Some(s) = lastfm_session_config {
            if let Some(l) = &mut config.lastfm {
                l.client = LastFMClientConfig::SessionAuth(s.to_owned());
            }
            Some(s.session_key)
        } else {
            None
        };
        #[cfg(not(feature = "standalone"))]
        let listenbrainz_token = if let Some(l) = &config.listenbrainz {
            Some(l.token.to_owned())
        } else {
            None
        };
        #[cfg(not(feature = "standalone"))]
        let token =
            Worker::get_token(&client, lastfm_session_key, listenbrainz_token, base_url).await?;
        #[cfg(not(feature = "standalone"))]
        {
            config.session = Some(TokenConfig {
                token: token.clone(),
            });
        }
        Ok(Worker {
            #[cfg(feature = "standalone")]
            listener,
            #[cfg(feature = "standalone")]
            image_proc,
            #[cfg(not(feature = "standalone"))]
            client,
            #[cfg(not(feature = "standalone"))]
            track_request_url,
            #[cfg(not(feature = "standalone"))]
            track_send_url,
            #[cfg(not(feature = "standalone"))]
            token,
            prev_track_with_count: None,
        })
    }

    #[cfg(not(feature = "standalone"))]
    pub async fn get_track(&self) -> Result<Track> {
        let response = self.client.get(&self.track_request_url).send().await?;
        let content = response.text().await?;
        let track: Track = serde_json::from_str(&content)?;
        Ok(track)
    }

    #[cfg(not(feature = "standalone"))]
    async fn get_session(
        client: &Client,
        password_config: &LastFMClientPasswordConfig,
        base_url: &str,
    ) -> Result<LastFMClientSessionConfig> {
        let session_response = client
            .post(&format!("{}{}", base_url, SESSION_END_POINT))
            .json(&SessionRequest {
                password_config: password_config.clone(),
            })
            .send()
            .await?
            .json::<SessionResponse>()
            .await?;
        Ok(session_response.session_config)
    }

    #[cfg(not(feature = "standalone"))]
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

    pub async fn work(&mut self) -> bool {
        match self.fragile_work().await {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Problem with: {}", e.to_string());
                false
            }
        }
    }

    #[cfg(not(feature = "standalone"))]
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

struct TrackWithCount {
    pub track: Track,
    pub count: usize,
}

impl TrackWithCount {
    fn new(track: Track) -> TrackWithCount {
        TrackWithCount { track, count: 1 }
    }
}

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Second video link cannot be found on config file.")]
    _MissingSecondVideoLink,
}
