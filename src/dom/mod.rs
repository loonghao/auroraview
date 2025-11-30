//! High-performance DOM manipulation module for AuroraView.
//!
//! This module provides Rust-native DOM operations that generate optimized JavaScript.
//! Using Rust for JS generation provides 5-10x performance improvement over Python,
//! and batch operations reduce IPC overhead by up to 90%.
//!
//! # Example
//!
//! ```python
//! from auroraview import DomBatch
//!
//! # Batch multiple operations - only one IPC call!
//! batch = DomBatch()
//! batch.set_text("#title", "Hello AuroraView!")
//! batch.add_class(".items", "active")
//! batch.click("#submit")
//!
//! # Execute all operations at once
//! webview.eval_js(batch.to_js())
//! ```

mod ops;

pub use ops::{DomBatch, DomOp};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

/// Register DOM module with Python
#[cfg(feature = "python-bindings")]
pub fn register_dom_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DomBatch>()?;
    Ok(())
}
