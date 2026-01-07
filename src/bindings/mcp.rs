//! MCP Server Python bindings
//!
//! This module re-exports the MCP Server bindings from the auroraview-mcp crate.

use pyo3::prelude::*;

/// Register MCP module functions and classes
pub fn register_mcp_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Re-export from auroraview-mcp crate
    auroraview_mcp::register_mcp_module(m)?;
    Ok(())
}
