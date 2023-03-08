use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::Message;
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

#[derive(Clone)]
pub struct KafkaConsumerImpl {
    kafka_url: String,
    group_id: String,
    consumers: Arc<Mutex<HashMap<String, Arc<StreamConsumer>>>>,
    senders: Arc<Mutex<HashMap<String, Arc<Sender<String>>>>>,
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
                .set("auto.offset.reset", "latest")
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
    fn get_sender(&self, topic: String, description: String) -> Arc<Sender<String>> {
        let identifier = self.get_identifier(topic, description);

        let mut all_senders = self.senders.lock().unwrap();

        if all_senders.get(&identifier).is_none() {
            // TODO: check if 256 is enough for all use cases
            // Possibly raise an alert if the channel gets half-full
            let (tx, _rx) = channel(256);

            all_senders.insert(identifier.clone(), Arc::new(tx));
        }

        Arc::clone(all_senders.get(&identifier).expect("should always exist"))
    }
}

impl KafkaConsumerImpl {
    fn get_receiver(&self, topic: String, description: String) -> Receiver<String> {
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
                                match sender.send(s.to_string()) {
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
    ) -> impl Stream<Item = Result<String, BroadcastStreamRecvError>> {
        let receiver = self.get_receiver(topic.to_string(), description.to_string());
        BroadcastStream::new(receiver)
    }
}
