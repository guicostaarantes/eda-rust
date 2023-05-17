use crate::domain::token_commander::TokenCommanderError;
use crate::domain::token_model::TokenPair;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use serde_json::json;

impl IntoResponse for TokenCommanderError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            TokenCommanderError::BadCredentials => (StatusCode::UNAUTHORIZED, "Bad Credentials"),
            TokenCommanderError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl IntoResponse for TokenPair {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(json!(self))).into_response()
    }
}
