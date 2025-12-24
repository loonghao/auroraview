//! Middleware system for event processing
//!
//! Middleware allows intercepting, filtering, and transforming events
//! before they reach handlers. This is useful for:
//! - Logging all events
//! - Filtering out unwanted events
//! - Transforming event data
//! - Rate limiting
//! - Authentication/authorization

use parking_lot::RwLock;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Result of middleware processing
#[derive(Debug, Clone, PartialEq)]
pub enum MiddlewareResult {
    /// Continue processing the event
    Continue,
    /// Stop processing, don't deliver to handlers
    Stop,
    /// Stop with a reason message
    StopWithReason(String),
}

impl MiddlewareResult {
    /// Check if processing should continue
    pub fn should_continue(&self) -> bool {
        matches!(self, MiddlewareResult::Continue)
    }
}

/// Log level for logging middleware
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

/// Middleware trait for event processing
///
/// Middleware can intercept events before and after they are delivered to handlers.
/// This enables logging, filtering, transformation, and other cross-cutting concerns.
pub trait Middleware: Send + Sync {
    /// Called before an event is emitted to handlers
    ///
    /// Return `MiddlewareResult::Continue` to allow the event to proceed,
    /// or `MiddlewareResult::Stop` to prevent delivery.
    ///
    /// The `data` parameter is mutable, allowing transformation.
    fn before_emit(&self, event: &str, data: &mut Value) -> MiddlewareResult;

    /// Called after an event has been delivered to handlers
    ///
    /// This is called regardless of whether any handlers were connected.
    /// The `handler_count` indicates how many handlers received the event.
    fn after_emit(&self, event: &str, data: &Value, handler_count: usize) {
        let _ = (event, data, handler_count); // Default: no-op
    }

    /// Get the middleware name for debugging
    fn name(&self) -> &str {
        "unnamed"
    }

    /// Get the middleware priority (lower = earlier)
    ///
    /// Middleware with lower priority values run first.
    /// Default is 100.
    fn priority(&self) -> i32 {
        100
    }
}

// ============================================================================
// LoggingMiddleware - Log all events
// ============================================================================

/// Middleware that logs all events
///
/// # Example
///
/// ```rust
/// use aurora_signals::prelude::*;
///
/// let logging = LoggingMiddleware::new(LogLevel::Debug);
/// ```
pub struct LoggingMiddleware {
    level: LogLevel,
    /// Optional prefix for log messages
    prefix: Option<String>,
    /// Whether to log event data
    log_data: bool,
}

impl LoggingMiddleware {
    /// Create a new logging middleware with the specified level
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            prefix: None,
            log_data: true,
        }
    }

    /// Set a prefix for log messages
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Disable logging of event data (for privacy)
    pub fn without_data(mut self) -> Self {
        self.log_data = false;
        self
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new(LogLevel::Debug)
    }
}

impl Middleware for LoggingMiddleware {
    fn before_emit(&self, event: &str, data: &mut Value) -> MiddlewareResult {
        let prefix = self.prefix.as_deref().unwrap_or("[Signal]");

        match self.level {
            LogLevel::Trace => {
                if self.log_data {
                    tracing::trace!("{} Emitting: {} with {:?}", prefix, event, data);
                } else {
                    tracing::trace!("{} Emitting: {}", prefix, event);
                }
            }
            LogLevel::Debug => {
                if self.log_data {
                    tracing::debug!("{} Emitting: {} with {:?}", prefix, event, data);
                } else {
                    tracing::debug!("{} Emitting: {}", prefix, event);
                }
            }
            LogLevel::Info => {
                if self.log_data {
                    tracing::info!("{} Emitting: {} with {:?}", prefix, event, data);
                } else {
                    tracing::info!("{} Emitting: {}", prefix, event);
                }
            }
            LogLevel::Warn => {
                if self.log_data {
                    tracing::warn!("{} Emitting: {} with {:?}", prefix, event, data);
                } else {
                    tracing::warn!("{} Emitting: {}", prefix, event);
                }
            }
            LogLevel::Error => {
                if self.log_data {
                    tracing::error!("{} Emitting: {} with {:?}", prefix, event, data);
                } else {
                    tracing::error!("{} Emitting: {}", prefix, event);
                }
            }
        }

        MiddlewareResult::Continue
    }

    fn after_emit(&self, event: &str, _data: &Value, handler_count: usize) {
        let prefix = self.prefix.as_deref().unwrap_or("[Signal]");
        tracing::trace!(
            "{} {} delivered to {} handlers",
            prefix,
            event,
            handler_count
        );
    }

