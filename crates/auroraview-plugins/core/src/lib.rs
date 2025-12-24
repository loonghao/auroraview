//! AuroraView Plugin Core
//!
//! This crate provides the core plugin system framework for AuroraView.
//! It defines the traits, types, and routing infrastructure that plugins use.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    JavaScript API                            │
//! │  window.auroraview.fs.readFile()                            │
//! │  window.auroraview.clipboard.write()                        │
//! │  window.auroraview.shell.open()                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │              Plugin Command Router                           │
//! │  invoke("plugin:fs|read_file", { path, ... })               │
//! ├────────────┬────────────┬────────────┬──────────────────────┤
//! │ fs_plugin  │ clipboard  │ shell      │ dialog               │
//! ├────────────┴────────────┴────────────┴──────────────────────┤
//! │               auroraview-plugin-core                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Command Format
//!
//! Plugin commands use the format: `plugin:<plugin_name>|<command_name>`
//!
//! Example: `plugin:fs|read_file`

mod error;
mod handler;
mod request;
mod router;
mod scope;
mod types;

pub use error::{PluginError, PluginErrorCode, PluginResult};
pub use handler::PluginHandler;
pub use request::{PluginRequest, PluginResponse};
pub use router::{PluginEventCallback, PluginRouter};
pub use scope::{PathScope, ScopeConfig, ScopeError, ShellScope};
pub use types::PluginCommand;
