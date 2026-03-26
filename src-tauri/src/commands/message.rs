use std::sync::Arc;
use tauri::{State, AppHandle, Emitter};
use tokio::sync::RwLock;

use crate::error::Result;
use crate::kafka::KafkaClientManager;
use crate::models::{ProduceRequest, ConsumeRequest, ConsumeResponse};
use crate::kafka::consumer::{KafkaConsumerOps, start_streaming_consume, StreamingSessions};

#[tauri::command]
pub async fn produce_message(
    client_id: String,
    request: ProduceRequest,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<()> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let producer_ops = crate::kafka::KafkaProducerOps::new(&client.producer);
    producer_ops.produce(request)
}

#[tauri::command]
pub async fn produce_batch(
    client_id: String,
    requests: Vec<ProduceRequest>,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<Vec<Result<()>>> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let producer_ops = crate::kafka::KafkaProducerOps::new(&client.producer);
    producer_ops.produce_batch(requests)
}

#[tauri::command]
pub async fn consume_messages(
    client_id: String,
    request: ConsumeRequest,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<ConsumeResponse> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let consumer_ops = KafkaConsumerOps::new(&client.consumer);
    consumer_ops.consume(request)
}

#[tauri::command]
pub async fn consume_from_offset(
    client_id: String,
    topic: String,
    partition: i32,
    offset: i64,
    limit: i32,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<ConsumeResponse> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let consumer_ops = KafkaConsumerOps::new(&client.consumer);
    consumer_ops.consume_from_offset(&topic, partition, offset, limit)
}

#[tauri::command]
pub async fn consume_from_position(
    client_id: String,
    topic: String,
    position: String,
    limit: i32,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
) -> Result<ConsumeResponse> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let consumer_ops = KafkaConsumerOps::new(&client.consumer);
    consumer_ops.consume_from_position(&topic, &position, limit)
}

#[tauri::command]
pub async fn start_realtime_consume(
    client_id: String,
    topic: String,
    position: String,
    app: AppHandle,
    manager: State<'_, Arc<RwLock<KafkaClientManager>>>,
    sessions: State<'_, StreamingSessions>,
) -> Result<()> {
    let manager = manager.read().await;
    let client = manager
        .get_client(&client_id)
        .await
        .ok_or_else(|| crate::error::KafkaClientError::Connection("Client not found".to_string()))?;

    let session_key = format!("{}:{}", client_id, topic);

    // If already streaming, stop it first
    {
        let s = sessions.read().await;
        if *s.get(&session_key).unwrap_or(&false) {
            // Drop the read lock before acquiring write lock
            drop(s);
            let mut s = sessions.write().await;
            s.insert(session_key.clone(), false);
            // Wait a bit for the streaming task to stop
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            s.remove(&session_key);
        }
    }

    let consumer = client.consumer.clone();
    let sessions_clone = sessions.inner().clone();
    let session_key_clone = session_key.clone();
    let topic_clone = topic.clone();
    let position_clone = position.clone();
    let app_clone = app.clone();

    // Spawn background task for streaming
    tokio::spawn(async move {
        let _ = start_streaming_consume(
            consumer,
            app_clone,
            topic_clone,
            position_clone,
            sessions_clone,
            session_key_clone,
        ).await;
    });

    // Notify frontend that streaming started
    let _ = app.emit("kafka-streaming-started", serde_json::json!({
        "client_id": client_id,
        "topic": topic,
        "position": position
    }));

    Ok(())
}

#[tauri::command]
pub async fn stop_realtime_consume(
    client_id: String,
    topic: String,
    app: AppHandle,
    sessions: State<'_, StreamingSessions>,
) -> Result<()> {
    let session_key = format!("{}:{}", client_id, topic);

    // Mark session as inactive
    {
        let mut s = sessions.write().await;
        s.insert(session_key.clone(), false);
    }

    // Notify frontend that streaming stopped
    let _ = app.emit("kafka-streaming-stopped", serde_json::json!({
        "client_id": client_id,
        "topic": topic
    }));

    Ok(())
}