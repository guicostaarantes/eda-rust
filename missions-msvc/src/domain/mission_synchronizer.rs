use crate::domain::mission_model::AstronautCreatedEvent;
use crate::domain::mission_model::AstronautDocument;
use crate::domain::mission_model::CrewMemberAddedEvent;
use crate::domain::mission_model::CrewMemberRemoveDocument;
use crate::domain::mission_model::CrewMemberRemovedEvent;
use crate::domain::mission_model::CrewMemberUpdateDocument;
use crate::domain::mission_model::MissionCreatedEvent;
use crate::domain::mission_model::MissionDocument;
use crate::domain::mission_model::MissionUpdateDocument;
use crate::domain::mission_model::MissionUpdatedEvent;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::listener::KafkaConsumerImpl;
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
        let mut stream = self.listener.listen_multiple(
            &[
                "mission_created",
                "mission_updated",
                "crew_member_added",
                "crew_member_removed",
                "astronaut_created",
            ],
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
                        let event = match JsonSerializerImpl::deserialize::<CrewMemberAddedEvent>(
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
                            .update_one_push_to_field(
                                "missions",
                                "_id",
                                &event.mission_id,
                                &CrewMemberUpdateDocument {
                                    crew: Some(event.astronaut_id.clone()),
                                    participant_of: None,
                                },
                            )
                            .await
                        {
                            Err(err) => {
                                error!("error updating mission in state: {}", err);
                                continue;
                            }
                            Ok(_) => {}
                        };

                        match self
                            .state
                            .update_one_push_to_field(
                                "astronauts",
                                "_id",
                                &event.astronaut_id,
                                &CrewMemberUpdateDocument {
                                    crew: None,
                                    participant_of: Some(event.mission_id.clone()),
                                },
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
                    3 => {
                        let event = match JsonSerializerImpl::deserialize::<CrewMemberRemovedEvent>(
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
                            .update_one_pull_from_field(
                                "missions",
                                "_id",
                                &event.mission_id,
                                &CrewMemberRemoveDocument {
                                    crew: Some(event.astronaut_id.clone()),
                                    participant_of: None,
                                },
                            )
                            .await
                        {
                            Err(err) => {
                                error!("error updating mission in state: {}", err);
                                continue;
                            }
                            Ok(_) => {}
                        };

                        match self
                            .state
                            .update_one_pull_from_field(
                                "astronauts",
                                "_id",
                                &event.mission_id,
                                &CrewMemberRemoveDocument {
                                    crew: None,
                                    participant_of: Some(event.mission_id.clone()),
                                },
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
                    4 => {
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
                    _ => {
                        error!("unsupported case");
                    }
                },
                Err(err) => error!("error in broadcast stream: {}", err),
            }
        }
    }
}
