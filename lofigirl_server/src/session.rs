use anyhow::Result;
use lofigirl_shared_common::config::{ConfigError, LastFMClientSessionConfig, ListenBrainzConfig};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct TokenDB {
    pool: SqlitePool,
}

impl TokenDB {
    pub async fn new(filename: &str) -> Result<Self> {
        let token_db = TokenDB {
            pool: SqlitePool::connect(&format!("sqlite:{}", filename)).await?,
        };
        Ok(token_db)
    }

    pub async fn get_or_generate_token(
        &self,
        lastfm_session_key: &Option<String>,
        listenbrainz_token: &Option<String>,
    ) -> Result<String> {
        (lastfm_session_key.is_some() || listenbrainz_token.is_some())
            .then(|| ())
            .ok_or(ConfigError::EmptyListeners)?;
        let mut conn = self.pool.acquire().await?;
        let lfm_id = if let Some(lastfm) = lastfm_session_key {
            Some(self.get_lastfm_id(&lastfm).await?)
        } else {
            None
        };
        let lb_id = if let Some(listenbrainz) = listenbrainz_token {
            Some(self.get_listenbrainz_id(&listenbrainz).await?)
        } else {
            None
        };

        let optional_token = sqlx::query!(
            r#"
                SELECT token FROM tokens WHERE lastfm_id = ?1 AND listenbrainz_id = ?2
            "#,
            lfm_id,
            lb_id
        )
        .fetch_optional(&mut conn)
        .await?;
        match optional_token {
            Some(rec) => Ok(rec.token),
            None => {
                let token = Uuid::new_v4();
                let token_str = token
                    .hyphenated()
                    .encode_lower(&mut Uuid::encode_buffer())
                    .to_owned();
                let _id = sqlx::query!(
                    r#"
                        INSERT INTO tokens ( token, lastfm_id, listenbrainz_id )
                        VALUES ( ?1, ?2, ?3)
                    "#,
                    token_str,
                    lfm_id,
                    lb_id
                )
                .execute(&mut conn)
                .await?
                .rows_affected();
                Ok(token_str)
            }
        }
    }

    async fn get_lastfm_id(&self, lastfm_session_key: &str) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let optional_id = sqlx::query!(
            r#"
                SELECT id FROM lastfm WHERE session_key = ?1
            "#,
            lastfm_session_key
        )
        .fetch_optional(&mut conn)
        .await?;
        match optional_id {
            Some(rec) => Ok(rec.id),
            None => {
                let id = sqlx::query!(
                    r#"
                        INSERT INTO lastfm ( session_key )
                        VALUES ( ?1 )
                    "#,
                    lastfm_session_key,
                )
                .execute(&mut conn)
                .await?
                .last_insert_rowid();
                Ok(id)
            }
        }
    }

    async fn get_listenbrainz_id(&self, listenbrainz_token: &str) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let optional_id = sqlx::query!(
            r#"
                SELECT id FROM listenbrainz WHERE token = ?1
            "#,
            listenbrainz_token
        )
        .fetch_optional(&mut conn)
        .await?;
        match optional_id {
            Some(rec) => Ok(rec.id),
            None => {
                let id = sqlx::query!(
                    r#"
                        INSERT INTO listenbrainz ( token )
                        VALUES ( ?1 )
                    "#,
                    listenbrainz_token,
                )
                .execute(&mut conn)
                .await?
                .last_insert_rowid();
                Ok(id)
            }
        }
    }

    pub async fn get_info_from_token(
        &self,
        token_str: &str,
    ) -> Result<(
        Option<LastFMClientSessionConfig>,
        Option<ListenBrainzConfig>,
    )> {
        let mut conn = self.pool.acquire().await?;
        let rec = sqlx::query!(
            r#"
                SELECT lastfm_id, listenbrainz_id FROM tokens
                WHERE token = ?1
            "#,
            token_str
        )
        .fetch_one(&mut conn)
        .await?;
        let lb_config = if let Some(id) = rec.listenbrainz_id {
            Some(self.get_listenbrainz_config(id).await?)
        } else {
            None
        };
        let lfm_config = if let Some(id) = rec.lastfm_id {
            Some(self.get_lastfm_config(id).await?)
        } else {
            None
        };
        Ok((lfm_config, lb_config))
    }

    async fn get_listenbrainz_config(&self, id: i64) -> Result<ListenBrainzConfig> {
        let mut conn = self.pool.acquire().await?;
        let rec = sqlx::query!(
            r#"
                SELECT token FROM listenbrainz
                WHERE id = ?1
            "#,
            id
        )
        .fetch_one(&mut conn)
        .await?;
        Ok(ListenBrainzConfig { token: rec.token })
    }

    async fn get_lastfm_config(&self, id: i64) -> Result<LastFMClientSessionConfig> {
        let mut conn = self.pool.acquire().await?;
        let rec = sqlx::query!(
            r#"
                SELECT session_key FROM lastfm
                WHERE id = ?1
            "#,
            id
        )
        .fetch_one(&mut conn)
        .await?;
        Ok(LastFMClientSessionConfig {
            session_key: rec.session_key,
        })
    }
}
