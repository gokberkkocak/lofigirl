use jwt_compact::{alg::Hs256, AlgorithmExt as _, Token, UntrustedToken};
use serde::{Deserialize, Serialize};

use crate::encrypt::SecureString;

const JWT_SHARED_SECRET: &str = include_str!("../../secrets/key.aes");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWTClaims {
    pub secure_token: SecureString,
}

impl JWTClaims {
    pub fn encode(token: String) -> anyhow::Result<String> {
        let time_options = jwt_compact::TimeOptions::default();
        let key = jwt_compact::alg::Hs256Key::new(JWT_SHARED_SECRET);
        let my_claims = JWTClaims {
            secure_token: token.into(),
        };
        let claims = jwt_compact::Claims::new(my_claims)
            .set_duration_and_issuance(&time_options, chrono::Duration::hours(1))
            .set_not_before(chrono::Utc::now());
        let header = jwt_compact::Header::empty();
        let token_string = jwt_compact::alg::Hs256.token(&header, &claims, &key)?;
        Ok(token_string)
    }

    pub fn decode(encoded: String) -> anyhow::Result<String> {
        let time_options = jwt_compact::TimeOptions::default();
        let key = jwt_compact::alg::Hs256Key::new(JWT_SHARED_SECRET);
        let token = UntrustedToken::new(&encoded)?;
        let token: Token<JWTClaims> = Hs256.validator(&key).validate(&token)?;

        token
            .claims()
            .validate_expiration(&time_options)?
            .validate_maturity(&time_options)?;
        Ok(token.claims().custom.clone().secure_token.into())
    }
}
