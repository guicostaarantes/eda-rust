mod domain;
mod http;
mod providers;

use crate::domain::astronaut_synchronizer::AstronautSynchronizer;
use crate::domain::token_commander::TokenCommander;
use crate::domain::token_synchronizer::TokenSynchronizer;
use crate::http::auth_route;
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

    let token_commander = Arc::new(TokenCommander::new(
        emitter_impl.clone(),
        state_impl.clone(),
        token_impl.clone(),
    ));

    let astronaut_synchronizer =
        AstronautSynchronizer::new(listener_impl.clone(), state_impl.clone());
    tokio::spawn(async move { astronaut_synchronizer.sync_events_to_state().await });

    let token_synchronizer = TokenSynchronizer::new(listener_impl.clone(), state_impl.clone());
    tokio::spawn(async move { token_synchronizer.sync_events_to_state().await });

    let app = auth_route(token_impl.clone(), token_commander.clone());

    Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
