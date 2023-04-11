use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_model::AstronautDocument;
use crate::domain::astronaut_model::AstronautUpdatedEvent;
use crate::providers::json::JsonSerializerImpl;
use crate::providers::listener::KafkaConsumerImpl;
use crate::providers::state::MongoStateImpl;
use crate::providers::token::RawToken;
use crate::providers::token::TokenImpl;
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
    #[error("token not found")]
    TokenNotFound,
    #[error("forbidden")]
    Forbidden,
}

#[derive(Clone)]
pub struct AstronautQuerier {
    listener: Arc<KafkaConsumerImpl>,
    state: Arc<MongoStateImpl>,
    token: Arc<TokenImpl>,
}

impl AstronautQuerier {
    pub fn new(
        listener: Arc<KafkaConsumerImpl>,
        state: Arc<MongoStateImpl>,
        token: Arc<TokenImpl>,
    ) -> Self {
        Self {
            listener,
            state,
            token,
        }
    }
}

async fn allow_get_astronaut_by_id(
    token_impl: Arc<TokenImpl>,
    raw_token: &RawToken,
) -> Result<(), AstronautQuerierError> {
    match token_impl.fetch_token(raw_token).await {
        Ok(token) => {
            let allowed = token.payload.permissions.iter().any(|perm| {
                if perm == &"GET_ANY_ASTRONAUT" {
                    true
                } else {
                    false
                }
            });

            if allowed {
                Ok(())
            } else {
                Err(AstronautQuerierError::Forbidden)
            }
        }
        Err(_) => Err(AstronautQuerierError::TokenNotFound),
    }
}

impl AstronautQuerier {
    pub async fn get_astronaut_by_id(
        &self,
        raw_token: &RawToken,
        id: String,
    ) -> Result<Astronaut, AstronautQuerierError> {
        allow_get_astronaut_by_id(self.token.clone(), raw_token).await?;

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

        let token_impl = self.token.clone();
        let raw_token_c = raw_token.clone();

        let task = tokio::spawn(async move {
            while let Some(r) = stream.next().await {
                match allow_get_astronaut_by_id(token_impl.clone(), &raw_token_c).await {
                    Ok(_) => {}
                    Err(_) => {
                        kill_tx.send(()).unwrap_or(());
                        break;
                    }
                };

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

        tokio::spawn(async move {
            tokio::select! {
                _ = tx2.closed() => {
                    task.abort();
                }
                _ = kill_rx => {
                    task.abort();
                }
            }
        });

        Ok(ReceiverStream::new(rx))
    }
}
