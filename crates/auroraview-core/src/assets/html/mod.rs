//! HTML templates and assets
//!
//! This module provides HTML templates for error pages, loading screens, and other UI components.

mod error_pages;

pub use error_pages::{
    connection_error_page, internal_error_page, loading_with_error, not_found_page,
    python_error_page, startup_error_page,
};
