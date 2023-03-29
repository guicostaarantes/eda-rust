use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_model::AstronautDocument;
use crate::domain::astronaut_model::AstronautUpdatedEvent;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::state::MongoStateImpl;
use log::error;
use thiserror::Error;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

#[derive(Debug, Error)]
pub enum AstronautQuerierError {
    #[error(transparent)]
    MemStateImplError(#[from] redis::RedisError),
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
    #[error("tokio mpsc send error")]
    MpscChannelSendError,
    #[error("astronaut not found")]
    AstronautNotFound,
    #[error("token not found")]
    TokenNotFound,
}

#[derive(Clone)]
pub struct AstronautQuerier {
    listener: KafkaConsumerImpl,
    state: MongoStateImpl,
}

impl AstronautQuerier {
    pub fn new(listener: KafkaConsumerImpl, state: MongoStateImpl) -> Self {
        Self { listener, state }
    }
}

impl AstronautQuerier {
    pub async fn get_astronaut_by_id(
        &self,
        id: String,
    ) -> Result<Astronaut, AstronautQuerierError> {
        match self
            .state
            .find_one_by_id::<AstronautDocument>("astronauts", &id)
            .await
        {
            Ok(Some(v)) => Ok(Astronaut::from(&v)),
            Ok(None) => Err(AstronautQuerierError::AstronautNotFound),
            Err(err) => Err(AstronautQuerierError::StateImplError(err)),
        }
    }
}

impl AstronautQuerier {
    pub async fn get_astronaut_by_id_stream(
        &self,
        id: String,
    ) -> Result<impl Stream<Item = Astronaut>, AstronautQuerierError> {
        let (tx, rx) = channel(8);

        let mut astronaut = match self.get_astronaut_by_id(id).await {
            Ok(astro) => match tx.send(astro.clone()).await {
                Ok(_) => Ok(astro),
                Err(err) => {
                    error!("error updating stream for get_astronaut_by_id: {}", err);
                    Err(AstronautQuerierError::MpscChannelSendError)
                }
            },
            Err(err) => Err(err),
        }?;

        let mut stream = self
            .listener
            .listen_multiple(&["astronaut_updated"], "mongo");

        tokio::spawn(async move {
            while let Some(r) = stream.next().await {
                match r {
                    Ok(re) => match re.topic_index {
                        0 => {
                            let event = match JsonSerializerImpl::deserialize::<AstronautUpdatedEvent>(
                                &re.message.get_payload(),
                            ) {
                                Ok(payload) => payload,
                                Err(err) => {
                                    error!("error deserializing payload: {}", err);
                                    continue;
                                }
                            };

                            astronaut = astronaut.apply_update_event(&event);

                            match tx.send(astronaut.clone()).await {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("error updating stream for get_astronaut_by_id: {}", err)
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

        Ok(ReceiverStream::new(rx))
    }
}
