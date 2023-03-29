use crate::domain::astronaut_model::AstronautCreatedEvent;
use crate::domain::astronaut_model::AstronautDocument;
use crate::domain::astronaut_model::AstronautUpdatedDocument;
use crate::domain::astronaut_model::AstronautUpdatedEvent;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::state::MongoStateImpl;
use log::error;
use log::info;
use thiserror::Error;
use tokio_stream::StreamExt;

#[derive(Debug, Error)]
pub enum AstronautSynchronizerError {
    #[error(transparent)]
    MemStateImplError(#[from] redis::RedisError),
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
}

#[derive(Clone)]
pub struct AstronautSynchronizer {
    listener: KafkaConsumerImpl,
    state: MongoStateImpl,
}

impl AstronautSynchronizer {
    pub fn new(listener: KafkaConsumerImpl, state: MongoStateImpl) -> Self {
        Self { listener, state }
    }
}

impl AstronautSynchronizer {
    pub async fn sync_events_to_state(&self) {
        let mut stream = self
            .listener
            .listen_multiple(&["astronaut_created", "astronaut_updated"], "mongo");

        while let Some(r) = stream.next().await {
            match r {
                Ok(re) => match re.topic_index {
                    0 => {
                        let event = match JsonSerializerImpl::deserialize::<AstronautCreatedEvent>(
                            &re.message.get_payload(),
                        ) {
                            Ok(payload) => payload,
                            Err(err) => {
                                error!("error deserializing payload: {}", err);
                                continue;
                            }
                        };

                        match self
                            .state
                            .find_one_by_field::<AstronautDocument>(
                                "astronauts",
                                "name",
                                &event.name,
                            )
                            .await
                        {
                            Err(err) => {
                                error!("error inserting astronaut in state: {}", err);
                                continue;
                            }
                            Ok(Some(_)) => {
                                info!("skipped syncing created astronaut with id {} since user with same name already exists", event.id);
                                continue;
                            }
                            _ => {}
                        };

                        match self
                            .state
                            .insert_one("astronauts", &AstronautDocument::from(&event))
                            .await
                        {
                            Err(err) => {
                                error!("error creating astronaut in state: {}", err);
                                continue;
                            }
                            Ok(_) => {}
                        };
                    }
                    1 => {
                        let event = match JsonSerializerImpl::deserialize::<AstronautUpdatedEvent>(
                            &re.message.get_payload(),
                        ) {
                            Ok(payload) => payload,
                            Err(err) => {
                                error!("error deserializing payload: {}", err);
                                continue;
                            }
                        };

                        match self
                            .state
                            .update_one(
                                "astronauts",
                                "_id",
                                &event.id,
                                &AstronautUpdatedDocument::from(&event),
                            )
                            .await
                        {
                            Err(err) => {
                                error!("error updating astronaut in state: {}", err);
                                continue;
                            }
                            Ok(_) => {}
                        };
                    }
                    _ => {
                        error!("unsupported case");
                    }
                },
                Err(err) => error!("error in broadcast stream: {}", err),
            }
        }
    }
}
