use lofigirl_shared_common::config::{LastFMConfig, ListenBrainzConfig};
use seed::browser::web_storage::LocalStorage;
use seed::browser::web_storage::WebStorage;
use seed::browser::web_storage::WebStorageError;

const LASTFM_API_KEY: &str = "lastfm_api_key";
const LASTFM_API_SECRET: &str = "lastfm_api_secret";
const LASTFM_USERNAME: &str = "lastfm_username";
const LASTFM_PASSWORD: &str = "lastfm_password";
const LISTENBRAINZ_TOKEN: &str = "listenbrainz_token";
const SERVER_URL: &str = "server_url";

pub fn set_lastfm_config(lastfm: &LastFMConfig) {
    LocalStorage::insert(LASTFM_API_KEY, &lastfm.api_key).unwrap();
    LocalStorage::insert(LASTFM_API_SECRET, &lastfm.api_secret).unwrap();
    LocalStorage::insert(LASTFM_USERNAME, &lastfm.username).unwrap();
    LocalStorage::insert(LASTFM_PASSWORD, &lastfm.password).unwrap();
}

pub fn get_lastfm_config() -> Option<LastFMConfig> {
    match (
        LocalStorage::get(LASTFM_API_KEY),
        LocalStorage::get(LASTFM_API_SECRET),
        LocalStorage::get(LASTFM_USERNAME),
        LocalStorage::get(LASTFM_PASSWORD),
    ) {
        (Ok(api_key), Ok(api_secret), Ok(username), Ok(password)) => Some(LastFMConfig {
            api_key,
            api_secret,
            username,
            password,
        }),
        _ => None,
    }
}

pub fn remove_lastfm_config() {
    LocalStorage::remove(LASTFM_API_KEY).unwrap();
    LocalStorage::remove(LASTFM_API_SECRET).unwrap();
    LocalStorage::remove(LASTFM_USERNAME).unwrap();
    LocalStorage::remove(LASTFM_PASSWORD).unwrap();
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