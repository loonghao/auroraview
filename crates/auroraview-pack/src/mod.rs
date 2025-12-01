//! AuroraView Pack Module - Standalone Executable Packaging
//!
//! This module provides functionality to package AuroraView-based applications
//! into standalone executables, similar to Pake.
//!
//! ## Features
//!
//! - **URL Mode**: Pack a URL into a standalone desktop app
//!   ```bash
//!   auroraview pack --url https://example.com --output my-app
//!   ```
//!
//! - **Frontend Mode**: Pack local HTML/CSS/JS into a standalone app
//!   ```bash
//!   auroraview pack --frontend ./dist --output my-app
//!   ```
//!
//! - **Full Stack Mode**: Pack frontend + Python backend (requires PyOxidizer)
//!   ```bash
//!   auroraview pack --frontend ./dist --backend "myapp:main" --output my-app
//!   ```
//!
//! ## Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                    Packed Executable                        │
//! ├────────────────────────────────────────────────────────────┤
//! │  ┌──────────────────┐  ┌──────────────────────────────────┐│
//! │  │  AuroraView CLI  │  │      Embedded Resources          ││
//! │  │  (Wry/Tao)       │  │  - Frontend HTML/CSS/JS          ││
//! │  │  WebView Engine  │  │  - Python Runtime (optional)     ││
//! │  └──────────────────┘  └──────────────────────────────────┘│
//! └────────────────────────────────────────────────────────────┘
//! ```

mod config;
mod error;
mod generator;
mod pyembed_integration;
mod templates;

pub use config::{PackConfig, PackMode};
pub use error::PackError;
pub use generator::PackGenerator;
pub use pyembed_integration::{PyEmbedConfig, generate_pyoxidizer_config, check_pyembed_availability};

/// Result type for pack operations
pub type PackResult<T> = Result<T, PackError>;

