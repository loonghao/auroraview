//! Action registry for managing available actions

use crate::actions::Action;
use crate::message::Tool;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing AI agent actions
pub struct ActionRegistry {
    actions: HashMap<String, Arc<dyn Action>>,
}

impl ActionRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Create registry with default browser actions
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(super::NavigateAction);
        registry.register(super::SearchAction);
        registry.register(super::ClickAction);
        registry.register(super::TypeAction);
        registry.register(super::ScreenshotAction);
        registry.register(super::ScrollAction);
        registry
    }

    /// Register an action
    pub fn register<A: Action + 'static>(&mut self, action: A) {
        let name = action.name().to_string();
        self.actions.insert(name, Arc::new(action));
    }

    /// Get action by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Action>> {
        self.actions.get(name).cloned()
    }

    /// Check if action exists
    pub fn contains(&self, name: &str) -> bool {
        self.actions.contains_key(name)
    }

    /// Remove an action
    pub fn remove(&mut self, name: &str) -> Option<Arc<dyn Action>> {
        self.actions.remove(name)
    }

    /// Get all action names
    pub fn names(&self) -> Vec<&str> {
        self.actions.keys().map(|s| s.as_str()).collect()
    }

    /// Get tools schema for AI model
    pub fn get_tools(&self) -> Vec<Tool> {
        self.actions
            .values()
            .map(|action| Tool {
                name: action.name().to_string(),
                description: action.description().to_string(),
                parameters: action.parameters(),
            })
            .collect()
    }

    /// Get number of registered actions
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_with_defaults() {
        let registry = ActionRegistry::with_defaults();
        assert!(registry.contains("navigate"));
        assert!(registry.contains("search"));
        assert!(registry.contains("click"));
    }

    #[test]
    fn test_get_tools() {
        let registry = ActionRegistry::with_defaults();
        let tools = registry.get_tools();

        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name == "navigate"));
    }
}
