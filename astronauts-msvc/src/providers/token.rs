use crate::providers::mem_state::RedisMemStateImpl;
use crate::providers::random::RandomImpl;
use chrono::DateTime;
use chrono::Duration;
use chrono::NaiveDateTime;
use chrono::Utc;
use thiserror::Error;

#[derive(Clone)]
pub struct TokenImpl {
    mem_state: RedisMemStateImpl,
}

pub struct Token {
    pub id: String,
    pub content: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum TokenImplError {
    #[error(transparent)]
    MemStateImplError(#[from] redis::RedisError),
    #[error("bad timestamp")]
    BadTimestamp,
    #[error("token expired")]
    TokenExpired,
}

impl TokenImpl {
    pub fn new(mem_state: RedisMemStateImpl) -> Self {
        Self { mem_state }
    }
}

impl TokenImpl {
    pub async fn produce_token(
        &self,
        payload: &str,
        expires_in: u64,
    ) -> Result<Token, TokenImplError> {
        let expires_at = Utc::now() + Duration::seconds(expires_in as i64);
        let id = format!("{}.{}", RandomImpl::string(64), expires_at.timestamp());
        let content = payload.to_string();

        self.mem_state.set(&id, &content, Some(expires_in)).await?;

        Ok(Token {
            id,
            content,
            expires_at,
        })
    }
}

impl TokenImpl {
    pub async fn fetch_token(&self, id: &str) -> Result<Token, TokenImplError> {
        let expires_at = match id.split_once('.') {
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

        let content = self.mem_state.get(&id).await?;

        Ok(Token {
            id: id.to_string(),
            content,
            expires_at,
        })
    }
}
