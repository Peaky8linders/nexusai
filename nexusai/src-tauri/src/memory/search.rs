//! Full-text search across conversations and memory entries.

use rusqlite::{params, Connection, Result as SqlResult};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub conversation_id: String,
    pub conversation_title: String,
    pub message_id: String,
    pub role: String,
    pub content_snippet: String,
    pub timestamp: i64,
    pub score: f64,
}

/// Search across all conversation messages using simple keyword matching.
/// Phase 5+: Replace with SQLite FTS5 for proper full-text search.
pub fn search_conversations(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> SqlResult<Vec<SearchHit>> {
    let like_pattern = format!("%{}%", query.replace('%', "\\%"));

    let mut stmt = conn.prepare(
        "SELECT m.id, m.conversation_id, m.role, m.content, m.timestamp, c.title
         FROM messages m
         JOIN conversations c ON c.id = m.conversation_id
         WHERE m.content LIKE ?1
         ORDER BY m.timestamp DESC
         LIMIT ?2",
    )?;

    let hits = stmt
        .query_map(params![like_pattern, limit as i64], |row| {
            let content: String = row.get(3)?;
            let query_lower = query.to_lowercase();
            let content_lower = content.to_lowercase();

            // Extract a snippet around the match
            let snippet = if let Some(pos) = content_lower.find(&query_lower) {
                let start = pos.saturating_sub(50);
                let end = (pos + query.len() + 50).min(content.len());
                let mut s = String::new();
                if start > 0 { s.push_str("..."); }
                s.push_str(&content[start..end]);
                if end < content.len() { s.push_str("..."); }
                s
            } else {
                content[..content.len().min(100)].to_string()
            };

            Ok(SearchHit {
                message_id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content_snippet: snippet,
                timestamp: row.get(4)?,
                conversation_title: row.get(5)?,
                score: 1.0, // placeholder — FTS5 would provide real scores
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;

    Ok(hits)
}

/// Store a memory entry (user fact, preference, or extracted context)
pub fn store_memory(
    conn: &Connection,
    key: &str,
    value: &str,
    category: &str,
) -> SqlResult<()> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO user_memory (id, key, value, category, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(key) DO UPDATE SET value = ?3, category = ?4, updated_at = ?6",
        params![id, key, value, category, now, now],
    )?;

    Ok(())
}

/// Retrieve all memory entries, optionally filtered by category
pub fn get_memories(
    conn: &Connection,
    category: Option<&str>,
) -> SqlResult<Vec<(String, String, String, String, i64)>> {
    let sql = if category.is_some() {
        "SELECT id, key, value, category, updated_at FROM user_memory WHERE category = ?1 ORDER BY updated_at DESC"
    } else {
        "SELECT id, key, value, category, updated_at FROM user_memory ORDER BY updated_at DESC"
    };

    let mut stmt = conn.prepare(sql)?;

    let rows = if let Some(cat) = category {
        stmt.query_map(params![cat], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?
        .collect::<SqlResult<Vec<_>>>()?
    } else {
        stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?
        .collect::<SqlResult<Vec<_>>>()?
    };

    Ok(rows)
}

/// Delete a memory entry
pub fn delete_memory(conn: &Connection, id: &str) -> SqlResult<()> {
    conn.execute("DELETE FROM user_memory WHERE id = ?1", params![id])?;
    Ok(())
}

/// Build a memory context string for prompt injection.
/// Retrieves relevant memories and formats them for the system prompt.
pub fn build_memory_context(conn: &Connection) -> SqlResult<String> {
    let memories = get_memories(conn, None)?;
    if memories.is_empty() {
        return Ok(String::new());
    }

    let mut context = String::from("## User Memory\n\n");

    let facts: Vec<_> = memories.iter().filter(|m| m.3 == "fact").collect();
    let prefs: Vec<_> = memories.iter().filter(|m| m.3 == "preference").collect();

    if !facts.is_empty() {
        context.push_str("### Known Facts\n");
        for (_, key, value, _, _) in &facts {
            context.push_str(&format!("- {}: {}\n", key, value));
        }
        context.push('\n');
    }

    if !prefs.is_empty() {
        context.push_str("### User Preferences\n");
        for (_, key, value, _, _) in &prefs {
            context.push_str(&format!("- {}: {}\n", key, value));
        }
        context.push('\n');
    }

    Ok(context)
}
