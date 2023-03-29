mod domain;
mod providers;
mod schema;

use std::env;

use crate::domain::astronaut_commander::AstronautCommander;
use crate::domain::astronaut_querier::AstronautQuerier;
use crate::domain::astronaut_synchronizer::AstronautSynchronizer;
use crate::providers::emitter::KafkaEmitterImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::mem_state::RedisMemStateImpl;
use crate::providers::state::MongoStateImpl;
use crate::providers::token::TokenImpl;
use crate::schema::AstronautsSchema;
use crate::schema::MutationRoot;
use crate::schema::QueryRoot;
use crate::schema::SubscriptionRoot;
use async_graphql::http::playground_source;
use async_graphql::http::GraphQLPlaygroundConfig;
use async_graphql::Schema;
use async_graphql_axum::GraphQLRequest;
use async_graphql_axum::GraphQLResponse;
use async_graphql_axum::GraphQLSubscription;
use axum::extract::Extension;
use axum::headers::HeaderMap;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum::Server;

async fn graphql_handler(
    schema: Extension<AstronautsSchema>,
    token_impl: Extension<TokenImpl>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();

    match headers.get("token") {
        Some(value) => match value.to_str() {
            Ok(val) => match token_impl.fetch_token(val).await {
                Ok(token) => {
                    req = req.data(token);
                }
                Err(_) => {}
            },
            Err(_) => {}
        },
        None => {}
    }

    schema.execute(req).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();

    let unique_pod_id = env::var("UNIQUE_POD_ID").expect("UNIQUE_POD_ID env var not set");
    let kafka_url = env::var("KAFKA_URL").expect("KAFKA_URL env var not set");
    let mongo_url = env::var("MONGO_URL").expect("MONGO_URL env var not set");
    let mongo_database = env::var("MONGO_DATABASE").expect("MONGO_DATABASE env var not set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL env var not set");

    let emitter =
        KafkaEmitterImpl::new(&kafka_url).expect("could not connect to kafka as a producer");
    let listener = KafkaConsumerImpl::new(&kafka_url, &unique_pod_id);
    let state = MongoStateImpl::new(&mongo_url, &mongo_database)
        .await
        .expect("could not connect to mongo");
    let mem_state = RedisMemStateImpl::new(&redis_url).expect("could not connect to redis");

    let token_impl = TokenImpl::new(mem_state.clone());

    let astronaut_commander = AstronautCommander::new(
        emitter.clone(),
        state.clone(),
        mem_state,
        token_impl.clone(),
    );
    let astronaut_querier = AstronautQuerier::new(listener.clone(), state.clone());
    let astronaut_synchronizer = AstronautSynchronizer::new(listener, state);

    tokio::spawn(async move { astronaut_synchronizer.sync_events_to_state().await });

    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(astronaut_commander)
        .data(astronaut_querier)
        // .limit_depth(5) // TODO: add this is production only
        .finish();

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route_service("/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(token_impl))
        .layer(Extension(schema));

    Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
