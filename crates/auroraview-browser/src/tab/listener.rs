//! Tab event listener system
//!
//! Provides `on_event`/`off_event` subscription API for [`TabManager`], modelled
//! after the `IpcRouter` listener pattern used in `auroraview-dcc`.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use dashmap::DashMap;

use super::TabState;

/// Opaque ID returned by [`TabListenerMap::on`].
///
/// Pass to [`TabListenerMap::off`] to unsubscribe that specific callback.
pub type TabListenerId = u64;

/// Discriminant used to scope tab listeners by event category.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TabEventKind {
    /// A new tab was created.
    Created,
    /// A tab was closed.
    Closed,
    /// The active tab changed.
    Activated,
    /// Tab title or URL changed.
    Updated,
    /// Tab loading state changed.
    LoadingChanged,
    /// Tab navigation history changed.
    HistoryChanged,
    /// Tab favicon changed.
    FaviconChanged,
    /// Tab pinned/muted state changed.
    StateChanged,
    /// DevTools opened/closed.
    DevToolsToggled,
}

/// Callback signature for tab event listeners.
///
/// The handler receives the current [`TabState`] snapshot at the time of the event.
pub type TabEventHandler = Arc<dyn Fn(&TabState) + Send + Sync>;

struct ListenerEntry {
    id: TabListenerId,
    handler: TabEventHandler,
}

static NEXT_LISTENER_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> TabListenerId {
    NEXT_LISTENER_ID.fetch_add(1, Ordering::Relaxed)
}

/// Thread-safe map from [`TabEventKind`] to a list of [`TabEventHandler`]s.
///
/// Embedded in [`TabManager`] to provide subscription/unsubscription without
/// requiring a full event-loop abstraction.
#[derive(Default)]
pub struct TabListenerMap {
    inner: DashMap<TabEventKind, Vec<ListenerEntry>>,
}

impl TabListenerMap {
    /// Create an empty listener map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe `handler` to events of `kind`.
    ///
    /// Returns a [`TabListenerId`] that can be used with [`off`](Self::off).
    pub fn on<F>(&self, kind: TabEventKind, handler: F) -> TabListenerId
    where
        F: Fn(&TabState) + Send + Sync + 'static,
    {
        let id = next_id();
        self.inner
            .entry(kind)
            .or_default()
            .push(ListenerEntry { id, handler: Arc::new(handler) });
        id
    }

    /// Unsubscribe the listener identified by `id` from events of `kind`.
    ///
    /// Returns `true` if the listener was found and removed.
    pub fn off(&self, kind: &TabEventKind, id: TabListenerId) -> bool {
        if let Some(mut entry) = self.inner.get_mut(kind) {
            let before = entry.len();
            entry.retain(|e| e.id != id);
            return entry.len() < before;
        }
        false
    }

    /// Remove all listeners for `kind`.
    ///
    /// Returns the number of listeners that were removed.
    pub fn off_all(&self, kind: &TabEventKind) -> usize {
        if let Some(mut entry) = self.inner.get_mut(kind) {
            let count = entry.len();
            entry.clear();
            count
        } else {
            0
        }
    }

    /// Return how many listeners are registered for `kind`.
    pub fn listener_count(&self, kind: &TabEventKind) -> usize {
        self.inner.get(kind).map(|e| e.len()).unwrap_or(0)
    }

    /// Dispatch `state` to all handlers registered for `kind`.
    pub fn emit(&self, kind: &TabEventKind, state: &TabState) {
        if let Some(handlers) = self.inner.get(kind) {
            let snapshot: Vec<TabEventHandler> =
                handlers.iter().map(|e| e.handler.clone()).collect();
            for h in snapshot {
                h(state);
            }
        }
    }
}
