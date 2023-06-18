use crate::domain::astronaut_model::AstronautCreatedEvent;
use crate::domain::astronaut_model::AstronautDocument;
use crate::domain::astronaut_model::AstronautUpdatedEvent;
use crate::domain::astronaut_model::CreateAstronautInput;
use crate::domain::astronaut_model::UpdateAstronautInput;
use crate::domain::token_model::AccessTokenPayload;
use crate::domain::token_model::Permission;
use crate::providers::emitter::KafkaEmitterImpl;
use crate::providers::hash::HashImpl;
use crate::providers::hash::HashImplError;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::random::RandomImpl;
use crate::providers::state::MongoStateImpl;
use crate::providers::token::TokenImplError;
use log::error;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AstronautCommanderError {
    #[error(transparent)]
    EmitterImplError(#[from] rdkafka::error::KafkaError),
    #[error(transparent)]
    HashImplError(#[from] HashImplError),
    #[error(transparent)]
    JsonSerializerImplError(#[from] serde_json::Error),
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
    #[error(transparent)]
    TokenImplError(#[from] TokenImplError),
    #[error("astronaut not found")]
    AstronautNotFound,
    #[error("astronaut with same name already exists")]
    AstronautWithNameExists,
    #[error("no fields to update")]
    NoFieldsToUpdate,
    #[error("forbidden")]
    Forbidden,
}

#[derive(Clone)]
pub struct AstronautCommander {
    emitter: Arc<KafkaEmitterImpl>,
    state: Arc<MongoStateImpl>,
}

impl AstronautCommander {
    pub fn new(emitter: Arc<KafkaEmitterImpl>, state: Arc<MongoStateImpl>) -> Self {
        Self { emitter, state }
    }
}

impl AstronautCommander {
    pub async fn create_astronaut(
        &self,
        input: CreateAstronautInput,
    ) -> Result<String, AstronautCommanderError> {
        let id = RandomImpl::uuid();

        match self
            .state
            .find_one_by_field::<AstronautDocument>("astronauts", "name", &input.name)
            .await
        {
            Err(err) => Err(AstronautCommanderError::StateImplError(err)),
            Ok(Some(_)) => Err(AstronautCommanderError::AstronautWithNameExists),
            _ => Ok(()),
        }?;

        let hashed_password = HashImpl::hash(&input.password)?;

        let event = AstronautCreatedEvent {
            id: id.clone(),
            name: input.name,
            password: hashed_password,
            birth_date: input.birth_date,
        };

        let payload = JsonSerializerImpl::serialize(&event)?;
        self.emitter
            .emit("astronaut_created", &id, &payload)
            .await?;

        Ok(id)
    }
}

impl AstronautCommander {
    pub async fn update_astronaut(
        &self,
        token: AccessTokenPayload,
        id: String,
        input: UpdateAstronautInput,
    ) -> Result<(), AstronautCommanderError> {
        let is_allowed = token.permissions.contains(&Permission::UpdateAstronaut);

        if !is_allowed {
            return Err(AstronautCommanderError::Forbidden);
        };

        match self
            .state
            .find_one_by_id::<AstronautDocument>("astronauts", &id)
            .await
        {
            Err(err) => Err(AstronautCommanderError::StateImplError(err)),
            Ok(None) => Err(AstronautCommanderError::AstronautNotFound),
            Ok(Some(a)) => Ok(a),
        }?;

        if input.is_empty() {
            return Err(AstronautCommanderError::NoFieldsToUpdate);
        }

        if let Some(name) = &input.name {
            match self
                .state
                .find_one_by_field::<AstronautDocument>("astronauts", "name", &name)
                .await
            {
                Err(err) => Err(AstronautCommanderError::StateImplError(err)),
                Ok(Some(_)) => Err(AstronautCommanderError::AstronautWithNameExists),
                _ => Ok(()),
            }?;
        }

        let hashed_password = match &input.password {
            Some(password) => match HashImpl::hash(password) {
                Ok(hashed) => Ok(Some(hashed)),
                Err(err) => Err(AstronautCommanderError::HashImplError(err)),
            },
            None => Ok(None),
        }?;

        let event = AstronautUpdatedEvent {
            id: id.clone(),
            name: input.name,
            password: hashed_password,
            birth_date: input.birth_date,
        };

        let payload = JsonSerializerImpl::serialize(&event)?;

        self.emitter
            .emit("astronaut_updated", &id, &payload)
            .await?;

        Ok(())
    }
}
