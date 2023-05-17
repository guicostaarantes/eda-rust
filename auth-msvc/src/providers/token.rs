use chrono::Utc;
use jwt_simple::prelude::Claims;
use jwt_simple::prelude::Duration;
use jwt_simple::prelude::NoCustomClaims;
use jwt_simple::prelude::RS256KeyPair;
use jwt_simple::prelude::RS256PublicKey;
use jwt_simple::prelude::RSAKeyPairLike;
use jwt_simple::prelude::RSAPublicKeyLike;
use jwt_simple::prelude::VerificationOptions;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

#[derive(Clone)]
pub struct JwtTokenImpl {
    public_keys: Vec<RS256PublicKey>,
    private_key: RS256KeyPair,
}

#[derive(Debug, Error)]
pub enum TokenImplError {
    #[error(transparent)]
    JwtImplError(#[from] jwt_simple::Error),
    #[error("token without expiration date")]
    TokenWithoutExpirationDate,
}

impl JwtTokenImpl {
    pub fn new(
        public_keys_pem: Vec<String>,
        private_key_pem: String,
    ) -> Result<Self, TokenImplError> {
        let public_keys = public_keys_pem
            .iter()
            .map(|pem| RS256PublicKey::from_pem(&pem))
            .collect::<Result<Vec<_>, _>>()?;

        let private_key = RS256KeyPair::from_pem(&private_key_pem)?;

        Ok(Self {
            public_keys,
            private_key,
        })
    }
}

impl JwtTokenImpl {
    pub fn produce_token<T: Serialize + DeserializeOwned>(
        &self,
        expires_in: u64,
        extra_parameters: T,
    ) -> Result<String, TokenImplError> {
        let claims =
            Claims::with_custom_claims::<T>(extra_parameters, Duration::from_secs(expires_in));
        let token = self.private_key.sign(claims)?;
        Ok(token)
    }
}

impl JwtTokenImpl {
    pub fn validate_token<T: Serialize + DeserializeOwned>(
        &self,
        raw_token: &str,
    ) -> Result<T, TokenImplError> {
        self.public_keys
            .iter()
            .find_map(|key| {
                match key.verify_token::<T>(
                    raw_token,
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
    #[allow(dead_code)]
    pub fn get_token_seconds_remaining(&self, raw_token: &str) -> Result<u64, TokenImplError> {
        let expires_at = self
            .public_keys
            .iter()
            .find_map(|key| {
                match key.verify_token::<NoCustomClaims>(
                    raw_token,
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
