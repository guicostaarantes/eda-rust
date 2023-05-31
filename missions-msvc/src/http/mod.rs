mod extractors;
mod into_responses;

use crate::domain::mission_commander::MissionCommander;
use crate::domain::mission_commander::MissionCommanderError;
use crate::domain::mission_model::AstronautCrewInfo;
use crate::domain::mission_model::CreateMissionInput;
use crate::domain::mission_model::CreateMissionOutput;
use crate::domain::mission_model::Mission;
use crate::domain::mission_model::MissionCrewInfo;
use crate::domain::mission_model::UpdateCrewInput;
use crate::domain::mission_model::UpdateMissionInput;
use crate::domain::mission_querier::MissionQuerier;
use crate::domain::mission_querier::MissionQuerierError;
use crate::http::extractors::BearerToken;
use crate::http::extractors::BearerTokenExpiresIn;
use crate::providers::token::JwtTokenImpl;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::sse::Event;
use axum::response::Sse;
use axum::routing::get;
use axum::routing::post;
use axum::routing::put;
use axum::Extension;
use axum::Json;
use axum::Router;
use futures_util::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::StreamExt;

async fn create_mission(
    BearerToken(payload): BearerToken,
    Extension(mission_commander): Extension<Arc<MissionCommander>>,
    Json(input): Json<CreateMissionInput>,
) -> Result<CreateMissionOutput, MissionCommanderError> {
    let id = mission_commander.create_mission(payload, input).await?;

    Ok(CreateMissionOutput { id })
}

async fn update_mission(
    BearerToken(payload): BearerToken,
    Extension(mission_commander): Extension<Arc<MissionCommander>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateMissionInput>,
) -> Result<StatusCode, MissionCommanderError> {
    mission_commander.update_mission(payload, id, input).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_mission(
    BearerToken(_payload): BearerToken,
    Path(_id): Path<String>,
) -> Result<StatusCode, MissionCommanderError> {
    // TODO: implement
    Ok(StatusCode::NOT_FOUND)
}

async fn update_crew(
    BearerToken(payload): BearerToken,
    Extension(mission_commander): Extension<Arc<MissionCommander>>,
    Path(mission_id): Path<String>,
    Json(mut input): Json<UpdateCrewInput>,
) -> Result<StatusCode, MissionCommanderError> {
    input.mission_id = mission_id;

    mission_commander.update_crew(payload, input).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_mission(
    BearerToken(payload): BearerToken,
    Extension(mission_querier): Extension<Arc<MissionQuerier>>,
    Path(id): Path<String>,
) -> Result<Mission, MissionQuerierError> {
    mission_querier.get_mission_by_id(payload, id).await
}

async fn get_mission_crew(
    BearerToken(payload): BearerToken,
    Extension(mission_querier): Extension<Arc<MissionQuerier>>,
    Path(id): Path<String>,
) -> Result<MissionCrewInfo, MissionQuerierError> {
    mission_querier.get_mission_crew_info(payload, id).await
}

async fn get_astronaut(
    BearerToken(payload): BearerToken,
    Extension(mission_querier): Extension<Arc<MissionQuerier>>,
    Path(id): Path<String>,
) -> Result<AstronautCrewInfo, MissionQuerierError> {
    mission_querier.get_astronaut_crew_info(payload, id).await
}

async fn missions_updated_sse(
    BearerToken(payload): BearerToken,
    BearerTokenExpiresIn(ttl): BearerTokenExpiresIn,
    Extension(mission_querier): Extension<Arc<MissionQuerier>>,
    Path(ids): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, MissionQuerierError> {
    let input: Vec<String> = ids.split(',').map(|s| s.to_string()).collect();

    let stream = mission_querier
        .missions_or_crew_updated_stream(payload, ttl, input)
        .await?;

    let stream = stream.map(|s| Ok(Event::default().event("mission_updated").data(s)));

    Ok(Sse::new(stream))
}

pub fn missions_route(
    token_impl: Arc<JwtTokenImpl>,
    mission_querier: Arc<MissionQuerier>,
    mission_commander: Arc<MissionCommander>,
) -> Router {
    Router::new()
        .route("/missions", post(create_mission))
        .route(
            "/missions/:id",
            put(update_mission).delete(delete_mission).get(get_mission),
        )
        .route("/missions/:id/crew", put(update_crew).get(get_mission_crew))
        .route("/live/missions/:ids", get(missions_updated_sse))
        .route("/astronauts/:id", get(get_astronaut))
        .layer(Extension(token_impl))
        .layer(Extension(mission_querier))
        .layer(Extension(mission_commander))
}
