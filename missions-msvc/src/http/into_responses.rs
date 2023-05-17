use crate::domain::mission_commander::MissionCommanderError;
use crate::domain::mission_model::CreateMissionOutput;
use crate::domain::mission_model::Mission;
use crate::domain::mission_querier::MissionQuerierError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use serde_json::json;

impl IntoResponse for MissionCommanderError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            MissionCommanderError::MissionNotFound => (StatusCode::NOT_FOUND, "Mission not found"),
            MissionCommanderError::MissionWithNameExists => (
                StatusCode::CONFLICT,
                "Mission with same name already exists",
            ),
            MissionCommanderError::NoFieldsToUpdate => {
                (StatusCode::UNPROCESSABLE_ENTITY, "No fields to update")
            }
            MissionCommanderError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl IntoResponse for MissionQuerierError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            MissionQuerierError::MissionNotFound => (StatusCode::NOT_FOUND, "Mission not found"),
            MissionQuerierError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            MissionQuerierError::AstronautNotFound => {
                (StatusCode::NOT_FOUND, "Astronaut not found")
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl IntoResponse for Mission {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(json!(self))).into_response()
    }
}

impl IntoResponse for CreateMissionOutput {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(json!(self))).into_response()
    }
}
