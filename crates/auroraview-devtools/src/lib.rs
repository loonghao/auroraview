//! AuroraView DevTools
//!
//! This crate provides DevTools and CDP (Chrome DevTools Protocol) support
//! that can be used by both WebView and Browser applications.
//!
//! # Features
//!
//! - DevTools panel management
//! - CDP remote debugging support
//! - Console message capture
//! - Network request inspection
//!
//! # Example
//!
//! ```rust,ignore
//! use auroraview_devtools::{DevToolsManager, DevToolsConfig};
//!
//! let config = DevToolsConfig {
//!     enabled: true,
//!     remote_debugging_port: 9222,
//!     ..Default::default()
//! };
//!
//! let mut manager = DevToolsManager::new(config);
//! manager.open();
//! ```

mod config;
mod console;
mod error;
mod manager;
mod network;

/// Chrome DevTools Protocol (CDP) client and session management.
pub mod cdp;

/// DevTools panel configuration and dock position types.
pub use config::{DevToolsConfig, DockSide};
/// Console message capture types.
pub use console::{ConsoleMessage, ConsoleMessageType};
/// Error and result types for DevTools operations.
pub use error::{DevToolsError, Result};
/// DevTools manager and panel state.
pub use manager::{DevToolsManager, DevToolsState};
/// Network request/response inspection types.
pub use network::{NetworkRequestInfo, NetworkResponseInfo};

/// CDP session info
pub use cdp::CdpSessionInfo;
