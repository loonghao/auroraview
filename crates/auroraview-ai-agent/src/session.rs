//! Chat session management

use crate::message::{Message, MessageRole, ToolCall};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A chat session containing conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    /// Unique session ID
    pub id: String,

    /// Session title (derived from first message or set manually)
    pub title: String,

    /// Messages in the conversation
    pub messages: Vec<Message>,

    /// System prompt for this session
    pub system_prompt: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub last_modified: DateTime<Utc>,

    /// Session metadata
    #[serde(default)]
    pub metadata: SessionMetadata,
}

/// Session metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Model used in this session
    pub model: Option<String>,

    /// Provider used
    pub provider: Option<String>,

    /// Total tokens used
    pub total_tokens: u64,

    /// Custom tags
    pub tags: Vec<String>,
}

impl ChatSession {
    /// Create a new empty session
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: "New Chat".to_string(),
            messages: Vec::new(),
            system_prompt: None,
            created_at: now,
            last_modified: now,
            metadata: SessionMetadata::default(),
        }
    }

    /// Create session with system prompt
    pub fn with_system_prompt(prompt: impl Into<String>) -> Self {
        let mut session = Self::new();
        session.system_prompt = Some(prompt.into());
        session
    }

    /// Add a message to the session
    pub fn add_message(&mut self, message: Message) {
        // Update title from first user message
        if self.title == "New Chat" && message.role == MessageRole::User {
            let text = message.content.as_text();
            self.title = if text.len() > 50 {
                format!("{}...", &text[..50])
            } else {
                text
            };
        }

        self.messages.push(message);
        self.last_modified = Utc::now();
    }

    /// Add a user message
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.add_message(Message::user(content.into()));
    }

    /// Add an assistant message
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.add_message(Message::assistant(content.into()));
    }

    /// Add an assistant message with tool calls
    pub fn add_assistant_with_tools(
        &mut self,
        content: impl Into<String>,
        tool_calls: Vec<ToolCall>,
    ) {
        let msg = Message::assistant(content.into()).with_tool_calls(tool_calls);
        self.add_message(msg);
    }

    /// Add a tool result message
    pub fn add_tool_result(&mut self, tool_call_id: impl Into<String>, result: impl Into<String>) {
        self.add_message(Message::tool_result(tool_call_id, result));
    }

    /// Get all messages including system prompt
    pub fn get_messages_for_api(&self) -> Vec<Message> {
        let mut messages = Vec::new();

        // Add system prompt if set
        if let Some(ref prompt) = self.system_prompt {
            messages.push(Message::system(prompt.clone()));
        }

        // Add conversation messages
        messages.extend(self.messages.clone());

        messages
    }

    /// Get the last message
    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    /// Get the last assistant message
    pub fn last_assistant_message(&self) -> Option<&Message> {
        self.messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::Assistant)
    }

    /// Clear all messages (keeps system prompt)
    pub fn clear(&mut self) {
        self.messages.clear();
        self.title = "New Chat".to_string();
        self.last_modified = Utc::now();
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Estimate token count (rough approximation)
    pub fn estimate_tokens(&self) -> usize {
        let mut chars = 0;

        if let Some(ref prompt) = self.system_prompt {
            chars += prompt.len();
        }

        for msg in &self.messages {
            chars += msg.content.as_text().len();
        }

        // Rough estimate: 4 chars per token
        chars / 4
    }

    /// Truncate old messages to fit token limit
    pub fn truncate_to_fit(&mut self, max_tokens: usize) {
        while self.estimate_tokens() > max_tokens && self.messages.len() > 1 {
            // Remove oldest messages (but keep at least the last pair)
            self.messages.remove(0);
        }
    }
}

impl Default for ChatSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Session manager for handling multiple sessions
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: Vec<ChatSession>,
    active_session_id: Option<String>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            active_session_id: None,
        }
    }

    /// Create a new session and make it active
    pub fn new_session(&mut self) -> &mut ChatSession {
        let session = ChatSession::new();
        let id = session.id.clone();
        self.sessions.push(session);
        self.active_session_id = Some(id.clone());
        self.get_session_mut(&id).unwrap()
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &str) -> Option<&ChatSession> {
        self.sessions.iter().find(|s| s.id == id)
    }

    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut ChatSession> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    /// Get the active session
    pub fn active_session(&self) -> Option<&ChatSession> {
        self.active_session_id
            .as_ref()
            .and_then(|id| self.get_session(id))
    }

    /// Get mutable active session
    pub fn active_session_mut(&mut self) -> Option<&mut ChatSession> {
        if let Some(id) = self.active_session_id.clone() {
            self.get_session_mut(&id)
        } else {
            None
        }
    }

    /// Set active session
    pub fn set_active(&mut self, id: &str) -> bool {
        if self.sessions.iter().any(|s| s.id == id) {
            self.active_session_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    /// Delete a session
    pub fn delete_session(&mut self, id: &str) -> bool {
        if let Some(pos) = self.sessions.iter().position(|s| s.id == id) {
            self.sessions.remove(pos);

            // Update active session if deleted
            if self.active_session_id.as_deref() == Some(id) {
                self.active_session_id = self.sessions.first().map(|s| s.id.clone());
            }
            true
        } else {
            false
        }
    }

    /// Get all sessions
    pub fn all_sessions(&self) -> &[ChatSession] {
        &self.sessions
    }

    /// Get all sessions sorted by last modified
    pub fn sessions_by_recent(&self) -> Vec<&ChatSession> {
        let mut sessions: Vec<_> = self.sessions.iter().collect();
        sessions.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = ChatSession::new();
        assert_eq!(session.title, "New Chat");
        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_session_with_messages() {
        let mut session = ChatSession::new();
        session.add_user_message("Hello");
        session.add_assistant_message("Hi there!");

        assert_eq!(session.message_count(), 2);
        assert_eq!(session.title, "Hello");
    }

    #[test]
    fn test_session_manager() {
        let mut manager = SessionManager::new();

        let session = manager.new_session();
        let id = session.id.clone();

        assert!(manager.active_session().is_some());
        assert_eq!(manager.active_session().unwrap().id, id);
    }
}
