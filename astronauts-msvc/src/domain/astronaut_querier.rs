use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_model::AstronautDocument;
use crate::domain::astronaut_model::AstronautUpdatedEvent;
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
pub enum AstronautQuerierError {
    #[error(transparent)]
    StateImplError(#[from] mongodb::error::Error),
    #[error("tokio mpsc send error")]
    MpscChannelSendError,
    #[error("astronaut not found")]
    AstronautNotFound,
    #[error("forbidden")]
    Forbidden,
}

#[derive(Clone)]
pub struct AstronautQuerier {
    listener: Arc<KafkaConsumerImpl>,
    state: Arc<MongoStateImpl>,
    token: Arc<JwtTokenImpl>,
}

impl AstronautQuerier {
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

impl AstronautQuerier {
    pub async fn get_astronaut_by_id(
        &self,
        raw_token: &RawToken,
        id: String,
    ) -> Result<Astronaut, AstronautQuerierError> {
        match self.token.validate_token::<AccessTokenPayload>(raw_token) {
            Ok(token) => {
                if token.permissions.contains(&Permission::GetAnyAstronaut)
                    || (token.permissions.contains(&Permission::GetOwnAstronaut)
                        && token.astronaut_id == id)
                {
                    Ok(())
                } else {
                    Err(AstronautQuerierError::Forbidden)
                }
            }
            Err(_) => Err(AstronautQuerierError::Forbidden),
        }?;

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
        raw_token: &RawToken,
        id: String,
    ) -> Result<impl Stream<Item = Astronaut>, AstronautQuerierError> {
        let (tx, rx) = mpsc::channel(1);
        let tx2 = tx.clone();
        let (kill_tx, kill_rx) = oneshot::channel();

        let mut astronaut = match self.get_astronaut_by_id(raw_token, id.clone()).await {
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

        let task = tokio::spawn(async move {
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

                            if event.id != id {
                                continue;
                            };

                            astronaut = astronaut.apply_update_event(&event);

                            match tx.send(astronaut.clone()).await {
                                Ok(_) => {}
                                Err(err) => {
                                    error!(
                                        "error updating stream for get_astronaut_by_id: {}",
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

        let task2 = match self.token.get_token_seconds_remaining(raw_token) {
            Ok(seconds) => Ok(tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
                match kill_tx.send(()) {
                    Err(_) => error!("error sending kill signal after token expired"),
                    _ => {}
                };
            })),
            Err(_) => Err(AstronautQuerierError::Forbidden),
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
