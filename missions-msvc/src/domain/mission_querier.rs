use crate::domain::mission_model::AstronautCrewInfo;
use crate::domain::mission_model::CrewDocument;
use crate::domain::mission_model::CrewUpdatedEvent;
use crate::domain::mission_model::Mission;
use crate::domain::mission_model::MissionCrewInfo;
use crate::domain::mission_model::MissionDocument;
use crate::domain::mission_model::MissionUpdatedEvent;
use crate::domain::token_model::AccessTokenPayload;
use crate::domain::token_model::Permission;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::state::MongoStateImpl;
use log::error;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

#[derive(Debug, Error)]
pub enum MissionQuerierError {
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
    #[error("mission not found")]
    MissionNotFound,
    #[error("forbidden")]
    Forbidden,
}

#[derive(Clone)]
pub struct MissionQuerier {
    listener: Arc<KafkaConsumerImpl>,
    state: Arc<MongoStateImpl>,
}

impl MissionQuerier {
    pub fn new(listener: Arc<KafkaConsumerImpl>, state: Arc<MongoStateImpl>) -> Self {
        Self { listener, state }
    }
}

impl MissionQuerier {
    pub async fn get_mission_by_id(
        &self,
        token: AccessTokenPayload,
        id: String,
    ) -> Result<Mission, MissionQuerierError> {
        let is_allowed = token.permissions.contains(&Permission::GetMission);

        if !is_allowed {
            return Err(MissionQuerierError::Forbidden);
        };

        let mission_doc = match self
            .state
            .find_one_by_id::<MissionDocument>("missions", &id)
            .await
        {
            Err(err) => Err(MissionQuerierError::StateImplError(err)),
            Ok(None) => Err(MissionQuerierError::MissionNotFound),
            Ok(Some(a)) => Ok(a),
        }?;

        let mission = Mission::from(&mission_doc);

        Ok(mission)
    }
}

impl MissionQuerier {
    pub async fn get_mission_crew_info(
        &self,
        token: AccessTokenPayload,
        id: String,
    ) -> Result<MissionCrewInfo, MissionQuerierError> {
        let is_allowed = token.permissions.contains(&Permission::GetMission);

        if !is_allowed {
            return Err(MissionQuerierError::Forbidden);
        };

        let crews = self
            .state
            .find_all_by_field::<CrewDocument>("crew", "mission_id", &id)
            .await?;

        Ok(MissionCrewInfo::from(&crews))
    }
}

impl MissionQuerier {
    pub async fn get_astronaut_crew_info(
        &self,
        token: AccessTokenPayload,
        id: String,
    ) -> Result<AstronautCrewInfo, MissionQuerierError> {
        let is_allowed = token.permissions.contains(&Permission::GetAstronaut);

        if !is_allowed {
            return Err(MissionQuerierError::Forbidden);
        };

        let crews = self
            .state
            .find_all_by_field::<CrewDocument>("crew", "astronaut_id", &id)
            .await?;

        Ok(AstronautCrewInfo::from(&crews))
    }
}

impl MissionQuerier {
    pub async fn missions_or_crew_updated_stream(
        &self,
        token: AccessTokenPayload,
        ttl: u64,
        ids: Vec<String>,
    ) -> Result<impl Stream<Item = String>, MissionQuerierError> {
        let is_allowed = token.permissions.contains(&Permission::GetMission);

        if !is_allowed {
            return Err(MissionQuerierError::Forbidden);
        };

        let (tx, rx) = mpsc::channel(1);
        let tx2 = tx.clone();
        let (kill_tx, kill_rx) = oneshot::channel();

        let mut stream = self
            .listener
            .listen(&["mission_updated", "crew_member_updated"], "sse");

        let task = tokio::spawn(async move {
            while let Some(r) = stream.next().await {
                match r {
                    Ok(re) => match re.topic_index {
                        0 => {
                            let event = match JsonSerializerImpl::deserialize::<MissionUpdatedEvent>(
                                &re.message.get_payload(),
                            ) {
                                Ok(payload) => payload,
                                Err(err) => {
                                    error!("error deserializing payload: {}", err);
                                    continue;
                                }
                            };

                            // just update in case it's a relevant id
                            if !ids.contains(&event.id) {
                                continue;
                            };

                            match tx.send(event.id).await {
                                Ok(_) => {}
                                Err(err) => {
                                    error!(
                                        "error updating missions_or_crew_updated_stream: {}",
                                        err
                                    );
                                }
                            };
                        }
                        1 => {
                            let event = match JsonSerializerImpl::deserialize::<CrewUpdatedEvent>(
                                &re.message.get_payload(),
                            ) {
                                Ok(payload) => payload,
                                Err(err) => {
                                    error!("error deserializing payload: {}", err);
                                    continue;
                                }
                            };

                            // just update in case it's a relevant id
                            if !ids.contains(&event.mission_id) {
                                continue;
                            };

                            match tx.send(event.mission_id).await {
                                Ok(_) => {}
                                Err(err) => {
                                    error!(
                                        "error updating missions_or_crew_updated_stream: {}",
                                        err
                                    );
                                }
                            };
                        }
                        _ => {
                            error!("unsupported case");
                        }
                    },
                    Err(err) => error!("error in mpsc stream: {}", err),
                }
            }
        });

        let task2 = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(ttl)).await;
            match kill_tx.send(()) {
                Err(_) => error!("error sending kill signal after token expired"),
                _ => {}
            };
        });

        tokio::spawn(async move {
            tokio::select! {
                _ = tx2.closed() => {
                    task.abort();
                    task2.abort();
                }
                _ = kill_rx => {
                    task.abort();
                }
            }
        });

        Ok(ReceiverStream::new(rx))
    }
}
