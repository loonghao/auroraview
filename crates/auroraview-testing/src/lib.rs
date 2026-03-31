//! AuroraView Testing Framework
//!
//! AI-friendly testing and inspection for AuroraView WebView applications.
//!
//! # Quick Start
//!
//! ```ignore
//! use auroraview_testing::Inspector;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to running AuroraView instance
//!     let inspector = Inspector::connect("http://localhost:9222").await?;
//!
//!     // Get page snapshot
//!     let snapshot = inspector.snapshot().await?;
//!     println!("{}", snapshot);
//!
//!     // Interact with elements using refs
//!     inspector.click("@3").await?;
//!     inspector.fill("@4", "hello").await?;
//!
//!     inspector.close().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - **AI-friendly snapshots** - Structured page state with interactive element refs
//! - **Simple interaction** - Click, fill, press keys using ref IDs
//! - **Navigation** - goto, back, forward, reload
//! - **Wait conditions** - Wait for text, elements, URLs, or custom JS
//! - **Zero external deps** - Core uses only WebSocket for CDP
//!
//! # Snapshot Format
//!
//! ```text
//! Page: "AuroraView Gallery" (http://localhost:5173/)
//! Viewport: 1280x720
//!
//! Interactive Elements (23 refs):
//!   @1  [link] "Home" - navigation
//!   @2  [textbox] "Search..." - search input
//!   @3  [button] "Filter: All" - dropdown
//!   ...
//!
//! Page Structure:
//!   header
//!     nav: Home [@1] | Gallery [@2]
//!   main
//!     ...
//! ```

/// Accessibility tree extraction and ARIA role mapping.
pub mod a11y;
/// Chrome DevTools Protocol (CDP) WebSocket client.
pub mod cdp;
/// Testing framework error types.
pub mod error;
/// Page inspector: connect, snapshot, interact, and wait.
pub mod inspector;
/// AI-friendly page snapshot format with interactive element refs.
pub mod snapshot;

/// Python bindings via PyO3 (when `python` feature is enabled).
#[cfg(feature = "python")]
pub mod python;

/// Error and result types for inspector operations.
pub use error::{InspectorError, Result};
/// Inspector client and configuration types.
pub use inspector::{Inspector, InspectorConfig};
/// Snapshot types: actions, refs, scroll, wait conditions, and formats.
pub use snapshot::{
    ActionResult, RefId, RefInfo, ScrollDirection, Snapshot, SnapshotFormat, WaitCondition,
};
