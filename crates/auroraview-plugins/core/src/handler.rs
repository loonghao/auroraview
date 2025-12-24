//! Plugin handler trait

use crate::{PluginResult, ScopeConfig};
use serde_json::Value;

/// Trait for plugin implementations
pub trait PluginHandler: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;

    /// Handle a command
    fn handle(&self, command: &str, args: Value, scope: &ScopeConfig) -> PluginResult<Value>;

    /// Get supported commands
    fn commands(&self) -> Vec<&str>;
}
