use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::inference::EngineState;
use crate::memory::DbState;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub model_id: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub conversation_id: String,
    pub content: String,
    pub model_id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamChunk {
    pub conversation_id: String,
    pub content: String,
    pub done: bool,
    pub tokens_per_second: Option<f64>,
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    request: SendMessageRequest,
    engine: State<'_, EngineState>,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let conversation_id = request.conversation_id.clone();

    // Store the user message
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        crate::memory::store_message(
            &conn,
            &conversation_id,
            "user",
            &request.content,
        )
        .map_err(|e| e.to_string())?;
    }

    // Generate response via inference engine — drop guard before any .await
    let response = {
        let engine_guard = engine.0.lock().map_err(|e| e.to_string())?;
        engine_guard
            .generate(&request.content)
            .map_err(|e| e.to_string())?
    };

    // Emit streamed chunks (string-safe slicing)
    let chars: Vec<char> = response.chars().collect();
    for (i, chunk) in chars.chunks(32).enumerate() {
        let text: String = chunk.iter().collect();
        let stream_chunk = StreamChunk {
            conversation_id: conversation_id.clone(),
            content: text,
            done: false,
            tokens_per_second: Some(25.0), // placeholder
        };
        app.emit("chat:stream", &stream_chunk)
            .map_err(|e| e.to_string())?;

        // Small delay to simulate streaming (removed when real inference is wired)
        if i > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }

    // Emit done signal
    app.emit(
        "chat:stream",
        &StreamChunk {
            conversation_id: conversation_id.clone(),
            content: String::new(),
            done: true,
            tokens_per_second: None,
        },
    )
    .map_err(|e| e.to_string())?;

    // Store assistant message
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        crate::memory::store_message(&conn, &conversation_id, "assistant", &response)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_generation(engine: State<'_, EngineState>) -> Result<(), String> {
    let engine_guard = engine.0.lock().map_err(|e| e.to_string())?;
    engine_guard.stop();
    Ok(())
}

#[tauri::command]
pub async fn get_conversations(db: State<'_, DbState>) -> Result<Vec<Conversation>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::memory::get_conversations(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_conversation(
    title: String,
    model_id: String,
    db: State<'_, DbState>,
) -> Result<Conversation, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::memory::create_conversation(&conn, &title, &model_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_conversation(id: String, db: State<'_, DbState>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::memory::delete_conversation(&conn, &id).map_err(|e| e.to_string())
}
