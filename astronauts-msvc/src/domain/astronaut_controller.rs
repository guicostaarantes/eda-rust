use crate::domain::Astronaut;
use crate::providers::emitter::KafkaEmitterImpl;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::state::MongoStateImpl;
use thiserror::Error;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

#[derive(Debug, Error)]
pub enum AstronautControllerError {
    #[error(transparent)]
    EmitterImplError(#[from] rdkafka::error::KafkaError),
    #[error(transparent)]
    JsonSerializerImplError(#[from] serde_json::Error),
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
    #[error("astronaut with same name already exists")]
    AstronautWithNameExists,
}

#[derive(Clone)]
pub struct AstronautController {
    emitter: KafkaEmitterImpl,
    listener: KafkaConsumerImpl,
    state: MongoStateImpl,
}

impl AstronautController {
    pub fn new(
        emitter: KafkaEmitterImpl,
        listener: KafkaConsumerImpl,
        state: MongoStateImpl,
    ) -> Self {
        Self {
            emitter,
            listener,
            state,
        }
    }
}

impl AstronautController {
    pub async fn get_astronaut_by_id(
        &self,
        id: String,
    ) -> Result<Option<Astronaut>, AstronautControllerError> {
        match self
            .state
            .find_one_by_id::<Astronaut>("astronauts", &id)
            .await
        {
            Ok(result) => Ok(result),
            Err(err) => Err(AstronautControllerError::StateImplError(err)),
        }
    }
}

impl AstronautController {
    pub async fn create_astronaut(
        &self,
        astronaut: Astronaut,
    ) -> Result<Astronaut, AstronautControllerError> {
        let name = astronaut.get_name();
        match self
            .state
            .find_one_by_field::<Astronaut>("astronauts", "name", &name)
            .await
        {
            Err(err) => Err(AstronautControllerError::StateImplError(err)),
            Ok(Some(_)) => Err(AstronautControllerError::AstronautWithNameExists),
            _ => Ok(()),
        }?;

        let id = astronaut.get_id();
        let payload = JsonSerializerImpl::serialize(&astronaut)?;
        self.emitter
            .emit("astronaut_created", &id, &payload)
            .await?;
        Ok(astronaut)
    }
}

impl AstronautController {
    pub async fn sync_events_to_state(&self) {
        let mut stream = self.listener.listen("astronaut_created", "mongo");

        while let Some(value) = stream.next().await {
            match value {
                Ok(val) => {
                    let astronaut = JsonSerializerImpl::deserialize::<Astronaut>(&val).unwrap();
                    match self.state.insert_one("astronauts", &astronaut).await {
                        Err(err) => println!("Error inserting astronaut in state: {}", err),
                        Ok(_) => {}
                    };
                }
                Err(err) => println!("Error in broadcast stream: {}", err),
            }
        }
    }
}

impl AstronautController {
    pub fn subscribe_to_astronaut_created(&self) -> impl Stream<Item = Astronaut> + '_ {
        self.listener
            .listen("astronaut_created", "graphql")
            .filter_map(|value| match value {
                Ok(val) => match JsonSerializerImpl::deserialize::<Astronaut>(&val) {
                    Ok(v) => Some(v),
                    Err(err) => {
                        println!("Error deserializing astronaut: {}", err);
                        None
                    }
                },
                Err(err) => {
                    println!("Error in broadcast stream: {}", err);
                    None
                }
            })
    }
}
