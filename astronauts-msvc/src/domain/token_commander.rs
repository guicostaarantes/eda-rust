use crate::domain::astronaut_model::AstronautDocument;
use crate::domain::token_model::AccessTokenPayload;
use crate::domain::token_model::AstronautCredentialsInput;
use crate::domain::token_model::RefreshTokenCreatedEvent;
use crate::domain::token_model::RefreshTokenDocument;
use crate::domain::token_model::RefreshTokenPayload;
use crate::domain::token_model::RefreshTokenRevokedEvent;
use crate::domain::token_model::TokenPair;
use crate::domain::token_model::ACCESS_TOKEN_EXPIRES_IN_SECONDS;
use crate::domain::token_model::PASSWORD_GRANT_PERMISSIONS;
use crate::domain::token_model::REFRESH_TOKEN_EXPIRES_IN_SECONDS;
use crate::providers::emitter::KafkaEmitterImpl;
use crate::providers::hash::HashImpl;
use crate::providers::hash::HashImplError;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::random::RandomImpl;
use crate::providers::state::MongoStateImpl;
use crate::providers::token::JwtTokenImpl;
use crate::providers::token::RawToken;
use crate::providers::token::TokenImplError;
use log::error;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenCommanderError {
    #[error(transparent)]
    EmitterImplError(#[from] rdkafka::error::KafkaError),
    #[error(transparent)]
    HashImplError(#[from] HashImplError),
    #[error(transparent)]
    JsonSerializerImplError(#[from] serde_json::Error),
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
    #[error(transparent)]
    TokenImplError(#[from] TokenImplError),
    #[error("bad credentials")]
    BadCredentials,
    #[error("forbidden")]
    Forbidden,
}

#[derive(Clone)]
pub struct TokenCommander {
    emitter: Arc<KafkaEmitterImpl>,
    state: Arc<MongoStateImpl>,
    token: Arc<JwtTokenImpl>,
}

impl TokenCommander {
    pub fn new(
        emitter: Arc<KafkaEmitterImpl>,
        state: Arc<MongoStateImpl>,
        token: Arc<JwtTokenImpl>,
    ) -> Self {
        Self {
            emitter,
            state,
            token,
        }
    }
}

fn get_token_signature(token: &str) -> String {
    token[token.len() - 16..].to_string()
}

impl TokenCommander {
    pub async fn exchange_astronaut_credentials_for_token_pair(
        &self,
        input: AstronautCredentialsInput,
    ) -> Result<TokenPair, TokenCommanderError> {
        let astronaut = match self
            .state
            .find_one_by_field::<AstronautDocument>("astronauts", "name", &input.name)
            .await
        {
            Err(err) => Err(TokenCommanderError::StateImplError(err)),
            Ok(None) => Err(TokenCommanderError::BadCredentials),
            Ok(Some(astronaut)) => Ok(astronaut),
        }?;

        match HashImpl::verify(&input.password, &astronaut.password) {
            Ok(_) => Ok(()),
            Err(_) => Err(TokenCommanderError::BadCredentials),
        }?;

        let refresh_token_payload = RefreshTokenPayload {
            family_id: RandomImpl::uuid(),
            astronaut_id: astronaut.id.to_string(),
            permissions: Vec::from(PASSWORD_GRANT_PERMISSIONS),
        };

        let access_token_payload = AccessTokenPayload {
            family_id: refresh_token_payload.family_id.clone(),
            astronaut_id: refresh_token_payload.astronaut_id.clone(),
            permissions: refresh_token_payload.permissions.clone(),
        };

        let refresh_token = match self.token.produce_token(
            REFRESH_TOKEN_EXPIRES_IN_SECONDS,
            refresh_token_payload.clone(),
        ) {
            Ok(token) => Ok(token),
            Err(err) => Err(TokenCommanderError::TokenImplError(err)),
        }?;

        let access_token = match self
            .token
            .produce_token(ACCESS_TOKEN_EXPIRES_IN_SECONDS, access_token_payload)
        {
            Ok(token) => Ok(token),
            Err(err) => Err(TokenCommanderError::TokenImplError(err)),
        }?;

        let refresh_token_event_payload = RefreshTokenCreatedEvent {
            id: refresh_token_payload.family_id.clone(),
            signature: get_token_signature(&refresh_token),
            astronaut_id: refresh_token_payload.astronaut_id.clone(),
            permissions: refresh_token_payload.permissions.clone(),
        };

        // TODO: internalize this in the emit fn
        let payload = JsonSerializerImpl::serialize(&refresh_token_event_payload)?;

        self.emitter
            .emit(
                "refresh_token_created",
                &refresh_token_payload.family_id.clone(),
                &payload,
            )
            .await?;

        Ok(TokenPair {
            refresh_token,
            access_token,
        })
    }
}

impl TokenCommander {
    pub async fn refresh_token_pair(
        &self,
        raw_token: &RawToken,
    ) -> Result<TokenPair, TokenCommanderError> {
        let refresh_token_payload =
            match self.token.validate_token::<RefreshTokenPayload>(raw_token) {
                Ok(token) => Ok(token),
                Err(_) => Err(TokenCommanderError::Forbidden),
            }?;

        // TODO: check if signature in state matches the one from the token
        // if it doesn't, it means that the same token is being used across
        // multiple devices, which means an attacker might have stolen the
        // refresh token. In this case, the family must be removed in order
        // to not produce new access tokens.

        let rt_in_state = match self
            .state
            .find_one_by_id::<RefreshTokenDocument>(
                "refresh_tokens",
                &refresh_token_payload.family_id,
            )
            .await
        {
            Err(err) => Err(TokenCommanderError::StateImplError(err)),
            Ok(None) => Err(TokenCommanderError::Forbidden),
            Ok(Some(token)) => Ok(token),
        }?;

        if rt_in_state.signature != get_token_signature(&raw_token.0) {
            return match self.invalidate_refresh_token(raw_token).await {
                Ok(_) => Err(TokenCommanderError::Forbidden),
                Err(err) => Err(err),
            };
        }

        let access_token_payload = AccessTokenPayload {
            family_id: refresh_token_payload.family_id.clone(),
            astronaut_id: refresh_token_payload.astronaut_id.clone(),
            permissions: refresh_token_payload.permissions.clone(),
        };

        let refresh_token = match self.token.produce_token(
            REFRESH_TOKEN_EXPIRES_IN_SECONDS,
            refresh_token_payload.clone(),
        ) {
            Ok(token) => Ok(token),
            Err(err) => Err(TokenCommanderError::TokenImplError(err)),
        }?;

        let access_token = match self
            .token
            .produce_token(ACCESS_TOKEN_EXPIRES_IN_SECONDS, access_token_payload)
        {
            Ok(token) => Ok(token),
            Err(err) => Err(TokenCommanderError::TokenImplError(err)),
        }?;

        let refresh_token_event_payload = RefreshTokenCreatedEvent {
            id: refresh_token_payload.family_id.clone(),
            signature: get_token_signature(&refresh_token),
            astronaut_id: refresh_token_payload.astronaut_id.clone(),
            permissions: refresh_token_payload.permissions.clone(),
        };

        let payload = JsonSerializerImpl::serialize(&refresh_token_event_payload)?;

        self.emitter
            .emit(
                "refresh_token_created",
                &refresh_token_payload.family_id.clone(),
                &payload,
            )
            .await?;

        Ok(TokenPair {
            refresh_token,
            access_token,
        })
    }
}

impl TokenCommander {
    pub async fn invalidate_refresh_token(
        &self,
        raw_token: &RawToken,
    ) -> Result<(), TokenCommanderError> {
        let refresh_token_payload =
            match self.token.validate_token::<RefreshTokenPayload>(raw_token) {
                Ok(token) => Ok(token),
                Err(_) => Err(TokenCommanderError::Forbidden),
            }?;

        let refresh_token_event_payload = RefreshTokenRevokedEvent {
            id: refresh_token_payload.family_id.clone(),
        };

        let payload = JsonSerializerImpl::serialize(&refresh_token_event_payload)?;

        self.emitter
            .emit(
                "refresh_token_revoked",
                &refresh_token_payload.family_id.clone(),
                &payload,
            )
            .await?;

        Ok(())
    }
}
