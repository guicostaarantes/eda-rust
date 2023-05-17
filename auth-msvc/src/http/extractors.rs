use crate::domain::token_model::RefreshTokenPayload;
use crate::providers::token::JwtTokenImpl;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Extension;
use axum::Json;
use axum::RequestPartsExt;
use axum::TypedHeader;
use serde_json::json;
use std::sync::Arc;

pub(super) struct BearerToken(pub RefreshTokenPayload);

#[async_trait]
impl<S> FromRequestParts<S> for BearerToken
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token_impl = parts
            .extract::<Extension<Arc<JwtTokenImpl>>>()
            .await
            .map_err(|rej| {
                log::error!("Expected to extract JwtTokenImpl: {}", rej.to_string());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Internal Server Error"})),
                )
                    .into_response()
            })?;

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Unauthorized"})),
                )
                    .into_response()
            })?;

        let payload = token_impl
            .validate_token::<RefreshTokenPayload>(bearer.token())
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Unauthorized"})),
                )
                    .into_response()
            })?;

        Ok(BearerToken(payload))
    }
}

pub(super) struct BearerTokenSignature(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for BearerTokenSignature
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Unauthorized"})),
                )
                    .into_response()
            })?;

        let token = bearer.token();
        Ok(BearerTokenSignature(token[token.len() - 16..].to_string()))
    }
}
