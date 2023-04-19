mod domain;
mod providers;
mod schema;

use crate::domain::astronaut_synchronizer::AstronautSynchronizer;
use crate::domain::token_commander::TokenCommander;
use crate::domain::token_synchronizer::TokenSynchronizer;
use crate::providers::emitter::KafkaEmitterImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::state::MongoStateImpl;
use crate::providers::token::JwtTokenImpl;
use crate::providers::token::RawToken;
use crate::schema::AuthSchema;
use crate::schema::MutationRoot;
use crate::schema::QueryRoot;
use crate::schema::SubscriptionRoot;
use async_graphql::http::playground_source;
use async_graphql::http::GraphQLPlaygroundConfig;
use async_graphql::Schema;
use async_graphql_axum::GraphQLRequest;
use async_graphql_axum::GraphQLResponse;
use axum::extract::Extension;
use axum::headers::HeaderMap;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum::Server;
use std::env;
use std::sync::Arc;

async fn graphql_handler(
    schema: Extension<AuthSchema>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();

    match headers.get("token") {
        Some(value) => match value.to_str() {
            Ok(val) => {
                req = req.data(RawToken(val.to_string()));
            }
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
    let public_keys_pem = env::var("PUBLIC_KEYS_PEM")
        .expect("PUBLIC_KEYS_PEM env var not set")
        .split(',')
        .map(|s| s.to_string().replace("\\n", "\n"))
        .collect();
    let private_key_pem = env::var("PRIVATE_KEY_PEM")
        .expect("PRIVATE_KEY_PEM env var not set")
        .replace("\\n", "\n");

    let emitter_impl = Arc::new(
        KafkaEmitterImpl::new(&kafka_url).expect("could not connect to kafka as a producer"),
    );
    let listener_impl = Arc::new(KafkaConsumerImpl::new(&kafka_url, &unique_pod_id));
    let state_impl = Arc::new(
        MongoStateImpl::new(&mongo_url, &mongo_database)
            .await
            .expect("could not connect to mongo"),
    );
    let token_impl = Arc::new(
        JwtTokenImpl::new(public_keys_pem, private_key_pem).expect("could not import pem keys"),
    );

    let token_commander =
        TokenCommander::new(emitter_impl.clone(), state_impl.clone(), token_impl.clone());

    let astronaut_synchronizer =
        AstronautSynchronizer::new(listener_impl.clone(), state_impl.clone());
    tokio::spawn(async move { astronaut_synchronizer.sync_events_to_state().await });

    let token_synchronizer = TokenSynchronizer::new(listener_impl.clone(), state_impl.clone());
    tokio::spawn(async move { token_synchronizer.sync_events_to_state().await });

    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(token_commander)
        .finish();

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema));

    Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
