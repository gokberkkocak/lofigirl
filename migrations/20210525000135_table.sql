-- Add migration script here
CREATE TABLE IF NOT EXISTS tokens (
    id                           INTEGER PRIMARY KEY NOT NULL,
    token                        TEXT NOT NULL,
    lastfm_id                    INT,
    listenbrainz_id              INT,
    FOREIGN KEY(lastfm_id)       REFERENCES lastfm(id),
    FOREIGN KEY(listenbrainz_id) REFERENCES listenbrainz(id)
);

CREATE TABLE IF NOT EXISTS lastfm (
    id          INTEGER PRIMARY KEY NOT NULL,
    session_key    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS listenbrainz (
    id          INTEGER PRIMARY KEY NOT NULL,
    token       TEXT NOT NULL
);