    fn name(&self) -> &str {
        "logging"
    }

    fn priority(&self) -> i32 {
        0 // Run first
    }
}

// ============================================================================
// FilterMiddleware - Filter events by pattern
// ============================================================================

/// Middleware that filters events by pattern
///
/// Supports both allow and deny patterns using regex.
///
/// # Example
///
/// ```rust
/// use aurora_signals::prelude::*;
///
/// // Deny all internal events
/// let filter = FilterMiddleware::new()
///     .deny_pattern("internal:.*")
///     .unwrap();
///
/// // Only allow specific events
/// let filter = FilterMiddleware::new()
///     .allow_pattern("user:.*")
///     .unwrap()
///     .allow_pattern("app:.*")
///     .unwrap();
/// ```
pub struct FilterMiddleware {
    allow_patterns: RwLock<Vec<Regex>>,
    deny_patterns: RwLock<Vec<Regex>>,
    /// If true, events not matching any allow pattern are denied
    /// If false, events not matching any deny pattern are allowed
    default_allow: bool,
}

impl FilterMiddleware {
    /// Create a new filter middleware
    ///
    /// By default, all events are allowed unless explicitly denied.
    pub fn new() -> Self {
        Self {
            allow_patterns: RwLock::new(Vec::new()),
            deny_patterns: RwLock::new(Vec::new()),
            default_allow: true,
        }
    }

    /// Create a filter that denies all events by default
    ///
    /// Events must match an allow pattern to be processed.
    pub fn deny_by_default() -> Self {
        Self {
            allow_patterns: RwLock::new(Vec::new()),
            deny_patterns: RwLock::new(Vec::new()),
            default_allow: false,
        }
    }

    /// Add an allow pattern
    pub fn allow_pattern(self, pattern: &str) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;
        self.allow_patterns.write().push(regex);
        Ok(self)
    }

    /// Add a deny pattern
    pub fn deny_pattern(self, pattern: &str) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;
        self.deny_patterns.write().push(regex);
        Ok(self)
    }

    /// Add an allow pattern at runtime
    pub fn add_allow_pattern(&self, pattern: &str) -> Result<(), regex::Error> {
        let regex = Regex::new(pattern)?;
        self.allow_patterns.write().push(regex);
        Ok(())
    }

    /// Add a deny pattern at runtime
    pub fn add_deny_pattern(&self, pattern: &str) -> Result<(), regex::Error> {
        let regex = Regex::new(pattern)?;
        self.deny_patterns.write().push(regex);
        Ok(())
    }

    /// Clear all patterns
    pub fn clear(&self) {
        self.allow_patterns.write().clear();
        self.deny_patterns.write().clear();
    }

    fn is_allowed(&self, event: &str) -> bool {
        // Check deny patterns first
        for pattern in self.deny_patterns.read().iter() {
            if pattern.is_match(event) {
                return false;
            }
        }

        // If no allow patterns, use default
        let allow_patterns = self.allow_patterns.read();
        if allow_patterns.is_empty() {
            return self.default_allow;
        }

        // Check allow patterns
        for pattern in allow_patterns.iter() {
            if pattern.is_match(event) {
                return true;
            }
        }

        // No match, use default
        self.default_allow
    }
}

impl Default for FilterMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for FilterMiddleware {
    fn before_emit(&self, event: &str, _data: &mut Value) -> MiddlewareResult {
        if self.is_allowed(event) {
            MiddlewareResult::Continue
        } else {
            tracing::trace!("Event filtered: {}", event);
            MiddlewareResult::StopWithReason(format!("Event '{}' filtered by pattern", event))
        }
    }

    fn name(&self) -> &str {
        "filter"
    }

    fn priority(&self) -> i32 {
        10 // Run early, after logging
    }
}

// ============================================================================
// TransformMiddleware - Transform event data
// ============================================================================

/// Type alias for transform functions
pub type TransformFn = Arc<dyn Fn(&mut Value) + Send + Sync>;

/// Middleware that transforms event data
///
/// # Example
///
/// ```rust
/// use aurora_signals::prelude::*;
/// use serde_json::json;
///
/// let transform = TransformMiddleware::new()
///     .add_transform("user:*", |data| {
///         // Add a field to all user events
///         if let Some(obj) = data.as_object_mut() {
///             obj.insert("processed".to_string(), json!(true));
///         }
///     });
/// ```
pub struct TransformMiddleware {
    /// Exact event name transforms
    exact_transforms: RwLock<HashMap<String, TransformFn>>,
    /// Pattern-based transforms (pattern, transform)
    pattern_transforms: RwLock<Vec<(Regex, TransformFn)>>,
    /// Global transform applied to all events
    global_transform: RwLock<Option<TransformFn>>,
}

