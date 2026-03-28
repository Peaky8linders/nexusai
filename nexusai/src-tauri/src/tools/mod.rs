pub mod agent;
pub mod parser;

use serde::{Deserialize, Serialize};

/// Tool definition that models can call during agent execution.
/// Follows the function-calling convention used by Llama 3.x, Qwen, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Result of executing a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Tool call parsed from model output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Built-in tools available to the agent
pub fn builtin_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "read_file".into(),
            description: "Read the contents of a file at the given path".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path to read" }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "write_file".into(),
            description: "Write content to a file at the given path".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path to write" },
                    "content": { "type": "string", "description": "Content to write" }
                },
                "required": ["path", "content"]
            }),
        },
        ToolDefinition {
            name: "list_directory".into(),
            description: "List files and directories at the given path".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory path" }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "web_fetch".into(),
            description: "Fetch the text content of a URL".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "URL to fetch" }
                },
                "required": ["url"]
            }),
        },
        ToolDefinition {
            name: "calculator".into(),
            description: "Evaluate a mathematical expression".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": { "type": "string", "description": "Math expression to evaluate" }
                },
                "required": ["expression"]
            }),
        },
    ]
}

/// Execute a tool call (Phase 2 — stub for now)
pub async fn execute_tool(call: &ToolCall) -> ToolResult {
    match call.name.as_str() {
        "read_file" => {
            let path = call.arguments["path"].as_str().unwrap_or("");
            match tokio::fs::read_to_string(path).await {
                Ok(content) => ToolResult {
                    name: call.name.clone(),
                    success: true,
                    output: content,
                    error: None,
                },
                Err(e) => ToolResult {
                    name: call.name.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                },
            }
        }
        "list_directory" => {
            let path = call.arguments["path"].as_str().unwrap_or(".");
            match tokio::fs::read_dir(path).await {
                Ok(mut entries) => {
                    let mut listing = Vec::new();
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
                        listing.push(if is_dir {
                            format!("{}/", name)
                        } else {
                            name
                        });
                    }
                    ToolResult {
                        name: call.name.clone(),
                        success: true,
                        output: listing.join("\n"),
                        error: None,
                    }
                }
                Err(e) => ToolResult {
                    name: call.name.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                },
            }
        }
        _ => ToolResult {
            name: call.name.clone(),
            success: false,
            output: String::new(),
            error: Some(format!("Tool '{}' not yet implemented", call.name)),
        },
    }
}
