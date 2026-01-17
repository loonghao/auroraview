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

pub mod a11y;
pub mod cdp;
pub mod error;
pub mod inspector;
pub mod snapshot;

#[cfg(feature = "python")]
pub mod python;

// Re-exports
pub use error::{InspectorError, Result};
pub use inspector::{Inspector, InspectorConfig};
pub use snapshot::{
    ActionResult, RefId, RefInfo, ScrollDirection, Snapshot, SnapshotFormat, WaitCondition,
};
