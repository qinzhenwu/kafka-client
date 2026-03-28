use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rdkafka::admin::AdminClient;
use rdkafka::consumer::BaseConsumer;
use rdkafka::producer::BaseProducer;
use rdkafka::client::DefaultClientContext;

use crate::error::{Result, KafkaClientError};
use crate::models::{ClusterConfig, ClusterInfo};

pub type AdminClientRef = Arc<AdminClient<DefaultClientContext>>;
pub type ConsumerRef = Arc<BaseConsumer>;
pub type ProducerRef = Arc<BaseProducer>;

#[derive(Clone)]
pub struct KafkaClientManager {
    clients: Arc<RwLock<HashMap<String, KafkaClient>>>,
}

#[derive(Clone)]
pub struct KafkaClient {
    pub config: ClusterConfig,
    pub admin: AdminClientRef,
    pub consumer: ConsumerRef,
    pub producer: ProducerRef,
}

impl KafkaClientManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn connect(&self, config: ClusterConfig) -> Result<String> {
        let client_id = config.id.clone();
        let rdkafka_config = config.to_rdkafka_config();

        // Create admin client
        let admin: AdminClient<_> = rdkafka_config.create()
            .map_err(|e| KafkaClientError::Connection(format!("Failed to create admin client: {}", e)))?;

        // Create consumer - use unique group id per session to avoid conflicts
        // but with auto commit disabled, offsets are never persisted
        let mut consumer_config = config.to_rdkafka_config();
        consumer_config.set("group.id", format!("kafka-client-tool-{}", config.id));
        consumer_config.set("enable.auto.commit", "false");
        consumer_config.set("enable.auto.offset.store", "false");
        consumer_config.set("auto.offset.reset", "earliest");
        let consumer: BaseConsumer = consumer_config.create()
            .map_err(|e| KafkaClientError::Connection(format!("Failed to create consumer: {}", e)))?;

        // Create producer
        let producer: BaseProducer = config.to_rdkafka_config().create()
            .map_err(|e| KafkaClientError::Connection(format!("Failed to create producer: {}", e)))?;

        // Wrap in Arc
        let admin = Arc::new(admin);
        let consumer = Arc::new(consumer);
        let producer = Arc::new(producer);

        // Verify connection by fetching cluster metadata
        use crate::kafka::KafkaAdminOps;
        let admin_ops = KafkaAdminOps::new(&admin, &consumer);
        admin_ops.get_cluster_info().await
            .map_err(|e| KafkaClientError::Connection(format!("Failed to connect to cluster: {}", e)))?;

        let client = KafkaClient {
            config,
            admin,
            consumer,
            producer,
        };

        let mut clients = self.clients.write().await;
        clients.insert(client_id.clone(), client);

        Ok(client_id)
    }

    pub async fn disconnect(&self, client_id: &str) -> Result<()> {
        let mut clients = self.clients.write().await;
        clients.remove(client_id);
        Ok(())
    }

    pub async fn get_client(&self, client_id: &str) -> Option<KafkaClient> {
        let clients = self.clients.read().await;
        clients.get(client_id).cloned()
    }

    pub async fn list_clients(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }

    pub async fn restore_connections(&self, configs: &[ClusterConfig]) -> Vec<(String, Result<String>)> {
        let mut results = Vec::new();
        for config in configs {
            let name = config.name.clone();
            let result = self.connect(config.clone()).await;
            results.push((name, result));
        }
        results
    }

    /// Test connection to a cluster without persisting it
    pub async fn test_connection(&self, config: &ClusterConfig) -> Result<ClusterInfo> {
        let rdkafka_config = config.to_rdkafka_config();

        // Create admin client
        let admin: AdminClient<_> = rdkafka_config.create()
            .map_err(|e| KafkaClientError::Connection(format!("Failed to create admin client: {}", e)))?;

        // Create consumer for fetching metadata - unique group id for testing
        let mut consumer_config = config.to_rdkafka_config();
        consumer_config.set("group.id", format!("kafka-client-test-{}", uuid::Uuid::new_v4()));
        consumer_config.set("enable.auto.commit", "false");
        consumer_config.set("enable.auto.offset.store", "false");
        let consumer: BaseConsumer = consumer_config.create()
            .map_err(|e| KafkaClientError::Connection(format!("Failed to create consumer: {}", e)))?;

        // Wrap in Arc to extend lifetime
        let admin = Arc::new(admin);
        let consumer = Arc::new(consumer);

        // Try to fetch cluster metadata to verify connection
        use crate::kafka::KafkaAdminOps;
        let admin_ops = KafkaAdminOps::new(&admin, &consumer);
        admin_ops.get_cluster_info().await
    }
}

impl Default for KafkaClientManager {
    fn default() -> Self {
        Self::new()
    }
}