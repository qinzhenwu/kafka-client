use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::message::{Message, Headers};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::error::{KafkaClientError, Result};
use crate::models::{KafkaMessage, ConsumeRequest, ConsumeResponse};
use chrono::{DateTime, Utc};
use std::time::Duration;
use tauri::AppHandle;
use tauri::Emitter;

// Track active streaming sessions
pub type StreamingSessions = Arc<RwLock<HashMap<String, bool>>>;

pub struct KafkaConsumerOps<'a> {
    consumer: &'a BaseConsumer,
}

impl<'a> KafkaConsumerOps<'a> {
    pub fn new(consumer: &'a BaseConsumer) -> Self {
        Self { consumer }
    }

    pub fn consume(&self, request: ConsumeRequest) -> Result<ConsumeResponse> {
        // Subscribe to topic
        self.consumer.subscribe(&[&request.topic])
            .map_err(|e| KafkaClientError::Kafka(e))?;

        let mut messages = Vec::new();
        let timeout = Duration::from_secs(5);

        while messages.len() < request.limit as usize {
            match self.consumer.poll(timeout) {
                Some(Ok(msg)) => {
                    let kafka_msg = self.parse_message(&msg)?;
                    messages.push(kafka_msg);
                }
                Some(Err(_)) => continue,
                None => break,
            }
        }

        let has_more = messages.len() >= request.limit as usize;

        Ok(ConsumeResponse {
            messages,
            has_more,
        })
    }

    pub fn consume_from_offset(
        &self,
        topic: &str,
        partition: i32,
        offset: i64,
        limit: i32,
    ) -> Result<ConsumeResponse> {
        // Clear any existing assignment to flush internal buffers
        let empty_tpl = rdkafka::TopicPartitionList::new();
        let _ = self.consumer.assign(&empty_tpl);

        // Create topic partition list with specific offset
        let mut tpl = rdkafka::TopicPartitionList::new();
        tpl.add_partition_offset(topic, partition, rdkafka::Offset::Offset(offset))
            .map_err(|e| KafkaClientError::Connection(format!("Failed to set offset: {:?}", e)))?;

        self.consumer.assign(&tpl)
            .map_err(|e| KafkaClientError::Kafka(e))?;

        let mut messages = Vec::new();
        let timeout = Duration::from_secs(5);

        while messages.len() < limit as usize {
            match self.consumer.poll(timeout) {
                Some(Ok(msg)) => {
                    let kafka_msg = self.parse_message(&msg)?;
                    messages.push(kafka_msg);
                }
                Some(Err(_)) => continue,
                None => break,
            }
        }

        let has_more = messages.len() >= limit as usize;

        Ok(ConsumeResponse {
            messages,
            has_more,
        })
    }

    pub fn consume_from_position(
        &self,
        topic: &str,
        position: &str,
        limit: i32,
    ) -> Result<ConsumeResponse> {
        // Clear any existing assignment to flush internal buffers
        let empty_tpl = rdkafka::TopicPartitionList::new();
        let _ = self.consumer.assign(&empty_tpl);

        // Get partition info first
        let metadata = self.consumer
            .fetch_metadata(Some(topic), Duration::from_secs(5))
            .map_err(|e| KafkaClientError::Kafka(e))?;

        let topic_meta = metadata.topics().iter()
            .find(|t| t.name() == topic)
            .ok_or_else(|| KafkaClientError::TopicNotFound(topic.to_string()))?;

        let mut tpl = rdkafka::TopicPartitionList::new();

        // Check if position is a named position or numeric offset
        if position == "earliest" {
            // Set starting position for all partitions to Beginning
            for partition in topic_meta.partitions() {
                tpl.add_partition_offset(topic, partition.id(), rdkafka::Offset::Beginning)
                    .map_err(|e| KafkaClientError::Connection(format!("Failed to set offset: {:?}", e)))?;
            }
        } else if position == "latest" {
            // For "latest", we want to get the most recent N messages
            // Get watermarks for each partition and calculate starting offsets

            let partitions = topic_meta.partitions();
            let partition_count = partitions.len();

            // Calculate how many messages to fetch from each partition
            // Distribute limit evenly across partitions
            let per_partition_limit = (limit as usize / partition_count).max(1);

            for partition in partitions {
                // Get watermarks (low_watermark, high_watermark)
                let (low, high) = self.consumer
                    .fetch_watermarks(topic, partition.id(), Duration::from_secs(5))
                    .map_err(|e| KafkaClientError::Kafka(e))?;

                // Calculate starting offset: go back per_partition_limit from high watermark
                // But ensure we don't go below low watermark
                let start_offset = std::cmp::max(low, high - per_partition_limit as i64);

                // If partition has no messages (low == high), skip it
                if low < high {
                    tpl.add_partition_offset(topic, partition.id(), rdkafka::Offset::Offset(start_offset))
                        .map_err(|e| KafkaClientError::Connection(format!("Failed to set offset: {:?}", e)))?;
                }
            }
        } else {
            // Try to parse as numeric offset
            let offset_value: i64 = position.parse()
                .map_err(|_| KafkaClientError::Connection(format!("Invalid offset value: {}", position)))?;

            for partition in topic_meta.partitions() {
                tpl.add_partition_offset(topic, partition.id(), rdkafka::Offset::Offset(offset_value))
                    .map_err(|e| KafkaClientError::Connection(format!("Failed to set offset: {:?}", e)))?;
            }
        }

        // Use assign instead of subscribe for manual partition assignment
        self.consumer.assign(&tpl)
            .map_err(|e| KafkaClientError::Kafka(e))?;

        let mut messages = Vec::new();
        let timeout = Duration::from_secs(5);

        while messages.len() < limit as usize {
            match self.consumer.poll(timeout) {
                Some(Ok(msg)) => {
                    let kafka_msg = self.parse_message(&msg)?;
                    messages.push(kafka_msg);
                }
                Some(Err(_)) => continue,
                None => break,
            }
        }

        let has_more = messages.len() >= limit as usize;

        Ok(ConsumeResponse {
            messages,
            has_more,
        })
    }

