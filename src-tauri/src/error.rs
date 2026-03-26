use thiserror::Error;

#[derive(Error, Debug)]
pub enum KafkaClientError {
    #[error("Kafka error: {0}")]
    Kafka(#[from] rdkafka::error::KafkaError),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Topic not found: {0}")]
    TopicNotFound(String),

    #[error("Consumer group not found: {0}")]
    ConsumerGroupNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for KafkaClientError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, KafkaClientError>;