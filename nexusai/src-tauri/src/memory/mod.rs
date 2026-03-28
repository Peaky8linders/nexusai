pub mod search;

use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;
use std::sync::Mutex;

use crate::commands::chat::{Conversation, Message};

pub struct DbState(pub Mutex<Connection>);

pub fn init_db(path: &Path) -> SqlResult<Connection> {
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            model_id TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_messages_conv ON messages(conversation_id, timestamp);

        CREATE TABLE IF NOT EXISTS user_memory (
            id TEXT PRIMARY KEY,
            key TEXT NOT NULL UNIQUE,
            value TEXT NOT NULL,
            category TEXT NOT NULL DEFAULT 'general',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        ",
    )?;

    Ok(conn)
}

pub fn create_conversation(
    conn: &Connection,
    title: &str,
    model_id: &str,
) -> SqlResult<Conversation> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO conversations (id, title, model_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, title, model_id, now, now],
    )?;

    Ok(Conversation {
        id,
        title: title.to_string(),
        created_at: now,
        updated_at: now,
        model_id: model_id.to_string(),
        messages: vec![],
    })
}

pub fn get_conversations(conn: &Connection) -> SqlResult<Vec<Conversation>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, model_id, created_at, updated_at FROM conversations ORDER BY updated_at DESC",
    )?;

    let convos = stmt
        .query_map([], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                model_id: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                messages: vec![],
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;

    Ok(convos)
}

pub fn delete_conversation(conn: &Connection, id: &str) -> SqlResult<()> {
    conn.execute("DELETE FROM messages WHERE conversation_id = ?1", params![id])?;
    conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn store_message(
    conn: &Connection,
    conversation_id: &str,
    role: &str,
    content: &str,
) -> SqlResult<()> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, conversation_id, role, content, now],
    )?;

    conn.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        params![now, conversation_id],
    )?;

    Ok(())
}

pub fn get_messages(conn: &Connection, conversation_id: &str) -> SqlResult<Vec<Message>> {
    let mut stmt = conn.prepare(
        "SELECT id, role, content, timestamp FROM messages WHERE conversation_id = ?1 ORDER BY timestamp ASC",
    )?;

    let messages = stmt
        .query_map(params![conversation_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;

    Ok(messages)
}
