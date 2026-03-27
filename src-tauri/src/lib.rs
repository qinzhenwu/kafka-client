mod commands;
mod error;
mod kafka;
mod models;

use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::Manager;

use kafka::KafkaClientManager;
use kafka::consumer::StreamingSessions;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let streaming_sessions: StreamingSessions = Arc::new(RwLock::new(Default::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When a second instance is started, focus the existing window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Arc::new(RwLock::new(KafkaClientManager::new())))
        .manage(streaming_sessions)
        .invoke_handler(tauri::generate_handler![
            commands::connect_cluster,
            commands::disconnect_cluster,
            commands::get_cluster_info,
            commands::list_connected_clusters,
            commands::test_cluster_connection,
            commands::list_topics,
            commands::get_topic_info,
            commands::create_topic,
            commands::update_topic_partitions,
            commands::delete_topic,
            commands::produce_message,
            commands::produce_batch,
            commands::consume_messages,
            commands::consume_from_offset,
            commands::consume_from_position,
            commands::start_realtime_consume,
            commands::stop_realtime_consume,
            commands::list_consumer_groups,
            commands::get_consumer_group_info,
            commands::get_group_lag,
            commands::reset_consumer_group_offsets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}