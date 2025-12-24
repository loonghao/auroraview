//! Plugin type definitions

use serde::{Deserialize, Serialize};

/// Plugin command specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Required arguments
    pub required_args: Vec<String>,
    /// Optional arguments
    pub optional_args: Vec<String>,
}

impl PluginCommand {
    /// Create a new command specification
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required_args: Vec::new(),
            optional_args: Vec::new(),
        }
    }

    /// Add required arguments
    pub fn with_required(mut self, args: &[&str]) -> Self {
        self.required_args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add optional arguments
    pub fn with_optional(mut self, args: &[&str]) -> Self {
        self.optional_args = args.iter().map(|s| s.to_string()).collect();
        self
    }
}
