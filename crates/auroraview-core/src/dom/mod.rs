//! DOM operation types and batch processing for AuroraView.
//!
//! This module provides core DOM manipulation primitives that generate
//! optimized JavaScript code. No Python bindings - pure Rust core logic.
//!
//! # Example (Rust)
//!
//! ```rust
//! use auroraview_core::dom::{DomBatch, DomOp};
//!
//! let mut batch = DomBatch::new();
//! batch.set_text("#title", "Hello!");
//! batch.add_class(".items", "active");
//! let js = batch.to_js();
//! ```

mod ops;

pub use ops::{DomBatch, DomOp};
