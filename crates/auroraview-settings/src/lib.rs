//! Settings and preferences management for AuroraView.
//!
//! This crate provides a unified settings system with support for:
//! - Type-safe preference access
//! - Nested settings with dot notation
//! - Default values and validation
//! - Persistence to JSON files
//! - Change notifications
//!
//! # Example
//!
//! ```rust
//! use auroraview_settings::{SettingsManager, SettingValue};
//!
//! let mut manager = SettingsManager::new();
//!
//! // Set values
//! manager.set("appearance.theme", SettingValue::String("dark".into()));
//! manager.set("browser.homepage", SettingValue::String("https://example.com".into()));
//! manager.set("privacy.do_not_track", SettingValue::Bool(true));
//!
//! // Get values with type safety
//! let theme = manager.get_string("appearance.theme");
//! let dnt = manager.get_bool("privacy.do_not_track");
//! ```

mod error;
mod schema;
mod store;
mod value;
mod manager;

pub use error::{SettingsError, Result};
pub use schema::{SettingSchema, SchemaType, SchemaRegistry};
pub use store::SettingsStore;
pub use value::SettingValue;
pub use manager::SettingsManager;
