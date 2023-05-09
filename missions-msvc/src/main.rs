mod domain;
mod providers;
mod schema;

use crate::domain::mission_commander::MissionCommander;
use crate::domain::mission_querier::MissionQuerier;
use crate::domain::mission_synchronizer::MissionSynchronizer;
use crate::providers::emitter::KafkaEmitterImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::state::MongoStateImpl;
use crate::providers::token::JwtTokenImpl;
use crate::providers::token::RawToken;
use crate::schema::MissionsSchema;
use crate::schema::Mutation;
use crate::schema::Query;
use crate::schema::Subscription;
use async_graphql::http::playground_source;
use async_graphql::http::GraphQLPlaygroundConfig;
use async_graphql::http::ALL_WEBSOCKET_PROTOCOLS;
use async_graphql::Schema;
use async_graphql_axum::GraphQLProtocol;
use async_graphql_axum::GraphQLRequest;
use async_graphql_axum::GraphQLResponse;
use async_graphql_axum::GraphQLWebSocket;
use axum::extract::Extension;
use axum::extract::WebSocketUpgrade;
use axum::headers::HeaderMap;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use axum::Server;
use std::env;
use std::sync::Arc;

async fn graphql_handler(
    schema: Extension<MissionsSchema>,
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

async fn graphql_ws_handler(
    Extension(schema): Extension<MissionsSchema>,
    protocol: GraphQLProtocol,
    websocket: WebSocketUpgrade,
) -> Response {
    let on_connection_init = |value: serde_json::Value| async move {
        #[derive(serde::Deserialize)]
        struct Payload {
            token: String,
        }

        let mut data = async_graphql::Data::default();

        if let Ok(payload) = serde_json::from_value::<Payload>(value) {
            data.insert(RawToken(payload.token));
        };

        Ok(data)
    };

    websocket
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema.clone(), protocol)
                .on_connection_init(on_connection_init)
                .serve()
        })
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

    let emitter_impl = Arc::new(
        KafkaEmitterImpl::new(&kafka_url).expect("could not connect to kafka as a producer"),
    );
    let listener_impl = Arc::new(KafkaConsumerImpl::new(&kafka_url, &unique_pod_id));
    let state_impl = Arc::new(
        MongoStateImpl::new(&mongo_url, &mongo_database)
            .await
            .expect("could not connect to mongo"),
    );
    let token_impl =
        Arc::new(JwtTokenImpl::new(public_keys_pem).expect("could not import pem keys"));

    let mission_commander =
        MissionCommander::new(emitter_impl.clone(), state_impl.clone(), token_impl.clone());
    let mission_querier = MissionQuerier::new(
        listener_impl.clone(),
        state_impl.clone(),
        token_impl.clone(),
    );

    let mission_synchronizer = MissionSynchronizer::new(listener_impl.clone(), state_impl.clone());
    tokio::spawn(async move { mission_synchronizer.sync_events_to_state().await });

    let schema = Schema::build(Query, Mutation, Subscription)
        .data(mission_commander)
        .data(mission_querier)
        .finish();

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route("/ws", get(graphql_ws_handler))
        .layer(Extension(schema));

    Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
