use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_model::AstronautCreatedEvent;
use crate::domain::astronaut_model::AstronautUpdatedEvent;
use crate::domain::astronaut_model::CreateAstronautInput;
use crate::domain::astronaut_model::UpdateAstronautInput;
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
    #[error("astronaut not found")]
    AstronautNotFound,
    #[error("astronaut with same name already exists")]
    AstronautWithNameExists,
    #[error("no fields to update")]
    NoFieldsToUpdate,
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
        input: CreateAstronautInput,
    ) -> Result<String, AstronautControllerError> {
        match self
            .state
            .find_one_by_field::<Astronaut>("astronauts", "name", &input.name)
            .await
        {
            Err(err) => Err(AstronautControllerError::StateImplError(err)),
            Ok(Some(_)) => Err(AstronautControllerError::AstronautWithNameExists),
            _ => Ok(()),
        }?;

        let id = uuid::Uuid::new_v4().to_string();
        let event = AstronautCreatedEvent {
            id: id.clone(),
            name: input.name,
            birth_date: input.birth_date,
        };

        let payload = JsonSerializerImpl::serialize(&event)?;
        self.emitter
            .emit("astronaut_created", &id, &payload)
            .await?;
        Ok(id)
    }
}

impl AstronautController {
    pub async fn update_astronaut(
        &self,
        id: String,
        input: UpdateAstronautInput,
    ) -> Result<String, AstronautControllerError> {
        match self
            .state
            .find_one_by_id::<Astronaut>("astronauts", &id)
            .await
        {
            Err(err) => Err(AstronautControllerError::StateImplError(err)),
            Ok(None) => Err(AstronautControllerError::AstronautNotFound),
            Ok(Some(a)) => Ok(a),
        }?;

        if input.is_empty() {
            return Err(AstronautControllerError::NoFieldsToUpdate);
        }

        if let Some(name) = &input.name {
            match self
                .state
                .find_one_by_field::<Astronaut>("astronauts", "name", &name)
                .await
            {
                Err(err) => Err(AstronautControllerError::StateImplError(err)),
                Ok(Some(_)) => Err(AstronautControllerError::AstronautWithNameExists),
                _ => Ok(()),
            }?;
        }

        let event = AstronautUpdatedEvent {
            id: id.clone(),
            name: input.name,
            birth_date: input.birth_date,
        };
        let payload = JsonSerializerImpl::serialize(&event)?;

        self.emitter
            .emit("astronaut_updated", &id, &payload)
            .await?;

        Ok(id)
    }
}

impl AstronautController {
    pub async fn sync_events_to_state(&self) {
        let mut stream1 = self.listener.listen("astronaut_created", "mongo");
        let mut stream2 = self.listener.listen("astronaut_updated", "mongo");

        // TODO: this must change
        // it should compare all the results from diff streams and apply them in timestamp order
        loop {
            let results = tokio::join!(
                tokio::time::timeout(std::time::Duration::from_millis(1000), stream1.next()),
                tokio::time::timeout(std::time::Duration::from_millis(1000), stream2.next()),
            );

            match results.0 {
                Ok(Some(Ok(value))) => {
                    let event =
                        JsonSerializerImpl::deserialize::<AstronautCreatedEvent>(&value).unwrap();

                    let astronaut = Astronaut {
                        id: event.id,
                        name: event.name,
                        birth_date: event.birth_date,
                    };

                    match self.state.insert_one("astronauts", &astronaut).await {
                        Err(err) => println!("Error inserting astronaut in state: {}", err),
                        Ok(_) => {}
                    };
                }
                Ok(Some(Err(err))) => {
                    println!("Error in broadcast stream: {}", err);
                }
                _ => {}
            }

            match results.1 {
                Ok(Some(Ok(value))) => {
                    let event =
                        JsonSerializerImpl::deserialize::<AstronautUpdatedEvent>(&value).unwrap();
                    match self
                        .state
                        .update_one("astronauts", "_id", &event.id, &event)
                        .await
                    {
                        Err(err) => println!("Error updating astronaut in state: {}", err),
                        Ok(_) => {}
                    };
                }
                Ok(Some(Err(err))) => {
                    println!("Error in broadcast stream: {}", err);
                }
                _ => {}
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
