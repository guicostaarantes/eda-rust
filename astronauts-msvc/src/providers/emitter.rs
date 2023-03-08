use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaResult;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;

#[derive(Clone)]
pub struct KafkaEmitterImpl {
    client: FutureProducer,
}

impl KafkaEmitterImpl {
    pub fn new(kafka_url: &str) -> KafkaResult<Self> {
        let kafka_producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", kafka_url)
            .create()?;

        Ok(Self {
            client: kafka_producer,
        })
    }
}

impl KafkaEmitterImpl {
    pub async fn emit(&self, topic: &str, id: &str, payload: &str) -> KafkaResult<()> {
        let record = FutureRecord::to(topic).key(id).payload(payload);
        match self
            .client
            .send(record, std::time::Duration::from_secs(0))
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(err.0),
        }
    }
}
