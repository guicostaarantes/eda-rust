use crate::domain::mission_model::CreateMissionInput;
use crate::domain::mission_model::CrewDocument;
use crate::domain::mission_model::CrewUpdatedEvent;
use crate::domain::mission_model::MissionCreatedEvent;
use crate::domain::mission_model::MissionDocument;
use crate::domain::mission_model::MissionRole;
use crate::domain::mission_model::MissionUpdatedEvent;
use crate::domain::mission_model::UpdateCrewInput;
use crate::domain::mission_model::UpdateMissionInput;
use crate::domain::token_model::AccessTokenPayload;
use crate::domain::token_model::Permission;
use crate::providers::emitter::KafkaEmitterImpl;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::random::RandomImpl;
use crate::providers::state::MongoStateImpl;
use crate::providers::token::TokenImplError;
use log::error;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MissionCommanderError {
    #[error(transparent)]
    EmitterImplError(#[from] rdkafka::error::KafkaError),
    #[error(transparent)]
    JsonSerializerImplError(#[from] serde_json::Error),
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
    #[error(transparent)]
    TokenImplError(#[from] TokenImplError),
    #[error("mission not found")]
    MissionNotFound,
    #[error("mission with same name already exists")]
    MissionWithNameExists,
    #[error("no fields to update")]
    NoFieldsToUpdate,
    #[error("forbidden")]
    Forbidden,
}

#[derive(Clone)]
pub struct MissionCommander {
    emitter: Arc<KafkaEmitterImpl>,
    state: Arc<MongoStateImpl>,
}

impl MissionCommander {
    pub fn new(emitter: Arc<KafkaEmitterImpl>, state: Arc<MongoStateImpl>) -> Self {
        Self { emitter, state }
    }
}

impl MissionCommander {
    pub async fn create_mission(
        &self,
        token: AccessTokenPayload,
        input: CreateMissionInput,
    ) -> Result<String, MissionCommanderError> {
        let is_allowed = token.permissions.contains(&Permission::CreateMission);

        if !is_allowed {
            return Err(MissionCommanderError::Forbidden);
        }

        let id = RandomImpl::uuid();

        match self
            .state
            .find_one_by_field::<MissionDocument>("missions", "name", &input.name)
            .await
        {
            Err(err) => Err(MissionCommanderError::StateImplError(err)),
            Ok(Some(_)) => Err(MissionCommanderError::MissionWithNameExists),
            _ => Ok(()),
        }?;

        let event = MissionCreatedEvent {
            id: id.clone(),
            name: input.name,
            start_date: input.start_date,
        };

        let payload = JsonSerializerImpl::serialize(&event)?;
        self.emitter.emit("mission_created", &id, &payload).await?;

        let event = CrewUpdatedEvent {
            mission_id: id.clone(),
            astronaut_id: token.astronaut_id,
            roles: vec![MissionRole::Leader],
        };

        let payload = JsonSerializerImpl::serialize(&event)?;
        self.emitter
            .emit("crew_member_updated", &id, &payload)
            .await?;

        Ok(id)
    }
}

impl MissionCommander {
    pub async fn update_mission(
        &self,
        token: AccessTokenPayload,
        id: String,
        input: UpdateMissionInput,
    ) -> Result<(), MissionCommanderError> {
        let crew_from_token = match self
            .state
            .find_one_by_two_fields::<CrewDocument>(
                "crew",
                "mission_id",
                &id,
                "astronaut_id",
                &token.astronaut_id,
            )
            .await
        {
            Err(err) => Err(MissionCommanderError::StateImplError(err)),
            Ok(None) => Err(MissionCommanderError::Forbidden),
            Ok(Some(m)) => Ok(m),
        }?;

        let is_allowed = token.permissions.contains(&Permission::UpdateMission)
            && crew_from_token.roles.contains(&MissionRole::Leader);

        if !is_allowed {
            return Err(MissionCommanderError::Forbidden);
        }

        match self
            .state
            .find_one_by_id::<MissionDocument>("missions", &id)
            .await
        {
            Err(err) => Err(MissionCommanderError::StateImplError(err)),
            Ok(None) => Err(MissionCommanderError::MissionNotFound),
            Ok(Some(a)) => Ok(a),
        }?;

        if input.is_empty() {
            return Err(MissionCommanderError::NoFieldsToUpdate);
        }

        if let Some(name) = &input.name {
            match self
                .state
                .find_one_by_field::<MissionDocument>("missions", "name", &name)
                .await
            {
                Err(err) => Err(MissionCommanderError::StateImplError(err)),
                Ok(Some(_)) => Err(MissionCommanderError::MissionWithNameExists),
                _ => Ok(()),
            }?;
        }

        let event = MissionUpdatedEvent {
            id: id.clone(),
            name: input.name,
            start_date: input.start_date,
        };

        let payload = JsonSerializerImpl::serialize(&event)?;

        self.emitter.emit("mission_updated", &id, &payload).await?;

        Ok(())
    }
}

impl MissionCommander {
    pub async fn update_crew(
        &self,
        token: AccessTokenPayload,
        input: UpdateCrewInput,
    ) -> Result<(), MissionCommanderError> {
        let crew_from_token = match self
            .state
            .find_one_by_two_fields::<CrewDocument>(
                "crew",
                "mission_id",
                &input.mission_id,
                "astronaut_id",
                &token.astronaut_id,
            )
            .await
        {
            Err(err) => Err(MissionCommanderError::StateImplError(err)),
            Ok(None) => Err(MissionCommanderError::Forbidden),
            Ok(Some(m)) => Ok(m),
        }?;

        let is_allowed = token.permissions.contains(&Permission::UpdateMission)
            && crew_from_token.roles.contains(&MissionRole::Leader);

        if !is_allowed {
            return Err(MissionCommanderError::Forbidden);
        }

        let event = CrewUpdatedEvent {
            mission_id: input.mission_id.clone(),
            astronaut_id: input.astronaut_id.clone(),
            roles: input.roles.clone(),
        };

        let payload = JsonSerializerImpl::serialize(&event)?;
        self.emitter
            .emit("crew_member_updated", &input.mission_id, &payload)
            .await?;

        Ok(())
    }
}
