//! Tool registration and execution

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{McpError, McpResult};
use crate::protocol::ToolDefinition;

/// Tool handler function type
pub type ToolHandler = Arc<dyn Fn(Value) -> McpResult<Value> + Send + Sync>;

/// Async tool handler function type
pub type AsyncToolHandler = Arc<
    dyn Fn(Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = McpResult<Value>> + Send>>
        + Send
        + Sync,
>;

/// Tool parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParam {
    /// Parameter name
    pub name: String,

    /// Parameter type (JSON Schema type)
    #[serde(rename = "type")]
    pub param_type: String,

    /// Parameter description
    pub description: String,

    /// Whether the parameter is required
    #[serde(default)]
    pub required: bool,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

/// MCP Tool
#[derive(Clone)]
pub struct Tool {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Tool parameters
    pub params: Vec<ToolParam>,

    /// Output schema (JSON Schema) for structured responses
    pub output_schema: Option<Value>,

    /// Hint that the tool does not modify its environment
    pub read_only_hint: Option<bool>,

    /// Hint that the tool may perform destructive updates
    pub destructive_hint: Option<bool>,

    /// Hint that repeated calls with same args have no additional effect
    pub idempotent_hint: Option<bool>,

    /// Hint that the tool interacts with external entities
    pub open_world_hint: Option<bool>,

    /// Synchronous handler
    handler: Option<ToolHandler>,

    /// Asynchronous handler
    async_handler: Option<AsyncToolHandler>,
}

impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("params", &self.params)
            .field("has_handler", &self.handler.is_some())
            .field("has_async_handler", &self.async_handler.is_some())
            .finish()
    }
}

impl Tool {
    /// Create a new tool
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            params: Vec::new(),
            output_schema: None,
            read_only_hint: None,
            destructive_hint: None,
            idempotent_hint: None,
            open_world_hint: None,
            handler: None,
            async_handler: None,
        }
    }

    /// Add a parameter
    pub fn with_param(
        mut self,
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.params.push(ToolParam {
            name: name.into(),
            param_type: param_type.into(),
            description: description.into(),
            required: true,
            default: None,
        });
        self
    }

    /// Add an optional parameter
    pub fn with_optional_param(
        mut self,
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
        default: Option<Value>,
    ) -> Self {
        self.params.push(ToolParam {
            name: name.into(),
            param_type: param_type.into(),
            description: description.into(),
            required: false,
            default,
        });
        self
    }

    /// Set synchronous handler
    pub fn with_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(Value) -> McpResult<Value> + Send + Sync + 'static,
    {
        self.handler = Some(Arc::new(handler));
        self
    }

    /// Set asynchronous handler
    pub fn with_async_handler<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = McpResult<Value>> + Send + 'static,
    {
        self.async_handler = Some(Arc::new(move |args| {
            let fut = handler(args);
            Box::pin(fut)
        }));
        self
    }

    /// Set output schema for structured responses
    pub fn with_output_schema(mut self, schema: Value) -> Self {
        self.output_schema = Some(schema);
        self
    }

    /// Set read-only hint
    pub fn read_only(mut self) -> Self {
        self.read_only_hint = Some(true);
        self
    }

    /// Set destructive hint
    pub fn destructive(mut self) -> Self {
        self.destructive_hint = Some(true);
        self
    }

    /// Set idempotent hint
    pub fn idempotent(mut self) -> Self {
        self.idempotent_hint = Some(true);
        self
    }

    /// Set open-world hint
    pub fn open_world(mut self) -> Self {
        self.open_world_hint = Some(true);
        self
    }

    /// Call the tool with arguments
    pub fn call(&self, args: Value) -> McpResult<Value> {
        if let Some(handler) = &self.handler {
            handler(args)
        } else {
            Err(McpError::Internal("Tool has no synchronous handler configured. Please add a handler using .with_handler() or .with_async_handler()".to_string()))
        }
    }

    /// Call the tool asynchronously
    pub async fn call_async(&self, args: Value) -> McpResult<Value> {
        if let Some(handler) = &self.async_handler {
            handler(args).await
        } else if let Some(handler) = &self.handler {
            // Fall back to sync handler - use spawn_blocking to avoid blocking
            // the async runtime (critical for Python GIL operations)
            let handler = Arc::clone(handler);
            tokio::task::spawn_blocking(move || handler(args))
                .await
                .map_err(|e| McpError::Internal(format!("Task join error: {}", e)))?
        } else {
            Err(McpError::Internal("Tool has no handler configured. Please add a handler using .with_handler() or .with_async_handler()".to_string()))
        }
    }

    /// Convert to MCP tool definition
    pub fn to_definition(&self) -> ToolDefinition {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &self.params {
            let mut prop = serde_json::Map::new();
            prop.insert("type".to_string(), Value::String(param.param_type.clone()));
            prop.insert(
                "description".to_string(),
                Value::String(param.description.clone()),
            );

            if let Some(default) = &param.default {
                prop.insert("default".to_string(), default.clone());
            }

            properties.insert(param.name.clone(), Value::Object(prop));

            if param.required {
                required.push(Value::String(param.name.clone()));
            }
        }

        let input_schema = serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required
        });

        ToolDefinition {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema,
            output_schema: self.output_schema.clone(),
            read_only_hint: self.read_only_hint,
            destructive_hint: self.destructive_hint,
            idempotent_hint: self.idempotent_hint,
            open_world_hint: self.open_world_hint,
        }
    }

    /// Check if tool has any handler
    pub fn has_handler(&self) -> bool {
        self.handler.is_some() || self.async_handler.is_some()
    }
}

