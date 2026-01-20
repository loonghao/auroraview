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

pub mod actions;
pub mod agent;
pub mod error;
pub mod message;
pub mod protocol;
pub mod providers;
pub mod session;
pub mod ui;

// Re-exports - Core types
pub use agent::{AIAgent, AIConfig};
pub use error::{AIError, AIResult};

// Re-exports - Message types
pub use message::{Message, MessageContent, MessageRole};

// Re-exports - Provider types
pub use providers::{
    AIClient, ChatOptions, CompletionResponse, ModelInfo, ProviderType, StreamEvent, ToolCall,
    ToolDef, UsageStats,
};

// Re-exports - Session management
pub use session::{ChatSession, SessionManager};

// Re-exports - Actions
pub use actions::{Action, ActionContext, ActionRegistry, ActionResult};

// Re-exports - AG-UI Protocol
pub use protocol::agui::{AGUIEmitter, AGUIEvent, AGUIMessage, AGUITool, AGUIToolCall};

// Re-exports - A2UI Protocol
pub use protocol::a2ui::{NotifyLevel, UIAction, UIComponentSpec, UIComponentType};
