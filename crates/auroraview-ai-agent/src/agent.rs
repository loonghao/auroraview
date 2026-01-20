//! AI Agent core implementation
//!
//! Provides a high-level interface for AI-powered browser control
//! using the genai crate for multi-provider support and AG-UI protocol
//! for standardized AI-UI communication.

use crate::actions::{ActionContext, ActionRegistry, ActionResult};
use crate::error::{AIError, AIResult};
use crate::protocol::agui::{AGUIEmitter, AGUIEvent, CallbackEmitter, NoOpEmitter};
use crate::providers::{AIClient, ChatOptions, ProviderType};
use crate::session::{ChatSession, SessionManager};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// AI Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// Model name (e.g., "gpt-4o", "claude-3-5-sonnet-20241022")
    pub model: String,

    /// Temperature (0.0 - 2.0)
    pub temperature: f32,

    /// Max tokens for response
    pub max_tokens: u32,

    /// System prompt
    pub system_prompt: Option<String>,

    /// Enable streaming
    pub stream: bool,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            system_prompt: None,
            stream: true,
        }
    }
}

impl AIConfig {
    /// Create config for a specific model
    pub fn for_model(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    /// Create config for OpenAI
    pub fn openai() -> Self {
        Self::for_model("gpt-4o")
    }

    /// Create config for Anthropic Claude
    pub fn anthropic() -> Self {
        Self::for_model("claude-3-5-sonnet-20241022")
    }

    /// Create config for Google Gemini
    pub fn gemini() -> Self {
        Self::for_model("gemini-2.0-flash-exp")
    }

    /// Create config for DeepSeek
    pub fn deepseek() -> Self {
        Self::for_model("deepseek-chat")
    }

    /// Create config for local Ollama
    pub fn ollama(model: &str) -> Self {
        Self::for_model(model)
    }

    /// Set temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp.clamp(0.0, 2.0);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = tokens;
        self
    }