/// Tool registry for managing registered tools
#[derive(Default)]
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Tool>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    /// Register a tool
    pub fn register(&self, tool: Tool) {
        let name = tool.name.clone();
        self.tools.write().insert(name, tool);
    }

    /// Unregister a tool
    pub fn unregister(&self, name: &str) -> Option<Tool> {
        self.tools.write().remove(name)
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Tool> {
        self.tools.read().get(name).cloned()
    }

    /// Check if a tool exists
    pub fn contains(&self, name: &str) -> bool {
        self.tools.read().contains_key(name)
    }

    /// List all tool names
    pub fn list(&self) -> Vec<String> {
        self.tools.read().keys().cloned().collect()
    }

    /// Get all tool definitions
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .read()
            .values()
            .map(|t| t.to_definition())
            .collect()
    }

    /// Call a tool by name
    pub fn call(&self, name: &str, args: Value) -> McpResult<Value> {
        let tool = self.get(name).ok_or_else(|| McpError::ToolNotFound {
            name: name.to_string(),
            available: format!("[{}]", self.list().join(", ")),
        })?;
        tool.call(args)
    }

    /// Call a tool asynchronously
    pub async fn call_async(&self, name: &str, args: Value) -> McpResult<Value> {
        let tool = self.get(name).ok_or_else(|| McpError::ToolNotFound {
            name: name.to_string(),
            available: format!("[{}]", self.list().join(", ")),
        })?;
        tool.call_async(args).await
    }

    /// Get tool count
    pub fn len(&self) -> usize {
        self.tools.read().len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.read().is_empty()
    }
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tools", &self.list())
            .finish()
    }
}

/// Create a tool from a function with typed parameters
///
/// This is a helper for creating tools from functions that take typed parameters.
pub fn tool_from_fn<T, R, F>(name: &str, description: &str, handler: F) -> Tool
where
    T: for<'de> Deserialize<'de> + JsonSchema,
    R: Serialize,
    F: Fn(T) -> McpResult<R> + Send + Sync + 'static,
{
    // Clone name for use in closure
    let tool_name = name.to_string();

    // Generate JSON Schema from type
    let schema = schemars::schema_for!(T);
    let schema_value = serde_json::to_value(&schema.schema).unwrap_or_default();

    let mut tool = Tool::new(name, description);

    // Extract properties from schema
    if let Some(obj) = schema_value.get("properties").and_then(|v| v.as_object()) {
        for (prop_name, prop_schema) in obj {
            let prop_type = prop_schema
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("string");
            let prop_desc = prop_schema
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            tool = tool.with_param(prop_name, prop_type, prop_desc);
        }
    }

    tool.with_handler(move |args| {
        let params: T = serde_json::from_value(args)
            .map_err(|e| McpError::InvalidArguments {
                tool: tool_name.clone(),
                reason: format!("Failed to parse arguments: {}", e),
                suggestion: "Ensure all required parameters are provided and match the expected types. Check the tool's input schema for details.".to_string(),
            })?;
        let result = handler(params)?;
        serde_json::to_value(result).map_err(McpError::from)
    })
}

