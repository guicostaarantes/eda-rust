use crate::domain::mission_model::Astronaut;
use crate::domain::mission_model::AstronautDocument;
use crate::domain::mission_model::CrewMemberAddedEvent;
use crate::domain::mission_model::CrewMemberRemovedEvent;
use crate::domain::mission_model::Mission;
use crate::domain::mission_model::MissionDocument;
use crate::domain::mission_model::MissionUpdatedEvent;
use crate::domain::token_model::AccessTokenPayload;
use crate::domain::token_model::Permission;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::state::MongoStateImpl;
use crate::providers::token::JwtTokenImpl;
use crate::providers::token::RawToken;
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
    #[error("tokio mpsc send error")]
    MpscChannelSendError,
    #[error("mission not found")]
    MissionNotFound,
    #[error("astronaut not found")]
    AstronautNotFound,
    #[error("forbidden")]
    Forbidden,
}

#[derive(Clone)]
pub struct MissionQuerier {
    listener: Arc<KafkaConsumerImpl>,
    state: Arc<MongoStateImpl>,
    token: Arc<JwtTokenImpl>,
}

impl MissionQuerier {
    pub fn new(
        listener: Arc<KafkaConsumerImpl>,
        state: Arc<MongoStateImpl>,
        token: Arc<JwtTokenImpl>,
    ) -> Self {
        Self {
            listener,
            state,
            token,
        }
    }
}

impl MissionQuerier {
    pub async fn get_mission_by_id(
        &self,
        raw_token: &RawToken,
        id: String,
    ) -> Result<Mission, MissionQuerierError> {
        let token = match self.token.validate_token::<AccessTokenPayload>(raw_token) {
            Ok(token) => Ok(token),
            Err(_) => Err(MissionQuerierError::Forbidden),
        }?;

        let mission_doc = match self
            .state
            .find_one_by_id::<MissionDocument>("missions", &id)
            .await
        {
            Err(err) => Err(MissionQuerierError::StateImplError(err)),
            Ok(None) => Err(MissionQuerierError::MissionNotFound),
            Ok(Some(a)) => Ok(a),
        }?;

        if token.permissions.contains(&Permission::GetAnyMission) {
            Ok(())
        } else if token
            .permissions
            .contains(&Permission::GetMissionIfCrewMember)
        {
            if mission_doc
                .crew
                .iter()
                .any(|ast_id| ast_id == &token.astronaut_id)
            {
                Ok(())
            } else {
                Err(MissionQuerierError::Forbidden)
            }
        } else {
            Err(MissionQuerierError::Forbidden)
        }?;

        let mission = Mission::from(&mission_doc);

        Ok(mission)
    }
}

impl MissionQuerier {
    pub async fn get_mission_by_id_stream(
        &self,
        raw_token: &RawToken,
        id: String,
    ) -> Result<impl Stream<Item = Mission>, MissionQuerierError> {
        let (tx, rx) = mpsc::channel(1);
        let tx2 = tx.clone();
        let (kill_tx, kill_rx) = oneshot::channel();

        let mut mission = match self.get_mission_by_id(raw_token, id.clone()).await {
            Ok(astro) => match tx.send(astro.clone()).await {
                Ok(_) => Ok(astro),
                Err(err) => {
                    error!("error updating stream for get_mission_by_id: {}", err);
                    Err(MissionQuerierError::MpscChannelSendError)
                }
            },
            Err(err) => Err(err),
        }?;

        let mut stream = self.listener.listen_multiple(
            &[
                "mission_updated",
                "crew_member_added",
                "crew_member_removed",
            ],
            "mongo",
        );

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

                            if event.id != id {
                                continue;
                            };

                            mission = mission.apply_mission_updated_event(&event);

                            match tx.send(mission.clone()).await {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("error updating stream for get_mission_by_id: {}", err);
                                }
                            };
                        }
                        1 => {
                            let event = match JsonSerializerImpl::deserialize::<CrewMemberAddedEvent>(
                                &re.message.get_payload(),
                            ) {
                                Ok(payload) => payload,
                                Err(err) => {
                                    error!("error deserializing payload: {}", err);
                                    continue;
                                }
                            };

                            if event.mission_id != id {
                                continue;
                            };

                            mission = mission.apply_crew_member_added_event(&event);

                            match tx.send(mission.clone()).await {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("error updating stream for get_mission_by_id: {}", err);
                                }
                            };
                        }
                        2 => {
                            let event =
                                match JsonSerializerImpl::deserialize::<CrewMemberRemovedEvent>(
                                    &re.message.get_payload(),
                                ) {
                                    Ok(payload) => payload,
                                    Err(err) => {
                                        error!("error deserializing payload: {}", err);
                                        continue;
                                    }
                                };

                            if event.mission_id != id {
                                continue;
                            };

                            mission = mission.apply_crew_member_removed_event(&event);

                            match tx.send(mission.clone()).await {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("error updating stream for get_mission_by_id: {}", err);
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

        let task2 = match self.token.get_token_seconds_remaining(raw_token) {
            Ok(seconds) => Ok(tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
                match kill_tx.send(()) {
                    Err(_) => error!("error sending kill signal after token expired"),
                    _ => {}
                };
            })),
            Err(_) => Err(MissionQuerierError::Forbidden),
        }?;

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

impl MissionQuerier {
    pub async fn get_astronaut_by_id(
        &self,
        raw_token: &RawToken,
        id: String,
    ) -> Result<Astronaut, MissionQuerierError> {
        let token = match self.token.validate_token::<AccessTokenPayload>(raw_token) {
            Ok(token) => Ok(token),
            Err(_) => Err(MissionQuerierError::Forbidden),
        }?;

        let astronaut_doc = match self
            .state
            .find_one_by_id::<AstronautDocument>("astronauts", &id)
            .await
        {
            Err(err) => Err(MissionQuerierError::StateImplError(err)),
            Ok(None) => Err(MissionQuerierError::AstronautNotFound),
            Ok(Some(a)) => Ok(a),
        }?;

        if token.permissions.contains(&Permission::GetAnyAstronaut)
            || token
                .permissions
                .contains(&Permission::GetAstronautIfCoCrew)
        {
            Ok(())
        } else {
            Err(MissionQuerierError::Forbidden)
        }?;

        let astronaut = Astronaut::from(&astronaut_doc);

        Ok(astronaut)
    }
}
