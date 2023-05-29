mod domain;
mod http;
mod providers;

use crate::domain::mission_commander::MissionCommander;
use crate::domain::mission_querier::MissionQuerier;
use crate::domain::mission_synchronizer::MissionSynchronizer;
use crate::http::missions_route;
use crate::providers::emitter::KafkaEmitterImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::state::MongoStateImpl;
use crate::providers::token::JwtTokenImpl;
use axum::Server;
use std::env;
use std::sync::Arc;

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

    let mission_commander = Arc::new(MissionCommander::new(
        emitter_impl.clone(),
        state_impl.clone(),
    ));
    let mission_querier = Arc::new(MissionQuerier::new(
        listener_impl.clone(),
        state_impl.clone(),
    ));

    let mission_synchronizer = MissionSynchronizer::new(listener_impl.clone(), state_impl.clone());
    tokio::spawn(async move { mission_synchronizer.sync_events_to_state().await });

    let app = missions_route(
        token_impl.clone(),
        mission_querier.clone(),
        mission_commander.clone(),
    );

    Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
