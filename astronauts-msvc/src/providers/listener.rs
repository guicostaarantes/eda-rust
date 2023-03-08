use async_stream::stream;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::Message;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct KafkaConsumerImpl {
    kafka_url: String,
    group_id: String,
    consumers: Arc<Mutex<HashMap<String, Arc<StreamConsumer>>>>,
}

impl KafkaConsumerImpl {
    pub fn new(kafka_url: &str, group_id: &str) -> Self {
        Self {
            kafka_url: kafka_url.to_string(),
            group_id: group_id.to_string(),
            consumers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl KafkaConsumerImpl {
    pub fn listen_and_commit<'a>(
        &'a self,
        topic: &'a str,
        description: &'a str,
    ) -> impl Stream<Item = String> + 'a {
        let identifier = format!("{}__{}", description, self.group_id);

        let mut all_consumers = self.consumers.lock().unwrap();

        let consumer = match all_consumers.get(&identifier) {
            Some(c) => {
                println!(
                    "Reusing existing consumer for {}, with {}",
                    &identifier,
                    Arc::strong_count(c)
                );
                Arc::clone(c)
            }
            None => {
                let consumer: StreamConsumer = ClientConfig::new()
                    .set("bootstrap.servers", &self.kafka_url)
                    .set("group.id", identifier.clone())
                    .set("enable.auto.commit", "true")
                    .set("auto.commit.interval.ms", "1000")
                    .set("auto.offset.reset", "latest")
                    .create()
                    .expect("Consumer creation failed");

                consumer
                    .subscribe(&[topic])
                    .expect(&format!("Could not subscribe to {}", &topic)[..]);

                all_consumers.insert(identifier.clone(), Arc::new(consumer));

                match all_consumers.get(&identifier) {
                    Some(c) => {
                        println!(
                            "Starting new consumer for {}, with {}",
                            &identifier,
                            Arc::strong_count(c)
                        );
                        Arc::clone(c)
                    }
                    None => panic!("Couldn't get a value that was just inserted"),
                }
            }
        };

        stream! {
            loop {
                match consumer.stream().next().await {
                    Some(Ok(msg)) => {
                        match msg.payload_view::<str>() {
                            Some(Ok(s)) => yield String::from(s),
                            Some(Err(e)) => println!("Error while deserializing message payload: {:?}", e),
                            None => println!("Received None from consumer"),
                        }
                    },
                    Some(Err(err)) => println!("Error while consuming from topic: {:?}", err),
                    None => println!("Received None from consumer"),
                }
            }
        }
    }
}
