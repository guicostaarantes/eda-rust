use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_model::AstronautDocument;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::mem_state::RedisMemStateImpl;
use crate::providers::state::MongoStateImpl;
use log::error;
use thiserror::Error;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

#[derive(Debug, Error)]
pub enum AstronautQuerierError {
    #[error(transparent)]
    MemStateImplError(#[from] redis::RedisError),
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
    #[error("token not found")]
    TokenNotFound,
}

#[derive(Clone)]
pub struct AstronautQuerier {
    listener: KafkaConsumerImpl,
    state: MongoStateImpl,
    mem_state: RedisMemStateImpl,
}

impl AstronautQuerier {
    pub fn new(
        listener: KafkaConsumerImpl,
        state: MongoStateImpl,
        mem_state: RedisMemStateImpl,
    ) -> Self {
        Self {
            listener,
            state,
            mem_state,
        }
    }
}

impl AstronautQuerier {
    pub async fn get_astronaut_by_id(
        &self,
        id: String,
    ) -> Result<Option<Astronaut>, AstronautQuerierError> {
        match self
            .state
            .find_one_by_id::<AstronautDocument>("astronauts", &id)
            .await
        {
            Ok(Some(v)) => Ok(Some(Astronaut::from(&v))),
            Ok(None) => Ok(None),
            Err(err) => Err(AstronautQuerierError::StateImplError(err)),
        }
    }
}

impl AstronautQuerier {
    pub async fn check_astronaut_credentials(
        &self,
        token: String,
    ) -> Result<String, AstronautQuerierError> {
        let astronaut_id = match self.mem_state.get(&token).await {
            Ok(id) => Ok(id),
            Err(err) => match err.kind() {
                redis::ErrorKind::TypeError => Err(AstronautQuerierError::TokenNotFound),
                _ => Err(AstronautQuerierError::MemStateImplError(err)),
            },
        }?;

        Ok(astronaut_id)
    }
}

impl AstronautQuerier {
    pub fn subscribe_to_astronaut_created(&self) -> impl Stream<Item = Astronaut> + '_ {
        self.listener
            .listen("astronaut_created", "graphql")
            .filter_map(|value| match value {
                Ok(value) => {
                    match JsonSerializerImpl::deserialize::<AstronautDocument>(&value.get_payload())
                    {
                        Ok(v) => Some(Astronaut::from(&v)),
                        Err(err) => {
                            error!("error deserializing astronaut: {}", err);
                            None
                        }
                    }
                }
                Err(err) => {
                    error!("error in broadcast stream: {}", err);
                    None
                }
            })
    }
}
