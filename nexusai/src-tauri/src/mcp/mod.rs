use serde::{Deserialize, Serialize};

/// MCP (Model Context Protocol) server configuration.
/// Phase 3 feature — stubbed here for architecture completeness.
///
/// MCP allows external tools to register with NexusAI and be called by the model.
/// Each MCP server exposes a set of tools via JSON-RPC over stdio or HTTP.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub server_id: String,
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// MCP registry — tracks installed MCP servers and their tools
pub struct McpRegistry {
    servers: Vec<McpServer>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    pub fn register_server(&mut self, server: McpServer) {
        self.servers.push(server);
    }

    pub fn list_servers(&self) -> &[McpServer] {
        &self.servers
    }

    /// Discover tools from all enabled MCP servers.
    /// Phase 3: will actually spawn each server process and query via JSON-RPC.
    pub async fn discover_tools(&self) -> Vec<McpTool> {
        // Stub — returns empty for now
        Vec::new()
    }
}