impl TransformMiddleware {
    /// Create a new transform middleware
    pub fn new() -> Self {
        Self {
            exact_transforms: RwLock::new(HashMap::new()),
            pattern_transforms: RwLock::new(Vec::new()),
            global_transform: RwLock::new(None),
        }
    }

    /// Add a transform for a specific event name
    pub fn add_exact_transform<F>(self, event: &str, transform: F) -> Self
    where
        F: Fn(&mut Value) + Send + Sync + 'static,
    {
        self.exact_transforms
            .write()
            .insert(event.to_string(), Arc::new(transform));
        self
    }

    /// Add a transform for events matching a pattern
    pub fn add_transform<F>(self, pattern: &str, transform: F) -> Result<Self, regex::Error>
    where
        F: Fn(&mut Value) + Send + Sync + 'static,
    {
        let regex = Regex::new(pattern)?;
        self.pattern_transforms
            .write()
            .push((regex, Arc::new(transform)));
        Ok(self)
    }

    /// Set a global transform applied to all events
    pub fn set_global_transform<F>(self, transform: F) -> Self
    where
        F: Fn(&mut Value) + Send + Sync + 'static,
    {
        *self.global_transform.write() = Some(Arc::new(transform));
        self
    }

    /// Add a transform at runtime
    pub fn add_runtime_transform<F>(&self, pattern: &str, transform: F) -> Result<(), regex::Error>
    where
        F: Fn(&mut Value) + Send + Sync + 'static,
    {
        let regex = Regex::new(pattern)?;
        self.pattern_transforms
            .write()
            .push((regex, Arc::new(transform)));
        Ok(())
    }
}

impl Default for TransformMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for TransformMiddleware {
    fn before_emit(&self, event: &str, data: &mut Value) -> MiddlewareResult {
        // Apply global transform
        if let Some(transform) = self.global_transform.read().as_ref() {
            transform(data);
        }

        // Apply exact match transform
        if let Some(transform) = self.exact_transforms.read().get(event) {
            transform(data);
        }

        // Apply pattern transforms
        for (pattern, transform) in self.pattern_transforms.read().iter() {
            if pattern.is_match(event) {
                transform(data);
            }
        }

        MiddlewareResult::Continue
    }

    fn name(&self) -> &str {
        "transform"
    }

    fn priority(&self) -> i32 {
        50 // Run in the middle
    }
}

// ============================================================================
// MiddlewareChain - Chain of middleware
// ============================================================================

/// A chain of middleware that processes events in order
pub struct MiddlewareChain {
    middlewares: RwLock<Vec<Arc<dyn Middleware>>>,
}

impl MiddlewareChain {
    /// Create a new empty middleware chain
    pub fn new() -> Self {
        Self {
            middlewares: RwLock::new(Vec::new()),
        }
    }

