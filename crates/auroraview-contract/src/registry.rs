//! Priority registry shared by the host-adapter and render-backend contracts.
//!
//! This is the Rust mirror of the Python dispatcher registry
//! (`auroraview.utils.thread_dispatcher.registry`). Same mechanics, so there is
//! one extension idiom to learn, not two:
//!
//! * candidates carry an `i32` priority, highest first
//! * candidates are constructed lazily, only when they are probed or selected
//! * an environment override can pin one candidate by name
//! * a failed override degrades to priority order **and reports a warning** --
//!   it does not abort selection

use std::fmt;

/// Type alias for a lazily constructed candidate.
pub type Factory<T> = Box<dyn Fn() -> T + Send + Sync>;

/// A single registry candidate.
pub struct Entry<T> {
    priority: i32,
    name: String,
    factory: Factory<T>,
}

impl<T> Entry<T> {
    /// Create an entry.
    pub fn new(
        name: impl Into<String>,
        priority: i32,
        factory: impl Fn() -> T + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            priority,
            factory: Box::new(factory),
        }
    }

    /// Priority of this candidate (higher is tried first).
    pub fn priority(&self) -> i32 {
        self.priority
    }

    /// Registered name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Construct an instance.
    pub fn create(&self) -> T {
        (self.factory)()
    }
}

impl<T> fmt::Debug for Entry<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("name", &self.name)
            .field("priority", &self.priority)
            .finish_non_exhaustive()
    }
}

/// The outcome of a registry lookup.
#[derive(Debug, Clone)]
pub struct Selection<T> {
    /// The selected candidate.
    pub value: T,
    /// Registered name of the selected candidate.
    pub name: String,
    /// Whether the selection came from an environment override.
    pub via_env_override: bool,
    /// Non-fatal problems encountered while selecting (e.g. an override that
    /// could not be honoured).
    pub warnings: Vec<String>,
}

impl<T> Selection<T> {
    /// Whether any warning was recorded.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// A priority-ordered registry of lazily constructed candidates.
///
/// Candidates are kept sorted by descending priority. `sort_by` is stable, so
/// candidates registered with the same priority are tried in registration
/// order.
pub struct Registry<T> {
    entries: Vec<Entry<T>>,
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T> fmt::Debug for Registry<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registry")
            .field("entries", &self.entries)
            .finish()
    }
}

