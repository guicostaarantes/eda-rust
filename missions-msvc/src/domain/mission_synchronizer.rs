use crate::domain::mission_model::CrewDocument;
use crate::domain::mission_model::CrewUpdateDocument;
use crate::domain::mission_model::CrewUpdatedEvent;
use crate::domain::mission_model::MissionCreatedEvent;
use crate::domain::mission_model::MissionDocument;
use crate::domain::mission_model::MissionUpdateDocument;
use crate::domain::mission_model::MissionUpdatedEvent;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::random::RandomImpl;
use crate::providers::state::MongoStateImpl;
use log::error;
use log::info;
use std::sync::Arc;
use thiserror::Error;
use tokio_stream::StreamExt;

#[derive(Debug, Error)]
pub enum MissionSynchronizerError {
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
}

#[derive(Clone)]
pub struct MissionSynchronizer {
    listener: Arc<KafkaConsumerImpl>,
    state: Arc<MongoStateImpl>,
}

impl MissionSynchronizer {
    pub fn new(listener: Arc<KafkaConsumerImpl>, state: Arc<MongoStateImpl>) -> Self {
        Self { listener, state }
    }
}

impl MissionSynchronizer {
    pub async fn sync_events_to_state(&self) {
        let mut stream = self.listener.listen(
            &["mission_created", "mission_updated", "crew_member_updated"],
            "mongo",
        );

        while let Some(r) = stream.next().await {
            match r {
                Ok(re) => match re.topic_index {
                    0 => {
                        let event = match JsonSerializerImpl::deserialize::<MissionCreatedEvent>(
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
                            .find_one_by_field::<MissionDocument>("missions", "name", &event.name)
                            .await
                        {
                            Err(err) => {
                                error!("error inserting mission in state: {}", err);
                                continue;
                            }
                            Ok(Some(_)) => {
                                info!("skipped syncing created mission with id {} since mission with same name already exists", event.id);
                                continue;
                            }
                            _ => {}
                        };

                        match self
                            .state
                            .insert_one("missions", &MissionDocument::from(&event))
                            .await
                        {
                            Err(err) => {
                                error!("error creating mission in state: {}", err);
                                continue;
                            }
                            Ok(_) => {}
                        };
                    }
                    1 => {
                        let event = match JsonSerializerImpl::deserialize::<MissionUpdatedEvent>(
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
                                "missions",
                                "_id",
                                &event.id,
                                &MissionUpdateDocument::from(&event),
                            )
                            .await
                        {
                            Err(err) => {
                                error!("error updating mission in state: {}", err);
                                continue;
                            }
                            Ok(_) => {}
                        };
                    }
                    2 => {
                        let event = match JsonSerializerImpl::deserialize::<CrewUpdatedEvent>(
                            &re.message.get_payload(),
                        ) {
                            Ok(payload) => payload,
                            Err(err) => {
                                error!("error deserializing payload: {}", err);
                                continue;
                            }
                        };

                        let doc_id = match self
                            .state
                            .find_one_by_two_fields::<CrewDocument>(
                                "crew",
                                "mission_id",
                                &event.mission_id,
                                "astronaut_id",
                                &event.astronaut_id,
                            )
                            .await
                        {
                            Err(err) => {
                                error!("error updating crew in state: {}", err);
                                continue;
                            }
                            Ok(None) => None,
                            Ok(Some(doc)) => Some(doc.id),
                        };

                        if let Some(doc_id) = doc_id {
                            if event.roles.len() == 0 {
                                match self
                                    .state
                                    .delete_one_by_id::<CrewDocument>("crew", &doc_id)
                                    .await
                                {
                                    Err(err) => {
                                        error!("error removing crew in state: {}", err);
                                        continue;
                                    }
                                    Ok(_) => {}
                                };
                            } else {
                                match self
                                    .state
                                    .update_one(
                                        "crew",
                                        "_id",
                                        &doc_id,
                                        &CrewUpdateDocument {
                                            roles: event.roles.clone(),
                                        },
                                    )
                                    .await
                                {
                                    Err(err) => {
                                        error!("error updating crew in state: {}", err);
                                        continue;
                                    }
                                    Ok(_) => {}
                                };
                            }
                        } else {
                            match self
                                .state
                                .insert_one(
                                    "crew",
                                    &CrewDocument {
                                        id: RandomImpl::uuid(),
                                        mission_id: event.mission_id.clone(),
                                        astronaut_id: event.astronaut_id.clone(),
                                        roles: event.roles.clone(),
                                    },
                                )
                                .await
                            {
                                Err(err) => {
                                    error!("error adding to crew in state: {}", err);
                                    continue;
                                }
                                Ok(_) => {}
                            };
                        }
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
