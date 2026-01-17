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

pub mod cdp;

pub use config::{DevToolsConfig, DockSide};
pub use console::{ConsoleMessage, ConsoleMessageType};
pub use error::{DevToolsError, Result};
pub use manager::{DevToolsManager, DevToolsState};
pub use network::{NetworkRequestInfo, NetworkResponseInfo};

/// CDP session info
pub use cdp::CdpSessionInfo;
