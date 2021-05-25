use anyhow::Result;
use lofigirl_shared_common::config::{ConfigError, LastFMConfig, ListenBrainzConfig};
use sqlx::SqlitePool;
use uuid::Uuid;

struct TokenDB {
    pool: SqlitePool,
}

impl TokenDB {
    pub async fn new(filename: &str) -> Result<Self> {
        let token_db = TokenDB {
            pool: SqlitePool::connect(&std::env::var(filename)?).await?,
        };
        Ok(token_db)
    }

    pub async fn insert_new(
        &self,
        lastfm: Option<LastFMConfig>,
        listenbrainz: Option<ListenBrainzConfig>,
    ) -> Result<Uuid> {
        (lastfm.is_some() && listenbrainz.is_some())
            .then(|| ())
            .ok_or(ConfigError::EmptyListeners)?;
        let mut conn = self.pool.acquire().await?;
        let uuid = Uuid::new_v4();
        let lastfm_fields = if let Some(l) = lastfm {
            (
                Some(l.api_key),
                Some(l.api_secret),
                Some(l.username),
                Some(l.password),
            )
        } else {
            (None, None, None, None)
        };
        let listenbrainz_field = if let Some(l) = listenbrainz {
            Some(l.token)
        } else {
            None
        };
        let token_string = uuid.to_hyphenated().to_string();
        let _id = sqlx::query!(
                r#"
                    INSERT INTO tokens ( token, listenbrainz_token, lastfm_api_key, lastfm_api_secret, lastfm_username, lastfm_password  )
                    VALUES ( ?1, ?2, ?3, ?4, ?5, ?6 )
                "#,
                token_string,
                listenbrainz_field,
                lastfm_fields.0,
                lastfm_fields.1,
                lastfm_fields.2,
                lastfm_fields.3,
            )
            .execute(&mut conn)
            .await?
            .rows_affected();

        Ok(uuid)
    }

    pub async fn get_info_from_token(
        &self,
        token_uuid: Uuid,
    ) -> Result<(Option<LastFMConfig>, Option<ListenBrainzConfig>)> {
        let mut conn = self.pool.acquire().await?;
        let token_string = token_uuid.to_hyphenated().to_string();
        let res = sqlx::query!(
            r#"
                SELECT listenbrainz_token, lastfm_api_key, lastfm_api_secret, lastfm_username, lastfm_password FROM tokens
                WHERE token = ?1
            "#,
            token_string
        )
        .fetch_one(&mut conn)
        .await?;
        let lb_config = if let Some(token) = res.listenbrainz_token {
            Some(ListenBrainzConfig { token })
        } else {
            None
        };
        let lfm_config = match (
            res.lastfm_api_key,
            res.lastfm_api_secret,
            res.lastfm_username,
            res.lastfm_password,
        ) {
            (Some(api_key), Some(api_secret), Some(username), Some(password)) => {
                Some(LastFMConfig {
                    api_key,
                    api_secret,
                    username,
                    password,
                })
            }
            _ => None,
        };

        Ok((lfm_config, lb_config))
    }
}
