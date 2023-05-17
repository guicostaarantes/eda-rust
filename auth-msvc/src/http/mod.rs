mod extractors;
mod into_responses;

use crate::domain::token_commander::TokenCommander;
use crate::domain::token_commander::TokenCommanderError;
use crate::domain::token_model::AstronautCredentialsInput;
use crate::domain::token_model::TokenPair;
use crate::http::extractors::BearerToken;
use crate::http::extractors::BearerTokenSignature;
use crate::providers::token::JwtTokenImpl;
use axum::http::StatusCode;
use axum::routing::delete;
use axum::routing::post;
use axum::routing::put;
use axum::Extension;
use axum::Json;
use axum::Router;
use std::sync::Arc;

async fn create_token_pair(
    Extension(token_commander): Extension<Arc<TokenCommander>>,
    Json(input): Json<AstronautCredentialsInput>,
) -> Result<TokenPair, TokenCommanderError> {
    token_commander
        .exchange_astronaut_credentials_for_token_pair(input)
        .await
}

async fn refresh_token_pair(
    BearerToken(payload): BearerToken,
    BearerTokenSignature(signature): BearerTokenSignature,
    Extension(token_commander): Extension<Arc<TokenCommander>>,
) -> Result<TokenPair, TokenCommanderError> {
    token_commander.refresh_token_pair(payload, signature).await
}

async fn delete_token_pair(
    BearerToken(payload): BearerToken,
    Extension(token_commander): Extension<Arc<TokenCommander>>,
) -> Result<StatusCode, TokenCommanderError> {
    token_commander.invalidate_refresh_token(payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn auth_route(token_impl: Arc<JwtTokenImpl>, token_commander: Arc<TokenCommander>) -> Router {
    Router::new()
        .route("/token_pair", post(create_token_pair))
        .route("/token_pair", put(refresh_token_pair))
        .route("/token_pair", delete(delete_token_pair))
        .layer(Extension(token_impl))
        .layer(Extension(token_commander))
}
