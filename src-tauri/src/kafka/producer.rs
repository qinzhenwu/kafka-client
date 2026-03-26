use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
use rdkafka::message::{OwnedHeaders, Header};

use crate::error::{KafkaClientError, Result};
use crate::models::ProduceRequest;

pub struct KafkaProducerOps<'a> {
    producer: &'a BaseProducer,
}

impl<'a> KafkaProducerOps<'a> {
    pub fn new(producer: &'a BaseProducer) -> Self {
        Self { producer }
    }

    pub fn produce(&self, request: ProduceRequest) -> Result<()> {
        let mut record = BaseRecord::to(&request.topic);

        if let Some(partition) = request.partition {
            record = record.partition(partition);
        }

        if let Some(ref key) = request.key {
            record = record.key(key.as_bytes());
        }

        record = record.payload(request.value.as_bytes());

        // Add headers
        if !request.headers.is_empty() {
            let mut headers = OwnedHeaders::new();
            for (key, value) in &request.headers {
                headers = headers.insert(Header {
                    key: key.as_str(),
                    value: Some(value.as_bytes()),
                });
            }
            record = record.headers(headers);
        }

        self.producer
            .send(record)
            .map_err(|(e, _)| KafkaClientError::Connection(format!("Failed to produce message: {}", e)))?;

        // Flush to ensure message is sent
        self.producer.flush(std::time::Duration::from_secs(1));

        Ok(())
    }

    pub fn produce_batch(&self, requests: Vec<ProduceRequest>) -> Result<Vec<Result<()>>> {
        let mut results = Vec::with_capacity(requests.len());

        for request in requests {
            results.push(self.produce(request));
        }

        // Flush all messages
        self.producer.flush(std::time::Duration::from_secs(5));

        Ok(results)
    }
}