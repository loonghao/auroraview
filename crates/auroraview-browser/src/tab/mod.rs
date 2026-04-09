//! Tab management module
//!
//! This module provides multi-tab browser functionality following
//! Microsoft WebView2Browser architecture patterns.

mod events;
mod listener;
mod manager;
mod state;

pub use events::TabEvent;
pub use listener::{TabEventHandler, TabEventKind, TabListenerId, TabListenerMap};
pub use manager::TabManager;
pub use state::{SecurityState, Tab, TabId, TabState};
