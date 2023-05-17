use crate::domain::astronaut_commander::AstronautCommanderError;
use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_model::CreateAstronautOutput;
use crate::domain::astronaut_model::UpdateAstronautOutput;
use crate::domain::astronaut_querier::AstronautQuerierError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use serde_json::json;

impl IntoResponse for AstronautCommanderError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AstronautCommanderError::AstronautNotFound => {
                (StatusCode::NOT_FOUND, "Astronaut not found")
            }
            AstronautCommanderError::AstronautWithNameExists => (
                StatusCode::CONFLICT,
                "Astronaut with same name already exists",
            ),
            AstronautCommanderError::NoFieldsToUpdate => {
                (StatusCode::UNPROCESSABLE_ENTITY, "No fields to update")
            }
            AstronautCommanderError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl IntoResponse for AstronautQuerierError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AstronautQuerierError::AstronautNotFound => {
                (StatusCode::NOT_FOUND, "Astronaut not found")
            }
            AstronautQuerierError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl IntoResponse for Astronaut {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(json!(self))).into_response()
    }
}

impl IntoResponse for CreateAstronautOutput {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(json!(self))).into_response()
    }
}

impl IntoResponse for UpdateAstronautOutput {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(json!(self))).into_response()
    }
}
