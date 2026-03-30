use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::inference::{EngineState, OllamaMessage};
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

/// Helper: emit done signal to frontend
fn emit_done(app: &AppHandle, conversation_id: &str) {
    app.emit(
        "chat:stream",
        &StreamChunk {
            conversation_id: conversation_id.to_string(),
            content: String::new(),
            done: true,
            tokens_per_second: None,
        },
    )
    .ok();
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    request: SendMessageRequest,
    engine: State<'_, EngineState>,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let conversation_id = request.conversation_id.clone();
    let content = request.content.trim().to_string();

    if content.is_empty() {
        return Err("Message content is empty.".into());
    }

    tracing::info!("send_message: conv={}, content_len={}", conversation_id, content.len());

    // Store the user message
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        crate::memory::store_message(&conn, &conversation_id, "user", &content)
            .map_err(|e| format!("Failed to store user message: {}", e))?;
    }

    // Build message history from DB for context
    let ollama_messages = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let db_messages = crate::memory::get_messages(&conn, &conversation_id)
            .map_err(|e| format!("Failed to get message history: {}", e))?;

        db_messages
            .into_iter()
            .map(|m| OllamaMessage {
                role: m.role,
                content: m.content,
            })
            .collect::<Vec<_>>()
    };

    // Stream response from Ollama
    let conv_id = conversation_id.clone();
    let app_clone = app.clone();

    let response = {
        let engine_guard = engine.0.lock().await;
        match engine_guard
            .stream_chat(ollama_messages, move |token, tps| {
                let chunk = StreamChunk {
                    conversation_id: conv_id.clone(),
                    content: token.to_string(),
                    done: false,
                    tokens_per_second: tps,
                };
                app_clone.emit("chat:stream", &chunk).ok();
            })
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Inference error: {}", e);
                // Always emit done so frontend doesn't hang
                emit_done(&app, &conversation_id);
                return Err(e);
            }
        }
    };

    // Emit done signal
    emit_done(&app, &conversation_id);

    // Store assistant message
    if !response.is_empty() {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        crate::memory::store_message(&conn, &conversation_id, "assistant", &response)
            .map_err(|e| format!("Failed to store assistant message: {}", e))?;
        tracing::info!("Stored assistant response: {} chars", response.len());
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_generation(engine: State<'_, EngineState>) -> Result<(), String> {
    let engine_guard = engine.0.lock().await;
    engine_guard.stop();
    tracing::info!("Generation stop requested");
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
    let title = if title.trim().is_empty() { "New Chat".to_string() } else { title };
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::memory::create_conversation(&conn, &title, &model_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_conversation(id: String, db: State<'_, DbState>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::memory::delete_conversation(&conn, &id).map_err(|e| e.to_string())
}
