use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::kafka::KafkaClientManager;
use crate::models::{CreateTopicRequest, TopicInfo};

#[tauri::command]
pub async fn list_topics(
    client_id: String,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<Vec<String>> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let admin_ops = crate::kafka::KafkaAdminOps::new(&client.admin, &client.consumer);
    admin_ops.list_topics().await
}

#[tauri::command]
pub async fn get_topic_info(
    client_id: String,
    topic_name: String,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<TopicInfo> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let admin_ops = crate::kafka::KafkaAdminOps::new(&client.admin, &client.consumer);
    admin_ops.get_topic_info(&topic_name).await
}

#[tauri::command]
pub async fn create_topic(
    client_id: String,
    request: CreateTopicRequest,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<()> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let admin_ops = crate::kafka::KafkaAdminOps::new(&client.admin, &client.consumer);
    admin_ops.create_topic(request).await
}

#[tauri::command]
pub async fn update_topic_partitions(
    client_id: String,
    topic_name: String,
    num_partitions: i32,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<()> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let admin_ops = crate::kafka::KafkaAdminOps::new(&client.admin, &client.consumer);
    admin_ops.update_topic_partitions(&topic_name, num_partitions).await
}

#[tauri::command]
pub async fn delete_topic(
    client_id: String,
    topic_name: String,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<()> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let admin_ops = crate::kafka::KafkaAdminOps::new(&client.admin, &client.consumer);
    admin_ops.delete_topic(&topic_name).await
}