/// Prompt handler function type
pub type PromptHandler = Arc<
    dyn Fn(serde_json::Map<String, Value>) -> McpResult<crate::protocol::GetPromptResult>
        + Send
        + Sync,
>;

/// MCP Prompt
#[derive(Clone)]
pub struct Prompt {
    /// Prompt name
    pub name: String,

    /// Prompt description
    pub description: String,

    /// Prompt arguments
    pub arguments: Vec<crate::protocol::PromptArgument>,

    /// Synchronous handler
    handler: Option<PromptHandler>,
}

impl Prompt {
    /// Create a new prompt
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            arguments: Vec::new(),
            handler: None,
        }
    }

    /// Add an argument to the prompt
    pub fn with_argument(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.arguments.push(crate::protocol::PromptArgument {
            name: name.into(),
            description: description.into(),
            required: Some(false),
        });
        self
    }

    /// Add a required argument to the prompt
    pub fn with_required_argument(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.arguments.push(crate::protocol::PromptArgument {
            name: name.into(),
            description: description.into(),
            required: Some(true),
        });
        self
    }

    /// Set the prompt handler
    pub fn with_handler(mut self, handler: PromptHandler) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Execute the prompt with given arguments
    pub fn execute(
        &self,
        args: serde_json::Map<String, Value>,
    ) -> McpResult<crate::protocol::GetPromptResult> {
        match &self.handler {
            Some(handler) => handler(args),
            None => Err(McpError::Internal(format!(
                "Prompt '{}' has no handler registered",
                self.name
            ))),
        }
    }
}

/// Prompt registry
#[derive(Clone, Default)]
pub struct PromptRegistry {
    prompts: HashMap<String, Prompt>,
}

impl PromptRegistry {
    /// Create a new prompt registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a prompt
    pub fn register(&mut self, prompt: Prompt) -> McpResult<()> {
        if self.prompts.contains_key(&prompt.name) {
            return Err(McpError::Internal(format!(
                "Prompt '{}' is already registered",
                prompt.name
            )));
        }
        self.prompts.insert(prompt.name.clone(), prompt);
        Ok(())
    }

    /// Get a prompt by name
    pub fn get(&self, name: &str) -> Option<&Prompt> {
        self.prompts.get(name)
    }

    /// Get all prompt definitions
    pub fn definitions(&self) -> Vec<crate::protocol::PromptDefinition> {
        self.prompts
            .values()
            .map(|p| crate::protocol::PromptDefinition {
                name: p.name.clone(),
                description: p.description.clone(),
                arguments: Some(p.arguments.clone()),
            })
            .collect()
    }

    /// List all prompt names
    pub fn list(&self) -> Vec<String> {
        self.prompts.keys().cloned().collect()
    }

    /// Get the number of prompts
    pub fn len(&self) -> usize {
        self.prompts.len()
    }

    /// Check if a prompt exists
    pub fn contains(&self, name: &str) -> bool {
        self.prompts.contains_key(name)
    }

    /// Execute a prompt by name
    pub fn execute(
        &self,
        name: &str,
        args: serde_json::Map<String, Value>,
    ) -> McpResult<crate::protocol::GetPromptResult> {
        let prompt = self.get(name).ok_or_else(|| McpError::ToolNotFound {
            name: name.to_string(),
            available: format!("[{}]", self.list().join(", ")),
        })?;
        prompt.execute(args)
    }
}
