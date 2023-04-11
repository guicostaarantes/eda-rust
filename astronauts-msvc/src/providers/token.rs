use crate::providers::json::JsonSerializerImpl;
use crate::providers::mem_state::RedisMemStateImpl;
use crate::providers::random::RandomImpl;
use chrono::DateTime;
use chrono::Duration;
use chrono::NaiveDateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone)]
pub struct TokenImpl {
    mem_state: Arc<RedisMemStateImpl>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct TokenPayload {
    pub astronaut_id: String,
    pub permissions: Vec<String>,
}

#[derive(Clone)]
pub struct RawToken(pub String);

pub struct Token {
    pub id: String,
    pub payload: TokenPayload,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum TokenImplError {
    #[error(transparent)]
    MemStateImplError(#[from] redis::RedisError),
    #[error(transparent)]
    JsonSerializerImplError(#[from] serde_json::Error),
    #[error("bad timestamp")]
    BadTimestamp,
    #[error("token expired")]
    TokenExpired,
}

impl TokenImpl {
    pub fn new(mem_state: Arc<RedisMemStateImpl>) -> Self {
        Self { mem_state }
    }
}

impl TokenImpl {
    pub async fn produce_token(
        &self,
        astronaut_id: &str,
        permissions: &[&str],
        expires_in: u64,
    ) -> Result<Token, TokenImplError> {
        let expires_at = Utc::now() + Duration::seconds(expires_in as i64);

        let id = format!("{}.{}", RandomImpl::string(64), expires_at.timestamp());

        let payload = TokenPayload {
            astronaut_id: astronaut_id.to_string(),
            permissions: permissions.iter().map(|s| s.to_string()).collect(),
        };

        let serialized_payload = match JsonSerializerImpl::serialize(&payload) {
            Ok(payload) => Ok(payload),
            Err(err) => Err(TokenImplError::JsonSerializerImplError(err)),
        }?;

        self.mem_state
            .set(&id, &serialized_payload, Some(expires_in))
            .await?;

        Ok(Token {
            id,
            payload,
            expires_at,
        })
    }
}

impl TokenImpl {
    pub async fn fetch_token(&self, raw_token: &RawToken) -> Result<Token, TokenImplError> {
        let expires_at = match raw_token.0.split_once('.') {
            Some((_, expires_at)) => match expires_at.parse::<i64>() {
                Ok(expires_at_i64) => match NaiveDateTime::from_timestamp_opt(expires_at_i64, 0) {
                    Some(tmstp) => Ok(DateTime::from_utc(tmstp, Utc)),
                    None => Err(TokenImplError::BadTimestamp),
                },
                Err(_) => Err(TokenImplError::BadTimestamp),
            },
            None => Err(TokenImplError::BadTimestamp),
        }?;

        if expires_at < Utc::now() {
            return Err(TokenImplError::TokenExpired);
        }

        let content = self.mem_state.get(&raw_token.0).await?;

        let payload = match JsonSerializerImpl::deserialize::<TokenPayload>(&content) {
            Ok(result) => Ok(result),
            Err(err) => Err(TokenImplError::JsonSerializerImplError(err)),
        }?;

        Ok(Token {
            id: raw_token.0.to_string(),
            payload,
            expires_at,
        })
    }
}

impl TokenImpl {
    pub async fn destroy_token(&self, token: &str) -> Result<(), TokenImplError> {
        self.mem_state.unset(&token).await?;
        Ok(())
    }
}