    /// Add a middleware to the chain
    pub fn add<M: Middleware + 'static>(&self, middleware: M) {
        let mut middlewares = self.middlewares.write();
        middlewares.push(Arc::new(middleware));
        // Sort by priority
        middlewares.sort_by_key(|m| m.priority());
    }

    /// Add an Arc-wrapped middleware
    pub fn add_arc(&self, middleware: Arc<dyn Middleware>) {
        let mut middlewares = self.middlewares.write();
        middlewares.push(middleware);
        middlewares.sort_by_key(|m| m.priority());
    }

    /// Remove all middleware
    pub fn clear(&self) {
        self.middlewares.write().clear();
    }

    /// Get the number of middleware in the chain
    pub fn len(&self) -> usize {
        self.middlewares.read().len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.middlewares.read().is_empty()
    }

    /// Process an event through all middleware (before emit)
    pub fn process_before(&self, event: &str, data: &mut Value) -> MiddlewareResult {
        for middleware in self.middlewares.read().iter() {
            let result = middleware.before_emit(event, data);
            if !result.should_continue() {
                return result;
            }
        }
        MiddlewareResult::Continue
    }

    /// Notify all middleware after emit
    pub fn process_after(&self, event: &str, data: &Value, handler_count: usize) {
        for middleware in self.middlewares.read().iter() {
            middleware.after_emit(event, data, handler_count);
        }
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_logging_middleware() {
        let middleware = LoggingMiddleware::new(LogLevel::Debug);
        let mut data = json!({"key": "value"});

        let result = middleware.before_emit("test:event", &mut data);
        assert_eq!(result, MiddlewareResult::Continue);
    }

    #[test]
    fn test_filter_middleware_deny() {
        let middleware = FilterMiddleware::new().deny_pattern("internal:.*").unwrap();

        let mut data = json!(null);

        // Should be denied
        let result = middleware.before_emit("internal:secret", &mut data);
        assert!(!result.should_continue());

        // Should be allowed
        let result = middleware.before_emit("public:event", &mut data);
        assert!(result.should_continue());
    }

    #[test]
    fn test_filter_middleware_allow() {
        let middleware = FilterMiddleware::deny_by_default()
            .allow_pattern("user:.*")
            .unwrap()
            .allow_pattern("app:.*")
            .unwrap();

        let mut data = json!(null);

        // Should be allowed
        let result = middleware.before_emit("user:login", &mut data);
        assert!(result.should_continue());

        let result = middleware.before_emit("app:start", &mut data);
        assert!(result.should_continue());

        // Should be denied
        let result = middleware.before_emit("internal:secret", &mut data);
        assert!(!result.should_continue());
    }

    #[test]
    fn test_transform_middleware() {
        let middleware = TransformMiddleware::new().add_exact_transform("test:event", |data| {
            if let Some(obj) = data.as_object_mut() {
                obj.insert("added".to_string(), json!(true));
            }
        });

        let mut data = json!({"original": "value"});

        middleware.before_emit("test:event", &mut data);

        assert_eq!(data["original"], "value");
        assert_eq!(data["added"], true);
    }

    #[test]
    fn test_transform_middleware_pattern() {
        let middleware = TransformMiddleware::new()
            .add_transform("user:.*", |data| {
                if let Some(obj) = data.as_object_mut() {
                    obj.insert("user_event".to_string(), json!(true));
                }
            })
            .unwrap();

        let mut data = json!({});

        middleware.before_emit("user:login", &mut data);
        assert_eq!(data["user_event"], true);

        let mut data2 = json!({});
        middleware.before_emit("app:start", &mut data2);
        assert!(data2.get("user_event").is_none());
    }

    #[test]
    fn test_middleware_chain() {
        let chain = MiddlewareChain::new();

        // Add filter that denies internal events
        chain.add(FilterMiddleware::new().deny_pattern("internal:.*").unwrap());

        // Add transform
        chain.add(
            TransformMiddleware::new().add_exact_transform("test:event", |data| {
                if let Some(obj) = data.as_object_mut() {
                    obj.insert("transformed".to_string(), json!(true));
                }
            }),
        );

        // Test allowed event
        let mut data = json!({});
        let result = chain.process_before("test:event", &mut data);
        assert!(result.should_continue());
        assert_eq!(data["transformed"], true);

        // Test denied event
        let mut data2 = json!({});
        let result = chain.process_before("internal:secret", &mut data2);
        assert!(!result.should_continue());
    }

    #[test]
    fn test_middleware_priority() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct PriorityMiddleware {
            priority: i32,
            order: Arc<AtomicUsize>,
            my_order: Arc<AtomicUsize>,
        }

        impl Middleware for PriorityMiddleware {
            fn before_emit(&self, _event: &str, _data: &mut Value) -> MiddlewareResult {
                let current = self.order.fetch_add(1, Ordering::SeqCst);
                self.my_order.store(current, Ordering::SeqCst);
                MiddlewareResult::Continue
            }

            fn priority(&self) -> i32 {
                self.priority
            }
        }

        let chain = MiddlewareChain::new();
        let order = Arc::new(AtomicUsize::new(0));

        let order1 = Arc::new(AtomicUsize::new(999));
        let order2 = Arc::new(AtomicUsize::new(999));
        let order3 = Arc::new(AtomicUsize::new(999));

        // Add in reverse priority order
        chain.add(PriorityMiddleware {
            priority: 100,
            order: order.clone(),
            my_order: order3.clone(),
        });
        chain.add(PriorityMiddleware {
            priority: 0,
            order: order.clone(),
            my_order: order1.clone(),
        });
        chain.add(PriorityMiddleware {
            priority: 50,
            order: order.clone(),
            my_order: order2.clone(),
        });

        let mut data = json!(null);
        chain.process_before("test", &mut data);

        // Should be sorted by priority
        assert_eq!(order1.load(Ordering::SeqCst), 0); // priority 0, runs first
        assert_eq!(order2.load(Ordering::SeqCst), 1); // priority 50, runs second
        assert_eq!(order3.load(Ordering::SeqCst), 2); // priority 100, runs third
    }
}
