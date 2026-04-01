use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication, NewPartitions};
use rdkafka::consumer::BaseConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::error::KafkaError;

use crate::error::{KafkaClientError, Result};
use crate::models::{BrokerInfo, ClusterInfo, CreateTopicRequest, TopicInfo, PartitionInfo};

pub struct KafkaAdminOps<'a> {
    admin: &'a AdminClient<rdkafka::client::DefaultClientContext>,
    consumer: &'a BaseConsumer,
}

impl<'a> KafkaAdminOps<'a> {
    pub fn new(admin: &'a AdminClient<rdkafka::client::DefaultClientContext>, consumer: &'a BaseConsumer) -> Self {
        Self { admin, consumer }
    }

    pub async fn get_cluster_info(&self) -> Result<ClusterInfo> {
        let metadata = self.admin
            .inner()
            .fetch_metadata(None, std::time::Duration::from_secs(5))?;

        let brokers: Vec<BrokerInfo> = metadata
            .brokers()
            .iter()
            .map(|b| BrokerInfo {
                id: b.id(),
                host: b.host().to_string(),
                port: b.port(),
                rack: None, // rack() method not available in this version
            })
            .collect();

        // Use a placeholder for cluster_id and controller_id
        // These methods may not be available in all rdkafka versions
        Ok(ClusterInfo {
            cluster_id: String::new(),
            controller_id: -1,
            brokers,
        })
    }

    pub async fn list_topics(&self) -> Result<Vec<String>> {
        let metadata = self.admin
            .inner()
            .fetch_metadata(None, std::time::Duration::from_secs(5))?;

        Ok(metadata
            .topics()
            .iter()
            .map(|t| t.name().to_string())
            .collect())
    }

    pub async fn get_topic_info(&self, topic_name: &str) -> Result<TopicInfo> {
        // fetch_metadata takes Option<&str> for topic name
        let metadata = self.admin
            .inner()
            .fetch_metadata(Some(topic_name), std::time::Duration::from_secs(5))?;

        let topic = metadata
            .topics()
            .first()
            .ok_or_else(|| KafkaClientError::TopicNotFound(topic_name.to_string()))?;

        // Build a map of broker id to host
        let broker_map: std::collections::HashMap<i32, String> = metadata
            .brokers()
            .iter()
            .map(|b| (b.id(), format!("{}:{}", b.host(), b.port())))
            .collect();

        let partitions: Vec<PartitionInfo> = topic
            .partitions()
            .iter()
            .map(|p| {
                // Fetch watermarks for each partition using the consumer
                let (low_watermark, high_watermark) = self.consumer
                    .fetch_watermarks(topic_name, p.id(), std::time::Duration::from_secs(5))
                    .unwrap_or((-1, -1));

                let leader_host = broker_map
                    .get(&p.leader())
                    .cloned()
                    .unwrap_or_else(|| p.leader().to_string());

                PartitionInfo {
                    id: p.id(),
                    leader: p.leader(),
                    leader_host,
                    replicas: p.replicas().to_vec(),
                    isr: p.isr().to_vec(),
                    high_watermark,
                    low_watermark,
                }
            })
            .collect();

        Ok(TopicInfo {
            name: topic.name().to_string(),
            partitions,
            configs: vec![],
        })
    }

    pub async fn create_topic(&self, request: CreateTopicRequest) -> Result<()> {
        let mut new_topic = NewTopic::new(
            &request.name,
            request.num_partitions,
            TopicReplication::Fixed(request.replication_factor),
        );

        for (key, value) in &request.configs {
            new_topic = new_topic.set(key, value);
        }

        let result = self.admin
            .create_topics(&[new_topic], &AdminOptions::new())
            .await?;

        for res in result {
            if let Err((_, err)) = res {
                return Err(KafkaClientError::Kafka(KafkaError::AdminOp(err)));
            }
        }

        Ok(())
    }

    pub async fn update_topic_partitions(&self, topic_name: &str, num_partitions: i32) -> Result<()> {
        // Get current partition count
        let metadata = self.admin
            .inner()
            .fetch_metadata(Some(topic_name), std::time::Duration::from_secs(5))?;

        let topic = metadata
            .topics()
            .first()
            .ok_or_else(|| KafkaClientError::TopicNotFound(topic_name.to_string()))?;

        let current_partitions = topic.partitions().len() as i32;

        if num_partitions <= current_partitions {
            return Err(KafkaClientError::InvalidConfig(
                format!("Cannot decrease partitions from {} to {}. Partition count can only be increased.",
                    current_partitions, num_partitions)
            ));
        }

        // Create new partitions request
        let new_partitions = NewPartitions::new(topic_name, num_partitions as usize);

        let result = self.admin
            .create_partitions(&[new_partitions], &AdminOptions::new())
            .await?;

        for res in result {
            if let Err((_, err)) = res {
                return Err(KafkaClientError::Kafka(KafkaError::AdminOp(err)));
            }
        }

        Ok(())
    }

    pub async fn delete_topic(&self, topic_name: &str) -> Result<()> {
        let result = self.admin
            .delete_topics(&[topic_name], &AdminOptions::new())
            .await?;

        for res in result {
            if let Err((_, err)) = res {
                return Err(KafkaClientError::Kafka(KafkaError::AdminOp(err)));
            }
        }

        Ok(())
    }
}