    fn parse_message(&self, msg: &rdkafka::message::BorrowedMessage) -> Result<KafkaMessage> {
        let key = msg.key().map(|k| String::from_utf8_lossy(k).to_string());
        let value = msg.payload().map(|p| String::from_utf8_lossy(p).to_string());

        let mut headers = HashMap::new();
        if let Some(msg_headers) = msg.headers() {
            for header in msg_headers.iter() {
                let key = header.key.to_string();
                let value = header.value
                    .map(|v| String::from_utf8_lossy(v).to_string())
                    .unwrap_or_default();
                headers.insert(key, value);
            }
        }

        let timestamp = msg.timestamp()
            .to_millis()
            .map(|ms| DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now))
            .unwrap_or_else(Utc::now);

        Ok(KafkaMessage {
            partition: msg.partition(),
            offset: msg.offset(),
            key,
            value,
            headers,
            timestamp,
        })
    }
}

/// Start streaming consume - emits messages via Tauri events
pub async fn start_streaming_consume(
    consumer: Arc<BaseConsumer>,
    app_handle: AppHandle,
    topic: String,
    position: String,
    sessions: StreamingSessions,
    session_key: String,
) -> Result<()> {
    // Mark session as active
    {
        let mut s = sessions.write().await;
        s.insert(session_key.clone(), true);
    }

    // Clear any existing assignment to flush internal buffers
    let empty_tpl = rdkafka::TopicPartitionList::new();
    let _ = consumer.assign(&empty_tpl);

    // Set starting position
    let offset = match position.as_str() {
        "earliest" => rdkafka::Offset::Beginning,
        "latest" => rdkafka::Offset::End,
        _ => rdkafka::Offset::Beginning,
    };

    // Get partition info and assign offsets
    let metadata = consumer
        .fetch_metadata(Some(&topic), Duration::from_secs(5))
        .map_err(|e| KafkaClientError::Kafka(e))?;

    let topic_meta = metadata.topics().iter()
        .find(|t| t.name() == topic)
        .ok_or_else(|| KafkaClientError::TopicNotFound(topic.clone()))?;

    let mut tpl = rdkafka::TopicPartitionList::new();
    for partition in topic_meta.partitions() {
        tpl.add_partition_offset(&topic, partition.id(), offset)
            .map_err(|e| KafkaClientError::Connection(format!("Failed to set offset: {:?}", e)))?;
    }

    // Use assign instead of subscribe for manual partition assignment
    consumer.assign(&tpl)
        .map_err(|e| KafkaClientError::Kafka(e))?;

    let timeout = Duration::from_millis(500);

    // Start polling loop
    loop {
        // Check if session is still active
        {
            let s = sessions.read().await;
            if !s.get(&session_key).unwrap_or(&false) {
                break;
            }
        }

        match consumer.poll(timeout) {
            Some(Ok(msg)) => {
                // Parse message
                let key = msg.key().map(|k| String::from_utf8_lossy(k).to_string());
                let value = msg.payload().map(|p| String::from_utf8_lossy(p).to_string());

                let mut headers = HashMap::new();
                if let Some(msg_headers) = msg.headers() {
                    for header in msg_headers.iter() {
                        let h_key = header.key.to_string();
                        let h_value = header.value
                            .map(|v| String::from_utf8_lossy(v).to_string())
                            .unwrap_or_default();
                        headers.insert(h_key, h_value);
                    }
                }

                let timestamp = msg.timestamp()
                    .to_millis()
                    .map(|ms| DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now))
                    .unwrap_or_else(Utc::now);

                let kafka_msg = KafkaMessage {
                    partition: msg.partition(),
                    offset: msg.offset(),
                    key,
                    value,
                    headers,
                    timestamp,
                };

                // Emit event to frontend
                let _ = app_handle.emit("kafka-message", kafka_msg);
            }
            Some(Err(_)) => continue,
            None => continue,
        }
    }

    // Clean up session
    {
        let mut s = sessions.write().await;
        s.remove(&session_key);
    }

    Ok(())
}