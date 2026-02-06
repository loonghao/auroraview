//! Chrome Omnibox API Implementation
//!
//! Provides address bar (omnibox) integration for extensions.
//!
//! ## Features
//! - Register keyword for omnibox
//! - Provide suggestions as user types
//! - Handle user selection

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// Suggestion result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestResult {
    /// Text to display in the omnibox
    pub content: String,
    /// Description text (can include XML markup for styling)
    pub description: String,
    /// Whether this is deletable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletable: Option<bool>,
}

/// Default suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultSuggestion {
    /// Description text
    pub description: String,
}

/// Input disposition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum OnInputEnteredDisposition {
    #[default]
    CurrentTab,
    NewForegroundTab,
    NewBackgroundTab,
}

/// Omnibox state for an extension
#[derive(Debug, Clone, Default)]
struct OmniboxState {
    /// Default suggestion
    default_suggestion: Option<DefaultSuggestion>,
    /// Current suggestions
    suggestions: Vec<SuggestResult>,
}

/// Omnibox API handler
pub struct OmniboxApi {
    /// Per-extension omnibox state
    states: Arc<RwLock<HashMap<String, OmniboxState>>>,
}

impl Default for OmniboxApi {
    fn default() -> Self {
        Self::new()
    }
}

impl OmniboxApi {
    /// Create a new OmniboxApi instance
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set default suggestion
    pub fn set_default_suggestion(
        &self,
        extension_id: &str,
        suggestion: DefaultSuggestion,
    ) -> ExtensionResult<Value> {
        let mut states = self.states.write().unwrap();
        let state = states.entry(extension_id.to_string()).or_default();
        state.default_suggestion = Some(suggestion);
        Ok(json!(null))
    }

    /// Send suggestions (called from onInputChanged handler)
    pub fn send_suggestions(
        &self,
        extension_id: &str,
        suggestions: Vec<SuggestResult>,
    ) -> ExtensionResult<Value> {
        let mut states = self.states.write().unwrap();
        let state = states.entry(extension_id.to_string()).or_default();
        state.suggestions = suggestions;
        Ok(json!(null))
    }

    /// Get current suggestions for an extension
    pub fn get_suggestions(&self, extension_id: &str) -> Vec<SuggestResult> {
        let states = self.states.read().unwrap();
        states
            .get(extension_id)
            .map(|s| s.suggestions.clone())
            .unwrap_or_default()
    }

    /// Get default suggestion for an extension
    pub fn get_default_suggestion(&self, extension_id: &str) -> Option<DefaultSuggestion> {
        let states = self.states.read().unwrap();
        states
            .get(extension_id)
            .and_then(|s| s.default_suggestion.clone())
    }

    /// Handle API call
    pub fn handle(
        &self,
        method: &str,
        params: Value,
        extension_id: &str,
    ) -> ExtensionResult<Value> {
        match method {
            "setDefaultSuggestion" => {
                let suggestion: DefaultSuggestion = serde_json::from_value(params)?;
                self.set_default_suggestion(extension_id, suggestion)
            }
            "sendSuggestions" => {
                let suggestions: Vec<SuggestResult> = params
                    .get("suggestions")
                    .cloned()
                    .map(|v| serde_json::from_value(v).unwrap_or_default())
                    .unwrap_or_default();
                self.send_suggestions(extension_id, suggestions)
            }
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_default_suggestion() {
        let api = OmniboxApi::new();
        api.set_default_suggestion(
            "test-ext",
            DefaultSuggestion {
                description: "Search for %s".to_string(),
            },
        )
        .unwrap();

        let suggestion = api.get_default_suggestion("test-ext");
        assert!(suggestion.is_some());
        assert_eq!(suggestion.unwrap().description, "Search for %s");
    }

    #[test]
    fn test_send_suggestions() {
        let api = OmniboxApi::new();
        api.send_suggestions(
            "test-ext",
            vec![
                SuggestResult {
                    content: "result1".to_string(),
                    description: "Result 1".to_string(),
                    deletable: None,
                },
                SuggestResult {
                    content: "result2".to_string(),
                    description: "Result 2".to_string(),
                    deletable: Some(true),
                },
            ],
        )
        .unwrap();

        let suggestions = api.get_suggestions("test-ext");
        assert_eq!(suggestions.len(), 2);
    }
}