    /// Set system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Enable/disable streaming
    pub fn with_streaming(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Get inferred provider type
    pub fn provider_type(&self) -> ProviderType {
        AIClient::infer_provider(&self.model)
    }
}

/// AI Agent for natural language browser control
///
/// The AI Agent provides a high-level interface for AI-powered interactions,
/// supporting multiple AI providers through the genai crate and emitting
/// AG-UI protocol events for standardized UI updates.
///
/// # Example
///
/// ```rust,ignore
/// use auroraview_ai_agent::{AIAgent, AIConfig};
///
/// let config = AIConfig::openai()
///     .with_system_prompt("You are a helpful browser assistant.");
///
/// let agent = AIAgent::new(config);
///
/// // Simple chat
/// let response = agent.chat("Open GitHub").await?;
///
/// // Chat with streaming events
/// agent.chat_with_events("Search for Rust tutorials", |event| {
///     println!("{:?}", event);
/// }).await?;
/// ```
pub struct AIAgent {
    config: AIConfig,
    client: AIClient,
    sessions: Arc<RwLock<SessionManager>>,
    actions: Arc<RwLock<ActionRegistry>>,
}

impl AIAgent {
    /// Create a new AI Agent with configuration
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            client: AIClient::new(),
            sessions: Arc::new(RwLock::new(SessionManager::new())),
            actions: Arc::new(RwLock::new(ActionRegistry::with_defaults())),
        }
    }

    /// Create agent with custom AI client
    pub fn with_client(config: AIConfig, client: AIClient) -> Self {
        Self {
            config,
            client,
            sessions: Arc::new(RwLock::new(SessionManager::new())),
            actions: Arc::new(RwLock::new(ActionRegistry::with_defaults())),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &AIConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: AIConfig) {
        self.config = config;
    }

    /// Set the model
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.config.model = model.into();
    }

    /// Send a chat message (non-streaming)
    pub async fn chat(&self, message: &str) -> AIResult<String> {
        let emitter = NoOpEmitter;
        self.chat_internal(message, &emitter).await
    }

    /// Send a chat message with AG-UI event callback
    pub async fn chat_with_events<F>(&self, message: &str, on_event: F) -> AIResult<String>
    where
        F: Fn(AGUIEvent) + Send + Sync + 'static,
    {
        let emitter = CallbackEmitter::new(on_event);
        self.chat_internal(message, &emitter).await
    }

    /// Internal chat implementation
    async fn chat_internal<E: AGUIEmitter>(&self, message: &str, emitter: &E) -> AIResult<String> {
        // Get or create session
        let mut sessions = self.sessions.write().await;
        if sessions.active_session().is_none() {
            let session = sessions.new_session();
            if let Some(ref prompt) = self.config.system_prompt {
                session.system_prompt = Some(prompt.clone());
            }
        }

        let session = sessions.active_session_mut().unwrap();
        session.add_user_message(message);

        // Generate run and message IDs
        let run_id = uuid::Uuid::new_v4().to_string();
        let thread_id = session.id.clone();
        let message_id = uuid::Uuid::new_v4().to_string();

        // Emit run started
        emitter.run_started(&run_id, &thread_id);

        // Build messages for API - collect to owned strings due to lifetime constraints
        let api_messages: Vec<(String, String)> = session
            .get_messages_for_api()
            .iter()
            .map(|m| {
                let role = match m.role {
                    crate::message::MessageRole::System => "system".to_string(),
                    crate::message::MessageRole::User => "user".to_string(),
                    crate::message::MessageRole::Assistant => "assistant".to_string(),
                    crate::message::MessageRole::Tool => "tool".to_string(),
                };
                (role, m.content.as_text())
            })
            .collect();

        // Convert to references for the API call
        let api_messages_ref: Vec<(&str, &str)> = api_messages
            .iter()
            .map(|(r, c)| (r.as_str(), c.as_str()))
            .collect();

        // Build options
        let options = ChatOptions {
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            top_p: None,
            stop: None,
        };

        // Execute non-streaming chat (streaming handled separately)
        emitter.text_start(&message_id, "assistant");

        let response = self
            .client
            .chat_with_options(&self.config.model, api_messages_ref, options)
            .await;

        let result = match response {
            Ok(r) => {
                let content = r.content.unwrap_or_default();
                emitter.text_delta(&message_id, &content);
                emitter.text_end(&message_id);
                content
            }
            Err(e) => {
                emitter.run_error(&run_id, &e.to_string());
                return Err(e);
            }
        };

        // Add assistant message to session
        session.add_assistant_message(&result);

        // Emit run finished
        emitter.run_finished(&run_id, &thread_id);

        Ok(result)
    }

    /// Execute an action by name
    pub async fn execute_action(&self, name: &str, arguments: &str) -> AIResult<ActionResult> {
        let actions = self.actions.read().await;
        let action = actions
            .get(name)
            .ok_or_else(|| AIError::ActionNotFound(name.to_string()))?;

        let args: serde_json::Value =
            serde_json::from_str(arguments).map_err(|e| AIError::ParseError(e.to_string()))?;

        let ctx = ActionContext::default();
        action
            .execute(args, &ctx)
            .map_err(|e| AIError::ActionExecutionFailed(e.to_string()))
    }

    /// Register a custom action
    pub async fn register_action<A: crate::actions::Action + 'static>(&self, action: A) {
        let mut actions = self.actions.write().await;
        actions.register(action);
    }

    /// Get available action names
    pub async fn action_names(&self) -> Vec<String> {
        let actions = self.actions.read().await;
        actions.names().iter().map(|s| s.to_string()).collect()
    }

    /// Get current session
    pub async fn current_session(&self) -> Option<ChatSession> {
        let sessions = self.sessions.read().await;
        sessions.active_session().cloned()
    }

    /// Create new session
    pub async fn new_session(&self) -> ChatSession {
        let mut sessions = self.sessions.write().await;
        let session = sessions.new_session();
        if let Some(ref prompt) = self.config.system_prompt {
            session.system_prompt = Some(prompt.clone());
        }
        session.clone()
    }

    /// Clear current session
    pub async fn clear_session(&self) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.active_session_mut() {
            session.clear();
        }
    }

    /// Get all sessions
    pub async fn all_sessions(&self) -> Vec<ChatSession> {
        let sessions = self.sessions.read().await;
        sessions.all_sessions().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = AIConfig::openai()
            .with_temperature(0.5)
            .with_max_tokens(2048)
            .with_system_prompt("You are a helpful assistant.");

        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, 2048);
        assert!(config.system_prompt.is_some());
    }

    #[test]
    fn test_config_providers() {
        assert_eq!(AIConfig::openai().provider_type(), ProviderType::OpenAI);
        assert_eq!(
            AIConfig::anthropic().provider_type(),
            ProviderType::Anthropic
        );
        assert_eq!(AIConfig::gemini().provider_type(), ProviderType::Gemini);
        assert_eq!(AIConfig::deepseek().provider_type(), ProviderType::DeepSeek);
    }

    #[test]
    fn test_agent_creation() {
        let config = AIConfig::openai();
        let agent = AIAgent::new(config);

        assert_eq!(agent.config().model, "gpt-4o");
    }
}
