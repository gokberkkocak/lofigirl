use std::fmt;

use crate::{
    config::{LastFMConfig, ListenBrainzConfig},
    track::Track,
};
use anyhow::Result;
use listenbrainz::ListenBrainz;
use rustfm_scrobble::{Scrobble, Scrobbler};

#[cfg(feature = "notify")]
use notify_rust::Notification;
#[cfg(feature = "notify")]
use notify_rust::Timeout;
#[derive(Default)]
pub struct Listener {
    lastfm_listener: Option<Scrobbler>,
    listenbrainz_listener: Option<ListenBrainz>,
}

impl Listener {
    pub fn new() -> Listener {
        Default::default()
    }

    pub fn set_lastfm_listener(&mut self, lastfm: &LastFMConfig) -> Result<()> {
        let mut lastfm_listener = Scrobbler::new(&lastfm.api_key, &lastfm.api_secret);
        lastfm_listener.authenticate_with_password(&lastfm.username, &lastfm.password)?;
        self.lastfm_listener = Some(lastfm_listener);
        Ok(())
    }

    pub fn set_listenbrainz_listener(&mut self, listenbrainz: &ListenBrainzConfig) -> Result<()> {
        let mut listenbrainz_listener = ListenBrainz::new();
        listenbrainz_listener.authenticate(&listenbrainz.token)?;
        self.listenbrainz_listener = Some(listenbrainz_listener);
        Ok(())
    }

    pub fn send_listen(&self, track: &Track) -> Result<()> {
        #[cfg(feature = "notify")]
        Notification::new()
            .summary("Scrobbled")
            .body(&format!("{} - {}", &track.artist, &track.song))
            .appname("lofigirl")
            .timeout(Timeout::Milliseconds(6000))
            .show()?;
        self.send_action(Action::Listened, track)
    }

    pub fn send_now_playing(&self, track: &Track) -> Result<()> {
        #[cfg(feature = "notify")]
        Notification::new()
            .summary("Now playing")
            .body(&format!("{} - {}", &track.artist, &track.song))
            .appname("lofigirl")
            .timeout(Timeout::Milliseconds(6000))
            .show()?;
        self.send_action(Action::PlayingNow, track)
    }

    fn send_action(&self, action: Action, track: &Track) -> Result<()> {
        if let Some(l) = &self.lastfm_listener {
            let scrobble = Scrobble::new(&track.artist, &track.song, "");
            action.act_for_lastfm(&l, &scrobble)?;
        }
        if let Some(l) = &self.listenbrainz_listener {
            action.act_for_listenbrainz(&l, track)?;
        }
        println!("Track \"{}\" has been marked: {}", track, action);
        Ok(())
    }
}

enum Action {
    Listened,
    PlayingNow,
}

impl Action {
    fn act_for_lastfm(&self, listener: &Scrobbler, scrobble: &Scrobble) -> Result<()> {
        match self {
            Action::Listened => {
                let _r = listener.scrobble(scrobble)?;
            }
            Action::PlayingNow => {
                let _r = listener.now_playing(scrobble)?;
            }
        }
        Ok(())
    }

    fn act_for_listenbrainz(&self, listener: &ListenBrainz, track: &Track) -> Result<()> {
        match self {
            Action::Listened => listener.listen(&track.artist, &track.song, "")?,
            Action::PlayingNow => listener.playing_now(&track.artist, &track.song, "")?,
        }
        Ok(())
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Listened => write!(f, "Listened"),
            Action::PlayingNow => write!(f, "Playing Now"),
        }
    }
}