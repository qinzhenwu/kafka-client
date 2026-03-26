use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::kafka::KafkaClientManager;
use crate::kafka::consumer::StreamingSessions;
use crate::models::{ClusterConfig, ClusterInfo};

#[tauri::command]
pub async fn connect_cluster(
    config: ClusterConfig,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<String> {
    let manager = manager.read().await;
    manager.connect(config).await
}

#[tauri::command]
pub async fn disconnect_cluster(
    client_id: String,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
    sessions: State<'_, StreamingSessions>,
) -> Result<()> {
    // Stop all streaming sessions for this client
    {
        let mut s = sessions.write().await;
        // Find all sessions for this client and mark them as inactive
        let keys_to_stop: Vec<String> = s.keys()
            .filter(|k| k.starts_with(&format!("{}:", client_id)))
            .cloned()
            .collect();

        for key in keys_to_stop {
            s.insert(key, false);
        }
    }

    // Wait a bit for streaming tasks to stop
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Remove all sessions for this client
    {
        let mut s = sessions.write().await;
        s.retain(|k, _| !k.starts_with(&format!("{}:", client_id)));
    }

    // Disconnect the client
    let manager = manager.read().await;
    manager.disconnect(&client_id).await
}

#[tauri::command]
pub async fn get_cluster_info(
    client_id: String,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<ClusterInfo> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let admin_ops = crate::kafka::KafkaAdminOps::new(&client.admin, &client.consumer);
    admin_ops.get_cluster_info().await
}

#[tauri::command]
pub async fn list_connected_clusters(
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<Vec<String>> {
    let manager = manager.read().await;
    Ok(manager.list_clients().await)
}

#[tauri::command]
pub async fn test_cluster_connection(
    config: ClusterConfig,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<ClusterInfo> {
    // Create a temporary client to test the connection
    let manager = manager.read().await;
    manager.test_connection(&config).await
}