use crate::domain::mission_model::CreateMissionInput;
use crate::domain::mission_model::CrewMemberAddedEvent;
use crate::domain::mission_model::MissionCreatedEvent;
use crate::domain::mission_model::MissionDocument;
use crate::domain::mission_model::MissionUpdatedEvent;
use crate::domain::mission_model::UpdateMissionInput;
use crate::domain::token_model::AccessTokenPayload;
use crate::domain::token_model::Permission;
use crate::providers::emitter::KafkaEmitterImpl;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::random::RandomImpl;
use crate::providers::state::MongoStateImpl;
use crate::providers::token::JwtTokenImpl;
use crate::providers::token::TokenImplError;
use log::error;
use log::info;
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
    token: Arc<JwtTokenImpl>,
}

impl MissionCommander {
    pub fn new(
        emitter: Arc<KafkaEmitterImpl>,
        state: Arc<MongoStateImpl>,
        token: Arc<JwtTokenImpl>,
    ) -> Self {
        Self {
            emitter,
            state,
            token,
        }
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

        info!("creating mission with id {}", id);

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

        let event = CrewMemberAddedEvent {
            mission_id: id.clone(),
            astronaut_id: token.astronaut_id,
        };

        let payload = JsonSerializerImpl::serialize(&event)?;
        self.emitter
            .emit("crew_member_added", &id, &payload)
            .await?;

        info!("mission created with id {}", id);

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
        info!("updating mission with id {}", id);

        let mission = match self
            .state
            .find_one_by_id::<MissionDocument>("missions", &id)
            .await
        {
            Err(err) => Err(MissionCommanderError::StateImplError(err)),
            Ok(None) => Err(MissionCommanderError::MissionNotFound),
            Ok(Some(a)) => Ok(a),
        }?;

        if token.permissions.contains(&Permission::GetAnyMission) {
            Ok(())
        } else if token
            .permissions
            .contains(&Permission::GetMissionIfCrewMember)
        {
            if mission
                .crew
                .iter()
                .any(|ast_id| ast_id == &token.astronaut_id)
            {
                Ok(())
            } else {
                Err(MissionCommanderError::Forbidden)
            }
        } else {
            Err(MissionCommanderError::Forbidden)
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

        info!("mission updated with id {}", id);

        Ok(())
    }
}
