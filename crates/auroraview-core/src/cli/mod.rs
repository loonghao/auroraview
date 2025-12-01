//! CLI utility functions for AuroraView.
//!
//! This module provides core CLI utilities that can be used by both
//! the CLI tool and Python bindings.

mod html_rewrite;
mod url_utils;

pub use html_rewrite::rewrite_html_for_custom_protocol;
pub use url_utils::{normalize_url, UrlError};
