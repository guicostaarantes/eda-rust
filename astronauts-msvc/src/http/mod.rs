mod extractors;
mod into_responses;

use crate::domain::astronaut_commander::AstronautCommander;
use crate::domain::astronaut_commander::AstronautCommanderError;
use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_model::CreateAstronautInput;
use crate::domain::astronaut_model::CreateAstronautOutput;
use crate::domain::astronaut_model::UpdateAstronautInput;
use crate::domain::astronaut_querier::AstronautQuerier;
use crate::domain::astronaut_querier::AstronautQuerierError;
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
use futures_util::stream;
use futures_util::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::StreamExt;

async fn create_astronaut(
    Extension(astronaut_commander): Extension<Arc<AstronautCommander>>,
    Json(input): Json<CreateAstronautInput>,
) -> Result<CreateAstronautOutput, AstronautCommanderError> {
    let id = astronaut_commander.create_astronaut(input).await?;

    Ok(CreateAstronautOutput { id })
}

async fn update_astronaut(
    BearerToken(payload): BearerToken,
    Extension(astronaut_commander): Extension<Arc<AstronautCommander>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateAstronautInput>,
) -> Result<StatusCode, AstronautCommanderError> {
    astronaut_commander
        .update_astronaut(payload, id, input)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_astronaut(
    BearerToken(_payload): BearerToken,
    Path(_id): Path<String>,
) -> Result<StatusCode, AstronautCommanderError> {
    // TODO: implement
    Ok(StatusCode::NO_CONTENT)
}

async fn get_astronaut(
    BearerToken(payload): BearerToken,
    Extension(astronaut_querier): Extension<Arc<AstronautQuerier>>,
    Path(id): Path<String>,
) -> Result<Astronaut, AstronautQuerierError> {
    astronaut_querier.get_astronaut_by_id(payload, id).await
}

async fn astronauts_updated_sse(
    BearerToken(payload): BearerToken,
    BearerTokenExpiresIn(ttl): BearerTokenExpiresIn,
    Extension(astronaut_querier): Extension<Arc<AstronautQuerier>>,
    Json(input): Json<Vec<String>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AstronautQuerierError> {
    let stream = astronaut_querier
        .astronauts_updated_stream(payload, ttl, input)
        .await?;

    let stream = stream.map(|s| Ok(Event::default().event("astronaut_updated").data(s)));

    Ok(Sse::new(stream))
}

async fn example_stream() -> Sse<impl Stream<Item = Result<Event, std::fmt::Error>>> {
    let stream = stream::iter(0..=10)
        .map(|s| Ok(Event::default().event("time_elapsed").data(s.to_string())))
        .throttle(std::time::Duration::from_secs(1));

    Sse::new(stream)
}

pub fn astronauts_route(
    token_impl: Arc<JwtTokenImpl>,
    astronaut_querier: Arc<AstronautQuerier>,
    astronaut_commander: Arc<AstronautCommander>,
) -> Router {
    Router::new()
        .route("/astronauts", post(create_astronaut))
        .route(
            "/astronauts/:id",
            put(update_astronaut)
                .delete(delete_astronaut)
                .get(get_astronaut),
        )
        .route("/astronauts-updated", get(astronauts_updated_sse))
        .route("/time-elapsed", get(example_stream))
        .layer(Extension(token_impl))
        .layer(Extension(astronaut_querier))
        .layer(Extension(astronaut_commander))
}
