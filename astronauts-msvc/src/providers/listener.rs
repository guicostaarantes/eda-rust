use async_graphql::futures_util::future::join_all;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::Message;
use rdkafka::Timestamp;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::broadcast::channel;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

#[derive(Clone, Debug)]
pub struct KafkaMessage {
    timestamp: Timestamp,
    payload: String,
}

impl KafkaMessage {
    pub fn get_timestamp(&self) -> isize {
        self.timestamp.to_millis().unwrap_or_else(|| {
            println!("got none for a kafka timestamp");
            0
        }) as isize
    }

    pub fn get_payload(&self) -> String {
        self.payload.to_owned()
    }
}

#[derive(Clone)]
pub struct KafkaConsumerImpl {
    kafka_url: String,
    group_id: String,
    consumers: Arc<Mutex<HashMap<String, Arc<StreamConsumer>>>>,
    senders: Arc<Mutex<HashMap<String, Arc<Sender<KafkaMessage>>>>>,
}

impl KafkaConsumerImpl {
    pub fn new(kafka_url: &str, group_id: &str) -> Self {
        Self {
            kafka_url: kafka_url.to_string(),
            group_id: group_id.to_string(),
            consumers: Arc::new(Mutex::new(HashMap::new())),
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl KafkaConsumerImpl {
    fn get_identifier(&self, topic: String, description: String) -> String {
        format!("{}__{}__{}", topic, description, self.group_id)
    }
}

impl KafkaConsumerImpl {
    fn get_consumer(&self, topic: String, description: String) -> Arc<StreamConsumer> {
        let identifier = self.get_identifier(topic.clone(), description);

        let mut all_consumers = self.consumers.lock().unwrap();

        if all_consumers.get(&identifier).is_none() {
            let consumer: StreamConsumer = ClientConfig::new()
                .set("bootstrap.servers", &self.kafka_url)
                .set("group.id", &identifier)
                .set("enable.auto.commit", "true")
                .set("auto.commit.interval.ms", "1000")
                .set("auto.offset.reset", "earliest")
                .create()
                .expect("Consumer creation failed");

            consumer
                .subscribe(&[&topic])
                .expect(&format!("Could not subscribe to {}", &topic)[..]);

            all_consumers.insert(identifier.clone(), Arc::new(consumer));
        }

        Arc::clone(all_consumers.get(&identifier).expect("should always exist"))
    }
}

impl KafkaConsumerImpl {
    fn get_sender(&self, topic: String, description: String) -> Arc<Sender<KafkaMessage>> {
        let identifier = self.get_identifier(topic, description);

        let mut all_senders = self.senders.lock().unwrap();

        if all_senders.get(&identifier).is_none() {
            let (tx, _rx) = channel(4096);

            all_senders.insert(identifier.clone(), Arc::new(tx));
        }

        Arc::clone(all_senders.get(&identifier).expect("should always exist"))
    }
}

impl KafkaConsumerImpl {
    fn get_receiver(&self, topic: String, description: String) -> Receiver<KafkaMessage> {
        let consumer = self.get_consumer(topic.clone(), description.clone());
        let sender = self.get_sender(topic.clone(), description.clone());

        tokio::spawn(async move {
            // no need to create one async thread per receiver, so we create only when it goes to one
            if sender.receiver_count() == 1 {
                // also, no need to keep this loop around if the number of receivers goes back to zero
                while sender.receiver_count() != 0 {
                    match consumer.stream().next().await {
                        Some(Ok(msg)) => match msg.payload_view::<str>() {
                            Some(Ok(s)) => {
                                match sender.send(KafkaMessage {
                                    timestamp: msg.timestamp(),
                                    payload: s.to_string(),
                                }) {
                                    Err(err) => {
                                        println!("Error while sending message to channel: {}", err)
                                    }
                                    _ => {}
                                };
                            }
                            Some(Err(err)) => {
                                println!("Error while deserializing message payload: {:?}", err)
                            }
                            None => println!("Received None from consumer deserializer"),
                        },
                        Some(Err(err)) => println!("Error while consuming from topic: {:?}", err),
                        None => println!("Received None from consumer"),
                    }
                }
            }
        });

        self.get_sender(topic.clone(), description.clone())
            .subscribe()
    }
}

impl KafkaConsumerImpl {
    pub fn listen(
        &self,
        topic: &str,
        description: &str,
    ) -> impl Stream<Item = Result<KafkaMessage, BroadcastStreamRecvError>> {
        let receiver = self.get_receiver(topic.to_string(), description.to_string());
        BroadcastStream::new(receiver)
    }
}

enum StreamState {
    HasMsg(KafkaMessage),
    NoNewMsg,
}

struct StreamProps {
    pub stream: BroadcastStream<KafkaMessage>,
    pub state: StreamState,
    pub topic_index: usize,
}

#[derive(Clone)]
pub struct MultipleStreamMessage {
    pub topic_index: usize,
    pub message: KafkaMessage,
}

impl KafkaConsumerImpl {
    pub fn listen_multiple(
        &self,
        topics: &[&str],
        description: &str,
    ) -> impl Stream<Item = Result<MultipleStreamMessage, BroadcastStreamRecvError>> {
        let mut streams: Vec<_> = topics
            .iter()
            .map(|t| self.get_receiver(t.to_string(), description.to_string()))
            .enumerate()
            .map(|(i, r)| StreamProps {
                stream: BroadcastStream::new(r),
                state: StreamState::NoNewMsg,
                topic_index: i,
            })
            .collect();

        let (tx, rx) = channel(4096);

        // for the first run we allow more time for all stream connections to stabilize
        // in the following runs we use 1000ms
        let mut timeout_ms = 5000;

        tokio::spawn(async move {
            loop {
                let st = &mut streams;

                let promises = st
                    .iter_mut()
                    .filter(|s| match s.state {
                        StreamState::HasMsg(_) => false,
                        StreamState::NoNewMsg => true,
                    })
                    .map(|s| async {
                        s.state = match tokio::time::timeout(
                            std::time::Duration::from_millis(timeout_ms),
                            s.stream.next(),
                        )
                        .await
                        {
                            Ok(Some(Ok(msg))) => StreamState::HasMsg(msg),
                            Ok(Some(Err(err))) => {
                                println!("error in stream from kafka: {}", err);
                                StreamState::NoNewMsg
                            }
                            Ok(None) => {
                                println!("should never return this");
                                StreamState::NoNewMsg
                            }
                            Err(_) => StreamState::NoNewMsg,
                        };
                    });

                join_all(promises).await;

                let chosen = st
                    .iter_mut()
                    .min_by(|a, b| match (&a.state, &b.state) {
                        (StreamState::HasMsg(m1), StreamState::HasMsg(m2)) => {
                            m1.get_timestamp().partial_cmp(&m2.get_timestamp()).unwrap()
                        }
                        (StreamState::HasMsg(_), StreamState::NoNewMsg) => Ordering::Less,
                        (StreamState::NoNewMsg, StreamState::HasMsg(_)) => Ordering::Greater,
                        _ => Ordering::Equal,
                    })
                    .unwrap();

                match &chosen.state {
                    StreamState::NoNewMsg => {
                        timeout_ms = 1000;
                    }
                    StreamState::HasMsg(msg) => {
                        timeout_ms = 1000;
                        match tx.send(MultipleStreamMessage {
                            topic_index: chosen.topic_index,
                            message: msg.clone(),
                        }) {
                            Err(err) => {
                                println!("Error while sending message to channel: {}", err)
                            }
                            _ => {}
                        };
                        chosen.state = StreamState::NoNewMsg;
                    }
                }
            }
        });

        BroadcastStream::new(rx)
    }
}
