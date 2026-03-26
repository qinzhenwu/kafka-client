use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::kafka::KafkaClientManager;
use crate::models::{ConsumerGroupInfo, LagInfo, ResetOffsetType};

#[tauri::command]
pub async fn list_consumer_groups(
    client_id: String,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<Vec<String>> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let group_ops = crate::kafka::KafkaConsumerGroupOps::new(&client.admin, &client.consumer, &client.config);
    group_ops.list_groups().await
}

#[tauri::command]
pub async fn get_consumer_group_info(
    client_id: String,
    group_id: String,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<ConsumerGroupInfo> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let group_ops = crate::kafka::KafkaConsumerGroupOps::new(&client.admin, &client.consumer, &client.config);
    group_ops.get_group_info(&group_id).await
}

#[tauri::command]
pub async fn get_group_lag(
    client_id: String,
    group_id: String,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<Vec<LagInfo>> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let group_ops = crate::kafka::KafkaConsumerGroupOps::new(&client.admin, &client.consumer, &client.config);
    group_ops.get_group_lag(&group_id).await
}

#[tauri::command]
pub async fn reset_consumer_group_offsets(
    client_id: String,
    group_id: String,
    topic: String,
    partitions: Vec<i32>,
    reset_type: ResetOffsetType,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<()> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let group_ops = crate::kafka::KafkaConsumerGroupOps::new(&client.admin, &client.consumer, &client.config);
    group_ops.reset_offsets(&group_id, &topic, partitions, reset_type).await
}