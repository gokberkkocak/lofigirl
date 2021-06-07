use lofigirl_shared_common::config::LastFMClientSessionConfig;
use lofigirl_shared_common::config::ListenBrainzConfig;
use seed::browser::web_storage::LocalStorage;
use seed::browser::web_storage::WebStorage;
use seed::browser::web_storage::WebStorageError;

const LASTFM_SESSION_KEY: &str = "lastfm_session_key";
const SESSION_TOKEN: &str = "session_token";
const LISTENBRAINZ_TOKEN: &str = "listenbrainz_token";
const SERVER_URL: &str = "server_url";

pub fn set_lastfm_config(lastfm: &LastFMClientSessionConfig) {
    LocalStorage::insert(LASTFM_SESSION_KEY, &lastfm.session_key).unwrap();
}

pub fn get_lastfm_config() -> Option<LastFMClientSessionConfig> {
    match LocalStorage::get(LASTFM_SESSION_KEY) {
        Ok(session_key) => Some(LastFMClientSessionConfig { session_key }),
        Err(err) => match err {
            WebStorageError::KeyNotFoundError => None,
            other_error => panic!("{:?}", other_error),
        },
    }
}

pub fn remove_lastfm_config() {
    LocalStorage::remove(LASTFM_SESSION_KEY).unwrap();
}

pub fn set_session_token(token: &str) {
    LocalStorage::insert(SESSION_TOKEN, &token).unwrap();
}

pub fn get_session_token() -> Option<String> {
    match LocalStorage::get(SESSION_TOKEN) {
        Ok(token) => Some(token),
        Err(err) => match err {
            WebStorageError::KeyNotFoundError => None,
            other_error => panic!("{:?}", other_error),
        },
    }
}

pub fn remove_session_token() {
    LocalStorage::remove(SESSION_TOKEN).unwrap();
}

pub fn set_listenbrainz_token(token: &str) {
    LocalStorage::insert(LISTENBRAINZ_TOKEN, token).unwrap();
}

pub fn get_listenbrainz_token() -> Option<ListenBrainzConfig> {
    match LocalStorage::get(LISTENBRAINZ_TOKEN) {
        Ok(token) => Some(ListenBrainzConfig { token }),
        Err(err) => match err {
            WebStorageError::KeyNotFoundError => None,
            other_error => panic!("{:?}", other_error),
        },
    }
}

pub fn remove_listenbrainz_token() {
    LocalStorage::remove(LISTENBRAINZ_TOKEN).unwrap();
}

pub fn set_server_url(url: &str) {
    LocalStorage::insert(SERVER_URL, url).unwrap();
}

pub fn get_server_url() -> Option<String> {
    match LocalStorage::get(SERVER_URL) {
        Ok(url) => Some(url),
        Err(err) => match err {
            WebStorageError::KeyNotFoundError => None,
            other_error => panic!("{:?}", other_error),
        },
    }
}

pub fn remove_server_url() {
    LocalStorage::remove(SERVER_URL).unwrap();
}
