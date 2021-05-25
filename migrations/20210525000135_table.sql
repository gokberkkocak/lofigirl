-- Add migration script here
CREATE TABLE IF NOT EXISTS tokens (
    id          INTEGER PRIMARY KEY NOT NULL,
    token       TEXT NOT NULL,
    listenbrainz_token TEXT,
    lastfm_api_key TEXT,
    lastfm_api_secret TEXT,
    lastfm_username TEXT,
    lastfm_password TEXT
);