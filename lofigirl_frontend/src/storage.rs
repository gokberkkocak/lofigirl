use seed::browser::web_storage::LocalStorage;
use seed::browser::web_storage::WebStorage;
use seed::browser::web_storage::WebStorageError;

const LASTFM_API_KEY: &'static str = "lastfm_api_key";
const LASTFM_API_SARE: &'static str = "lastfm_api_secret";
const LASTFM_USERNAME: &'static str = "lastfm_username";
const LASTFM_PASSWORD: &'static str = "lastfm_password";
const LISTENBRAINZ_TOKEN: &'static str = "listenbrainz_token";

pub fn set_listenbrainz_token(token: &str) {
    LocalStorage::insert(LISTENBRAINZ_TOKEN, token).unwrap();
}

pub fn get_listenbrainz_token() -> Option<String> {
    match LocalStorage::get(LISTENBRAINZ_TOKEN) {
        Ok(value) => Some(value),
        Err(err) => match err {
            WebStorageError::KeyNotFoundError => None,
            other_error => panic!("{:?}", other_error),
        },
    }
}

pub fn remove_listenbrainz_token() {
    LocalStorage::remove(LISTENBRAINZ_TOKEN).unwrap();
}
