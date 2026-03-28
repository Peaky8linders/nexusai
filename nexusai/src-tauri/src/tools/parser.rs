//! Parse tool/function calls from LLM output.
//!
//! Supports multiple formats:
//! - Llama 3.x: <|python_tag|> and <function=name>{json}</function>
//! - Qwen/ChatML: <tool_call>{"name": ..., "arguments": ...}</tool_call>
//! - Generic JSON: {"tool_calls": [...]}

use super::{ToolCall, ToolResult};
use serde_json::Value;

/// Extract tool calls from model output text
pub fn parse_tool_calls(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();

    // Try Llama 3.x format: <function=name>{"arg": "val"}</function>
    calls.extend(parse_llama3_format(text));

    // Try Qwen/ChatML format: <tool_call>...</tool_call>
    if calls.is_empty() {
        calls.extend(parse_chatml_format(text));
    }

    // Try generic JSON format
    if calls.is_empty() {
        calls.extend(parse_json_format(text));
    }

    calls
}

fn parse_llama3_format(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("<function=") {
        let after_tag = &remaining[start + 10..];
        if let Some(name_end) = after_tag.find('>') {
            let name = &after_tag[..name_end];
            let body_start = &after_tag[name_end + 1..];

            if let Some(end) = body_start.find("</function>") {
                let json_str = &body_start[..end];
                if let Ok(args) = serde_json::from_str::<Value>(json_str) {
                    calls.push(ToolCall {
                        name: name.to_string(),
                        arguments: args,
                    });
                }
                remaining = &body_start[end + 11..];
            } else {
                break;
            }
        } else {
            break;
        }
    }

    calls
}

fn parse_chatml_format(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("<tool_call>") {
        let body = &remaining[start + 11..];
        if let Some(end) = body.find("</tool_call>") {
            let json_str = &body[..end].trim();
            if let Ok(val) = serde_json::from_str::<Value>(json_str) {
                if let (Some(name), Some(args)) = (
                    val.get("name").and_then(|n| n.as_str()),
                    val.get("arguments"),
                ) {
                    calls.push(ToolCall {
                        name: name.to_string(),
                        arguments: args.clone(),
                    });
                }
            }
            remaining = &body[end + 12..];
        } else {
            break;
        }
    }

    calls
}

fn parse_json_format(text: &str) -> Vec<ToolCall> {
    // Look for JSON objects with "tool_calls" or "function_call" keys
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('{') {
            continue;
        }

        if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
            // {"tool_calls": [{"name": ..., "arguments": ...}]}
            if let Some(tool_calls) = val.get("tool_calls").and_then(|t| t.as_array()) {
                return tool_calls
                    .iter()
                    .filter_map(|tc| {
                        let name = tc.get("name")?.as_str()?;
                        let args = tc.get("arguments")?;
                        Some(ToolCall {
                            name: name.to_string(),
                            arguments: args.clone(),
                        })
                    })
                    .collect();
            }

            // Single function call: {"name": ..., "arguments": ...}
            if let (Some(name), Some(args)) = (
                val.get("name").and_then(|n| n.as_str()),
                val.get("arguments"),
            ) {
                return vec![ToolCall {
                    name: name.to_string(),
                    arguments: args.clone(),
                }];
            }
        }
    }

    Vec::new()
}

/// Check if a tool call requires user approval (dangerous operations)
pub fn requires_approval(call: &ToolCall) -> bool {
    match call.name.as_str() {
        "write_file" | "shell_exec" | "web_fetch" => true,
        "read_file" | "list_directory" | "calculator" => false,
        _ => true, // Unknown tools require approval by default
    }
}

/// Format a tool result for injection back into the model prompt
pub fn format_tool_result(result: &ToolResult) -> String {
    if result.success {
        format!(
            "<tool_response>\n{{\n  \"name\": \"{}\",\n  \"output\": {}\n}}\n</tool_response>",
            result.name,
            serde_json::to_string(&result.output).unwrap_or_default()
        )
    } else {
        format!(
            "<tool_response>\n{{\n  \"name\": \"{}\",\n  \"error\": {}\n}}\n</tool_response>",
            result.name,
            serde_json::to_string(&result.error).unwrap_or_default()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_llama3_format() {
        let text = r#"I'll read that file for you.
<function=read_file>{"path": "/tmp/test.txt"}</function>"#;
        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_file");
        assert_eq!(calls[0].arguments["path"], "/tmp/test.txt");
    }

    #[test]
    fn test_parse_chatml_format() {
        let text = r#"<tool_call>
{"name": "calculator", "arguments": {"expression": "2 + 2"}}
</tool_call>"#;
        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "calculator");
    }

    #[test]
    fn test_approval_check() {
        let safe = ToolCall {
            name: "read_file".into(),
            arguments: serde_json::json!({"path": "/tmp/test"}),
        };
        let dangerous = ToolCall {
            name: "shell_exec".into(),
            arguments: serde_json::json!({"command": "rm -rf /"}),
        };
        assert!(!requires_approval(&safe));
        assert!(requires_approval(&dangerous));
    }
}
