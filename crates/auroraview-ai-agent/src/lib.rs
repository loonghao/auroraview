//! AI Agent integration for AuroraView
//!
//! This crate provides AI agent capabilities for AuroraView, enabling natural language
//! control of the browser and WebView. It uses the genai crate for multi-provider
//! support and implements AG-UI/A2UI protocols for standardized AI-frontend communication.
//!
//! # Features
//!
//! - **Multi-provider support** via genai crate:
//!   - OpenAI (GPT-4o, GPT-4, O1, etc.)
//!   - Anthropic (Claude 3.5, 3.7, etc.)
//!   - Google Gemini
//!   - DeepSeek (including R1 reasoning model)
//!   - Ollama (local models)
//!   - Groq, xAI (Grok), Cohere, and more
//!
//! - **AG-UI Protocol compliance** for AI-UI interaction:
//!   - Text streaming events
//!   - Tool call lifecycle events
//!   - State synchronization
//!   - Thinking/reasoning events (for models like DeepSeek R1, O1)
//!
//! - **A2UI Protocol** for dynamic UI generation:
//!   - Component specifications
//!   - UI actions (render, update, notify)
//!
//! - **Browser control actions**:
//!   - Navigate, search, click, type, scroll
//!   - Screenshot capture
//!   - Custom action registration
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview_ai_agent::{AIAgent, AIConfig};
//!
//! // Create agent with OpenAI
//! let agent = AIAgent::new(AIConfig::openai());
//!
//! // Simple chat
//! let response = agent.chat("Hello!").await?;
//!
//! // Chat with AG-UI events
//! agent.chat_with_events("Search for Rust tutorials", |event| {
//!     match event {
//!         AGUIEvent::TextMessageContent { delta, .. } => print!("{}", delta),
//!         AGUIEvent::RunFinished { .. } => println!("\n--- Done ---"),
//!         _ => {}
//!     }
//! }).await?;
//!
//! // Direct AI client usage
//! let client = AIClient::new();
//! let response = client.chat("claude-3-5-sonnet-20241022", "What is Rust?").await?;
//! ```

/// Browser control and custom action definitions.
pub mod actions;
/// AI agent core: configuration, execution, and lifecycle.
pub mod agent;
/// AI-specific error types and result aliases.
pub mod error;
/// Chat message types and roles.
pub mod message;
/// AG-UI and A2UI protocol definitions.
pub mod protocol;
/// Multi-provider AI client (OpenAI, Anthropic, Gemini, etc.).
pub mod providers;
/// Chat session persistence and management.
pub mod session;
/// AI-generated UI component types.
pub mod ui;

/// Core agent and configuration types.
pub use agent::{AIAgent, AIConfig};
/// Error and result types for AI operations.
pub use error::{AIError, AIResult};

/// Chat message content and role types.
pub use message::{Message, MessageContent, MessageRole};

/// AI client, provider types, streaming events, and tool definitions.
pub use providers::{
    AIClient, ChatOptions, CompletionResponse, ModelInfo, ProviderType, StreamEvent, ToolCall,
    ToolDef, UsageStats,
};

/// Session persistence and management types.
pub use session::{ChatSession, SessionManager};

/// Browser control action types and registry.
pub use actions::{Action, ActionContext, ActionRegistry, ActionResult};

/// AG-UI protocol types for AI-UI streaming interaction.
pub use protocol::agui::{AGUIEmitter, AGUIEvent, AGUIMessage, AGUITool, AGUIToolCall};

/// A2UI protocol types for dynamic UI generation.
pub use protocol::a2ui::{NotifyLevel, UIAction, UIComponentSpec, UIComponentType};
