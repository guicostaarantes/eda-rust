mod domain;
mod providers;
mod schema;

use std::env;

use crate::schema::AstronautsSchema;
use crate::schema::MutationRoot;
use crate::schema::QueryRoot;
use async_graphql::http::GraphiQLSource;
use async_graphql::Schema;
use async_graphql_axum::GraphQLRequest;
use async_graphql_axum::GraphQLResponse;
use async_graphql_axum::GraphQLSubscription;
use axum::extract::Extension;
use axum::response;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum::Server;
use domain::astronaut_controller::AstronautController;
use providers::emitter::KafkaEmitterImpl;
use providers::listener::KafkaConsumerImpl;
use providers::state::MongoStateImpl;
use schema::SubscriptionRoot;

async fn graphql_handler(
    schema: Extension<AstronautsSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .finish(),
    )
}

#[tokio::main]
async fn main() {
    let kafka_url = env::var("KAFKA_URL").expect("KAFKA_URL env var not set");
    let unique_pod_id = env::var("UNIQUE_POD_ID").expect("UNIQUE_POD_ID env var not set");
    let mongo_url = env::var("MONGO_URL").expect("MONGO_URL env var not set");
    let mongo_database = env::var("MONGO_DATABASE").expect("MONGO_DATABASE env var not set");

    let emitter =
        KafkaEmitterImpl::new(&kafka_url).expect("could not connect to kafka as a producer");
    let listener = KafkaConsumerImpl::new(&kafka_url, &unique_pod_id);
    let state = MongoStateImpl::new(&mongo_url, &mongo_database)
        .await
        .expect("could not connect to mongo");
    let astronaut_controller = AstronautController::new(emitter, listener, state);

    let astronaut_controller_sync = astronaut_controller.clone();
    tokio::spawn(async move { astronaut_controller_sync.sync_events_to_state().await });

    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(astronaut_controller)
        // .limit_depth(5) // TODO: add this is production only
        .finish();

    let app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .route_service("/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema));

    Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