impl<T> Registry<T> {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a candidate.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        priority: i32,
        factory: impl Fn() -> T + Send + Sync + 'static,
    ) {
        self.register_entry(Entry::new(name, priority, factory));
    }

    /// Register a pre-built [`Entry`].
    pub fn register_entry(&mut self, entry: Entry<T>) {
        self.entries.push(entry);
        self.entries
            .sort_by_key(|entry| std::cmp::Reverse(entry.priority));
    }

    /// Register a [`Default`]-constructible candidate.
    pub fn register_default<C>(&mut self, name: impl Into<String>, priority: i32)
    where
        C: Default + Into<T> + 'static,
    {
        self.register(name, priority, || C::default().into());
    }

    /// Remove a candidate by name. Returns whether it was present.
    pub fn unregister(&mut self, name: &str) -> bool {
        match self.entries.iter().position(|entry| entry.name == name) {
            Some(position) => {
                self.entries.remove(position);
                true
            }
            None => false,
        }
    }

    /// Remove every candidate.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of registered candidates.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Candidates in priority order (highest first).
    pub fn entries(&self) -> &[Entry<T>] {
        &self.entries
    }

    /// Registered names in priority order.
    pub fn names(&self) -> Vec<&str> {
        self.entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect()
    }

    /// Construct a candidate by name, bypassing priority and availability.
    pub fn get(&self, name: &str) -> Option<T> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.create())
    }

    /// Run `predicate` against one candidate.
    ///
    /// A panicking predicate is converted into "not usable" plus a warning, so a
    /// single broken candidate cannot abort discovery for everyone. This is the
    /// Rust counterpart of the same guarantee in the Python registry.
    ///
    /// Only unwinding panics are caught; a build with `panic = "abort"` cannot
    /// be rescued, which is why the contract still requires `predicate`
    /// implementations to stay panic-free.
    fn check(
        predicate: &mut impl FnMut(&T) -> bool,
        value: &T,
        display: &str,
        warnings: &mut Vec<String>,
    ) -> bool {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| predicate(value))) {
            Ok(result) => result,
            Err(_) => {
                warnings.push(format!(
                    "'{}' panicked during selection; treating it as unavailable",
                    display
                ));
                false
            }
        }
    }

    /// First candidate (priority order) satisfying `predicate`.
    pub fn first(&self, predicate: impl FnMut(&T) -> bool) -> Option<Selection<T>> {
        self.select(None, predicate)
    }

    /// Select a candidate.
    ///
    /// When `env_override` names a registered candidate it is tried first. If it
    /// is missing or fails `predicate`, selection falls through to priority
    /// order and a warning is recorded on the returned [`Selection`] -- an
    /// unusable override never turns into a hard error.
    ///
    /// A candidate that *panics* while being probed is treated as unusable and
    /// recorded as a warning, not propagated: discovery is shared
    /// infrastructure, so one broken third-party adapter must not take down host
    /// detection for everyone. This mirrors the Python registry's guarantee.
    ///
    /// Unwinding panics are caught with `catch_unwind`. This cannot rescue a
    /// build compiled with `panic = "abort"`, nor non-unwinding aborts; the
    /// contract requirement that `predicate` implementations (notably
    /// [`crate::host::HostAdapter::detect`] and
    /// [`crate::backend::RenderBackend::available`]) stay panic-free still
    /// holds.
    pub fn select(
        &self,
        env_override: Option<&str>,
        mut predicate: impl FnMut(&T) -> bool,
    ) -> Option<Selection<T>> {
        let mut warnings = Vec::new();

        if let Some(requested) = env_override {
            let requested = requested.trim();
            if !requested.is_empty() {
                match self
                    .entries
                    .iter()
                    .find(|entry| entry.name.eq_ignore_ascii_case(requested))
                {
                    Some(entry) => {
                        let value = entry.create();
                        if Self::check(&mut predicate, &value, &entry.name, &mut warnings) {
                            return Some(Selection {
                                value,
                                name: entry.name.clone(),
                                via_env_override: true,
                                warnings,
                            });
                        }
                        warnings.push(format!(
                            "'{}' was requested by override but is not available; \
                             falling back to priority order",
                            entry.name
                        ));
                    }
                    None => warnings.push(format!(
                        "'{}' was requested by override but is not registered",
                        requested
                    )),
                }
            }
        }

        for entry in &self.entries {
            let value = entry.create();
            if Self::check(&mut predicate, &value, &entry.name, &mut warnings) {
                return Some(Selection {
                    value,
                    name: entry.name.clone(),
                    via_env_override: false,
                    warnings,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> Registry<String> {
        let mut registry = Registry::new();
        registry.register("maya", 200, || "maya".to_string());
        registry.register("qt", 100, || "qt".to_string());
        registry.register("fallback", 0, || "fallback".to_string());
        registry
    }

    #[test]
    fn entries_are_kept_in_descending_priority_order() {
        let registry = registry();
        assert_eq!(registry.names(), vec!["maya", "qt", "fallback"]);
        assert_eq!(registry.len(), 3);
        assert!(!registry.is_empty());
    }

    #[test]
    fn equal_priority_keeps_registration_order() {
        let mut registry = Registry::new();
        registry.register("first", 100, || "first".to_string());
        registry.register("second", 100, || "second".to_string());
        assert_eq!(registry.names(), vec!["first", "second"]);
    }

    #[test]
    fn first_uses_priority_order() {
        let registry = registry();
        let selection = registry.first(|value| value == "qt" || value == "fallback");
        let selection = selection.expect("a candidate must match");
        assert_eq!(selection.name, "qt");
        assert!(!selection.via_env_override);
        assert!(!selection.has_warnings());
    }

    #[test]
    fn candidates_are_constructed_lazily() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let calls = Arc::new(AtomicUsize::new(0));
        let mut registry = Registry::new();
        let counter = Arc::clone(&calls);
        registry.register("counted", 10, move || {
            counter.fetch_add(1, Ordering::SeqCst);
            "counted".to_string()
        });
        registry.register("cheap", 20, || "cheap".to_string());

        assert_eq!(
            calls.load(Ordering::SeqCst),
            0,
            "registration must not build"
        );

        let selection = registry.first(|value| value == "cheap");
        assert_eq!(selection.expect("cheap").name, "cheap");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            0,
            "a lower-priority match must not be constructed"
        );
    }

    #[test]
    fn env_override_wins_over_priority() {
        let registry = registry();
        let selection = registry
            .select(Some("fallback"), |_| true)
            .expect("override must select");
        assert_eq!(selection.name, "fallback");
        assert!(selection.via_env_override);
    }

    #[test]
    fn env_override_is_case_insensitive() {
        let registry = registry();
        let selection = registry
            .select(Some("  MAYA  "), |_| true)
            .expect("override must select");
        assert_eq!(selection.name, "maya");
    }

    #[test]
    fn unusable_override_falls_back_and_warns() {
        let registry = registry();
        let selection = registry
            .select(Some("maya"), |value| value != "maya")
            .expect("must fall back to a usable candidate");
        assert_eq!(selection.name, "qt");
        assert!(!selection.via_env_override);
        assert!(selection.has_warnings());
        assert!(selection.warnings[0].contains("not available"));
    }

    #[test]
    fn unknown_override_name_warns_and_falls_back() {
        let registry = registry();
        let selection = registry
            .select(Some("nonexistent"), |_| true)
            .expect("must fall back");
        assert_eq!(selection.name, "maya");
        assert!(selection.warnings[0].contains("not registered"));
    }

    #[test]
    fn blank_override_is_ignored() {
        let selection = registry()
            .select(Some("   "), |_| true)
            .expect("must select");
        assert_eq!(selection.name, "maya");
        assert!(!selection.via_env_override);
        assert!(!selection.has_warnings());
    }

    #[test]
    fn no_match_returns_none() {
        assert!(registry().first(|_| false).is_none());
    }

    #[test]
    fn a_panicking_predicate_is_skipped_not_propagated() {
        // Mirrors the Python guarantee in `adapter/registry.py`: discovery is
        // shared infrastructure, so one broken third-party candidate must not
        // take down host detection for everyone.
        let mut registry = Registry::new();
        registry.register("panicky", 100, || "panicky".to_string());
        registry.register("good", 50, || "good".to_string());

        let selection = registry
            .first(|value| {
                if value == "panicky" {
                    panic!("this candidate is broken on purpose");
                }
                true
            })
            .expect("must fall through to the healthy candidate");

        assert_eq!(selection.name, "good");
        assert!(selection.has_warnings());
        assert!(selection.warnings[0].contains("panicked"));
    }

    #[test]
    fn a_panicking_predicate_under_an_override_also_falls_back() {
        let mut registry = Registry::new();
        registry.register("panicky", 100, || "panicky".to_string());
        registry.register("good", 50, || "good".to_string());

        let selection = registry
            .select(Some("panicky"), |value| {
                if value == "panicky" {
                    panic!("this candidate is broken on purpose");
                }
                true
            })
            .expect("must fall through to the healthy candidate");

        assert_eq!(selection.name, "good");
        assert!(!selection.via_env_override);
        assert!(selection.warnings.iter().any(|w| w.contains("panicked")));
    }

    #[test]
    fn unregister_clear_and_get() {
        let mut registry = registry();
        assert!(registry.unregister("qt"));
        assert!(!registry.unregister("qt"));
        assert_eq!(registry.names(), vec!["maya", "fallback"]);

        assert_eq!(registry.get("fallback"), Some("fallback".to_string()));
        assert_eq!(registry.get("qt"), None);

        registry.clear();
        assert!(registry.is_empty());
        assert!(registry.first(|_| true).is_none());
    }
}
