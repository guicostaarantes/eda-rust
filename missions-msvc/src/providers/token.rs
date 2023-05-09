use chrono::Utc;
use jwt_simple::prelude::NoCustomClaims;
use jwt_simple::prelude::RS256PublicKey;
use jwt_simple::prelude::RSAPublicKeyLike;
use jwt_simple::prelude::VerificationOptions;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

#[derive(Clone)]
pub struct JwtTokenImpl {
    public_keys: Vec<RS256PublicKey>,
}

#[derive(Clone)]
pub struct RawToken(pub String);

#[derive(Debug, Error)]
pub enum TokenImplError {
    #[error(transparent)]
    JwtImplError(#[from] jwt_simple::Error),
    #[error("token without expiration date")]
    TokenWithoutExpirationDate,
}

impl JwtTokenImpl {
    pub fn new(public_keys_pem: Vec<String>) -> Result<Self, TokenImplError> {
        let public_keys = public_keys_pem
            .iter()
            .map(|pem| RS256PublicKey::from_pem(&pem))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { public_keys })
    }
}

impl JwtTokenImpl {
    pub fn validate_token<T: Serialize + DeserializeOwned>(
        &self,
        raw_token: &RawToken,
    ) -> Result<T, TokenImplError> {
        self.public_keys
            .iter()
            .find_map(|key| {
                match key.verify_token::<T>(
                    &raw_token.0,
                    Some(VerificationOptions {
                        time_tolerance: None,
                        ..VerificationOptions::default()
                    }),
                ) {
                    Ok(claims) => Some(claims.custom),
                    Err(_) => None,
                }
            })
            .ok_or(TokenImplError::JwtImplError(jwt_simple::Error::new(
                jwt_simple::JWTError::InvalidSignature,
            )))
    }
}

impl JwtTokenImpl {
    pub fn get_token_seconds_remaining(&self, raw_token: &RawToken) -> Result<u64, TokenImplError> {
        let expires_at = self
            .public_keys
            .iter()
            .find_map(|key| {
                match key.verify_token::<NoCustomClaims>(
                    &raw_token.0,
                    Some(VerificationOptions {
                        time_tolerance: None,
                        ..VerificationOptions::default()
                    }),
                ) {
                    Ok(claims) => Some(claims.expires_at),
                    Err(_) => None,
                }
            })
            .ok_or(TokenImplError::JwtImplError(jwt_simple::Error::new(
                jwt_simple::JWTError::InvalidSignature,
            )))?;

        match expires_at {
            Some(exp) => Ok(exp.as_secs() - Utc::now().timestamp() as u64),
            None => Err(TokenImplError::TokenWithoutExpirationDate),
        }
    }
}
