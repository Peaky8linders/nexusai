pub mod commands;
pub mod inference;
pub mod mcp;
pub mod memory;
pub mod models;
pub mod rag;
pub mod tools;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data).ok();
            std::fs::create_dir_all(app_data.join("models")).ok();
            std::fs::create_dir_all(app_data.join("conversations")).ok();
            std::fs::create_dir_all(app_data.join("rag_index")).ok();

            let db_path = app_data.join("nexusai.db");
            let db = memory::init_db(&db_path).expect("failed to init database");
            app.manage(memory::DbState(std::sync::Mutex::new(db)));

            let engine = inference::InferenceEngine::new();
            app.manage(inference::EngineState(tokio::sync::Mutex::new(engine)));

            tracing::info!("NexusAI initialized, data dir: {:?}", app_data);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::chat::send_message,
            commands::chat::stop_generation,
            commands::chat::get_conversations,
            commands::chat::create_conversation,
            commands::chat::delete_conversation,
            commands::models::list_models,
            commands::models::download_model,
            commands::models::delete_model,
            commands::models::get_download_progress,
            commands::models::load_model,
            commands::models::unload_model,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::system::get_system_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running NexusAI");
}
