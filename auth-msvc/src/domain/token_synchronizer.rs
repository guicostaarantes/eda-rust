use crate::domain::token_model::RefreshTokenCreatedEvent;
use crate::domain::token_model::RefreshTokenDocument;
use crate::domain::token_model::RefreshTokenRevokedEvent;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::state::MongoStateImpl;
use log::error;
use std::sync::Arc;
use thiserror::Error;
use tokio_stream::StreamExt;

#[derive(Debug, Error)]
pub enum TokenSynchronizerError {
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
}

#[derive(Clone)]
pub struct TokenSynchronizer {
    listener: Arc<KafkaConsumerImpl>,
    state: Arc<MongoStateImpl>,
}

impl TokenSynchronizer {
    pub fn new(listener: Arc<KafkaConsumerImpl>, state: Arc<MongoStateImpl>) -> Self {
        Self { listener, state }
    }
}

impl TokenSynchronizer {
    pub async fn sync_events_to_state(&self) {
        let mut stream = self
            .listener
            .listen(&["refresh_token_created", "refresh_token_revoked"], "mongo");

        while let Some(r) = stream.next().await {
            match r {
                Ok(re) => match re.topic_index {
                    0 => {
                        let event = match JsonSerializerImpl::deserialize::<RefreshTokenCreatedEvent>(
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
                            .find_one_by_id::<RefreshTokenDocument>("refresh_tokens", &event.id)
                            .await
                        {
                            Err(err) => {
                                error!("error inserting refresh_token in state: {}", err);
                                continue;
                            }
                            Ok(Some(_)) => {
                                match self
                                    .state
                                    .update_one(
                                        "refresh_tokens",
                                        "_id",
                                        &event.id,
                                        &RefreshTokenDocument::from(&event),
                                    )
                                    .await
                                {
                                    Ok(_) => {}
                                    Err(err) => {
                                        error!("error inserting refresh_token in state: {}", err);
                                        continue;
                                    }
                                };
                            }
                            Ok(None) => {
                                // TODO: check if astronaut has more than 5 refresh tokens, clean
                                // the oldest ones. This way we avoid accumulation of refresh
                                // tokens that are likely no longer used, however we limit
                                // astronaut access to only 5 devices.
                                match self
                                    .state
                                    .insert_one(
                                        "refresh_tokens",
                                        &RefreshTokenDocument::from(&event),
                                    )
                                    .await
                                {
                                    Ok(_) => {}
                                    Err(err) => {
                                        error!("error inserting refresh_token in state: {}", err);
                                        continue;
                                    }
                                }
                            }
                        };
                    }
                    1 => {
                        let event = match JsonSerializerImpl::deserialize::<RefreshTokenRevokedEvent>(
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
                            .delete_one_by_id::<RefreshTokenDocument>("refresh_tokens", &event.id)
                            .await
                        {
                            Err(err) => {
                                error!("error deleting refresh_token in state: {}", err);
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
