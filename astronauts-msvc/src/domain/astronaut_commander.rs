use crate::domain::astronaut_model::AstronautCreatedEvent;
use crate::domain::astronaut_model::AstronautDocument;
use crate::domain::astronaut_model::AstronautUpdatedEvent;
use crate::domain::astronaut_model::CreateAstronautInput;
use crate::domain::astronaut_model::GetAstronautCredentialsInput;
use crate::domain::astronaut_model::UpdateAstronautInput;
use crate::providers::emitter::KafkaEmitterImpl;
use crate::providers::hash::HashImpl;
use crate::providers::hash::HashImplError;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::mem_state::RedisMemStateImpl;
use crate::providers::random::RandomImpl;
use crate::providers::state::MongoStateImpl;
use log::error;
use log::info;
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
    MemStateImplError(#[from] redis::RedisError),
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
    #[error("astronaut not found")]
    AstronautNotFound,
    #[error("astronaut with same name already exists")]
    AstronautWithNameExists,
    #[error("no fields to update")]
    NoFieldsToUpdate,
    #[error("password does not match")]
    PasswordDoesNotMatch,
}

#[derive(Clone)]
pub struct AstronautCommander {
    emitter: KafkaEmitterImpl,
    state: MongoStateImpl,
    mem_state: RedisMemStateImpl,
}

impl AstronautCommander {
    pub fn new(
        emitter: KafkaEmitterImpl,
        state: MongoStateImpl,
        mem_state: RedisMemStateImpl,
    ) -> Self {
        Self {
            emitter,
            state,
            mem_state,
        }
    }
}

impl AstronautCommander {
    pub async fn create_astronaut(
        &self,
        input: CreateAstronautInput,
    ) -> Result<String, AstronautCommanderError> {
        let id = uuid::Uuid::new_v4().to_string();

        info!("creating astronaut with id {}", id);

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

        info!("astronaut created with id {}", id);

        Ok(id)
    }
}

impl AstronautCommander {
    pub async fn update_astronaut(
        &self,
        id: String,
        input: UpdateAstronautInput,
    ) -> Result<String, AstronautCommanderError> {
        info!("updating astronaut with id {}", id);

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

        info!("astronaut updated with id {}", id);

        Ok(id)
    }
}

impl AstronautCommander {
    pub async fn get_astronaut_credentials(
        &self,
        input: GetAstronautCredentialsInput,
    ) -> Result<String, AstronautCommanderError> {
        let astronaut = match self
            .state
            .find_one_by_field::<AstronautDocument>("astronauts", "name", &input.name)
            .await
        {
            Err(err) => Err(AstronautCommanderError::StateImplError(err)),
            Ok(None) => Err(AstronautCommanderError::AstronautNotFound),
            Ok(Some(astronaut)) => Ok(astronaut),
        }?;

        match HashImpl::verify(&input.password, &astronaut.password) {
            Ok(_) => Ok(()),
            Err(_) => Err(AstronautCommanderError::PasswordDoesNotMatch),
        }?;

        let token = RandomImpl::string(64);

        self.mem_state
            .set(&token, &astronaut.id, Some(3600))
            .await?;

        Ok(token)
    }
}
