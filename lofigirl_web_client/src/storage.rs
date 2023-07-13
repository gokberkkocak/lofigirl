use lofigirl_shared_common::config::LastFMClientSessionConfig;
use lofigirl_shared_common::config::ListenBrainzConfig;

use gloo_storage::LocalStorage;
use gloo_storage::Storage;
use gloo_storage::errors::StorageError;

const LASTFM_SESSION_KEY: &str = "lastfm_session_key";
const SESSION_TOKEN: &str = "session_token";
const LISTENBRAINZ_TOKEN: &str = "listenbrainz_token";
const SERVER_URL: &str = "server_url";

pub fn set_lastfm_config(lastfm: &LastFMClientSessionConfig) {
    LocalStorage::set(LASTFM_SESSION_KEY, &lastfm.session_key).unwrap();
}

pub fn get_lastfm_config() -> Option<LastFMClientSessionConfig> {
    match LocalStorage::get(LASTFM_SESSION_KEY) {
        Ok(session_key) => Some(LastFMClientSessionConfig { session_key }),
        Err(err) => match err {
            StorageError::KeyNotFound(_s) => None,
            other_error => panic!("{:?}", other_error),
        },
    }
}

pub fn remove_lastfm_config() {
    LocalStorage::delete(LASTFM_SESSION_KEY);
}

pub fn set_session_token(token: &str) {
    LocalStorage::set(SESSION_TOKEN, &token).unwrap();
}

pub fn get_session_token() -> Option<String> {
    match LocalStorage::get(SESSION_TOKEN) {
        Ok(token) => Some(token),
        Err(err) => match err {
            StorageError::KeyNotFound(_s) => None,
            other_error => panic!("{:?}", other_error),
        },
    }
}

pub fn remove_session_token() {
    LocalStorage::delete(SESSION_TOKEN);
}

pub fn set_listenbrainz_token(token: &str) {
    LocalStorage::set(LISTENBRAINZ_TOKEN, token).unwrap();
}

pub fn get_listenbrainz_token() -> Option<ListenBrainzConfig> {
    match LocalStorage::get(LISTENBRAINZ_TOKEN) {
        Ok(token) => Some(ListenBrainzConfig { token }),
        Err(err) => match err {
            StorageError::KeyNotFound(_s) => None,
            other_error => panic!("{:?}", other_error),
        },
    }
}

pub fn remove_listenbrainz_token() {
    LocalStorage::delete(LISTENBRAINZ_TOKEN);
}

pub fn set_server_url(url: &str) {
    LocalStorage::set(SERVER_URL, url).unwrap();
}

pub fn get_server_url() -> Option<String> {
    match LocalStorage::get(SERVER_URL) {
        Ok(url) => Some(url),
        Err(err) => match err {
            StorageError::KeyNotFound(_s) => None,
            other_error => panic!("{:?}", other_error),
        },
    }
}

pub fn remove_server_url() {
    LocalStorage::delete(SERVER_URL);
}
