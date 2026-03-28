//! Agent execution loop — orchestrates multi-step tool calling.
//!
//! The agent loop:
//! 1. Send prompt + tool definitions to the model
//! 2. Parse tool calls from output
//! 3. Check if approval needed → emit approval_needed event
//! 4. Execute approved tool calls
//! 5. Feed results back to model
//! 6. Repeat until model generates final text (no tool calls)

use super::{execute_tool, parser, ToolCall, ToolDefinition, ToolResult};
use serde::{Deserialize, Serialize};

/// Agent configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub max_steps: usize,
    pub auto_approve_safe: bool,
    pub timeout_per_step_ms: u64,
    pub tools: Vec<ToolDefinition>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_steps: 10,
            auto_approve_safe: true,
            timeout_per_step_ms: 30_000,
            tools: super::builtin_tools(),
        }
    }
}

/// Status of a pending tool call awaiting approval
#[derive(Debug, Clone, Serialize)]
pub struct PendingApproval {
    pub step: usize,
    pub tool_call: ToolCall,
    pub reason: String,
}

/// Result of an agent step
#[derive(Debug, Clone, Serialize)]
pub enum AgentStepResult {
    /// Model generated text with no tool calls — agent is done
    FinalResponse(String),
    /// Tool calls need user approval before execution
    NeedsApproval(Vec<PendingApproval>),
    /// Tool calls executed, results ready for next model turn
    ToolResults(Vec<ToolResult>),
    /// Agent hit max steps limit
    MaxStepsReached(String),
    /// Agent encountered an error
    Error(String),
}

/// Agent session tracking the multi-step execution
pub struct AgentSession {
    config: AgentConfig,
    messages: Vec<(String, String)>, // (role, content) history
    step_count: usize,
    tool_history: Vec<(ToolCall, ToolResult)>,
}

impl AgentSession {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            messages: Vec::new(),
            step_count: 0,
            tool_history: Vec::new(),
        }
    }

    /// Process model output — parse tool calls and determine next action
    pub fn process_output(&mut self, model_output: &str) -> AgentStepResult {
        self.step_count += 1;

        if self.step_count > self.config.max_steps {
            return AgentStepResult::MaxStepsReached(format!(
                "Agent reached maximum of {} steps. Last output:\n{}",
                self.config.max_steps, model_output
            ));
        }

        // Parse tool calls from model output
        let tool_calls = parser::parse_tool_calls(model_output);

        if tool_calls.is_empty() {
            // No tool calls — model is done, return final response
            return AgentStepResult::FinalResponse(model_output.to_string());
        }

        // Check which calls need approval
        let mut needs_approval = Vec::new();
        let mut auto_approved = Vec::new();

        for call in tool_calls {
            if self.config.auto_approve_safe && !parser::requires_approval(&call) {
                auto_approved.push(call);
            } else {
                needs_approval.push(PendingApproval {
                    step: self.step_count,
                    tool_call: call,
                    reason: "This tool modifies files or executes commands".into(),
                });
            }
        }

        if !needs_approval.is_empty() {
            // Some calls need approval — pause and ask user
            // Auto-approved calls are queued for after approval
            return AgentStepResult::NeedsApproval(needs_approval);
        }

        // All calls auto-approved — execute them
        AgentStepResult::ToolResults(Vec::new()) // placeholder — async execution happens in command handler
    }

    /// Execute a list of approved tool calls
    pub async fn execute_calls(&mut self, calls: &[ToolCall]) -> Vec<ToolResult> {
        let mut results = Vec::new();

        for call in calls {
            let result = execute_tool(call).await;
            self.tool_history.push((call.clone(), result.clone()));
            results.push(result);
        }

        results
    }

    /// Build the system prompt with tool definitions
    pub fn system_prompt(&self) -> String {
        let mut prompt = String::from(
            "You are NexusAI, a local AI assistant with access to tools. \
             When you need to perform an action, use the available tools. \
             Always explain what you're going to do before calling a tool.\n\n\
             Available tools:\n",
        );

        for tool in &self.config.tools {
            prompt.push_str(&format!(
                "\n### {}\n{}\nParameters: {}\n",
                tool.name,
                tool.description,
                serde_json::to_string_pretty(&tool.parameters).unwrap_or_default()
            ));
        }

        prompt.push_str(
            "\nTo call a tool, use this format:\n\
             <tool_call>\n\
             {\"name\": \"tool_name\", \"arguments\": {\"param\": \"value\"}}\n\
             </tool_call>\n",
        );

        prompt
    }

    /// Add a message to the conversation history
    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push((role.to_string(), content.to_string()));
    }

    /// Get full message history including tool results
    pub fn messages(&self) -> &[(String, String)] {
        &self.messages
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }

    pub fn tool_history(&self) -> &[(ToolCall, ToolResult)] {
        &self.tool_history
    }
}
