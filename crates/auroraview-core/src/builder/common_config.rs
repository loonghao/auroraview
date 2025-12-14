//! Common WebView configuration utilities
//!
//! This module provides shared configuration logic for WebView builders,
//! including background color, devtools, and initialization scripts.

/// Dark background color (Tailwind slate-950: #020617)
/// Used to prevent white flash during WebView initialization
pub const DARK_BACKGROUND: (u8, u8, u8, u8) = (2, 6, 23, 255);

/// Get the dark background color as RGBA tuple
///
/// Returns the standard dark background color used by AuroraView
/// to prevent white flash during WebView initialization.
///
/// # Returns
/// RGBA tuple (2, 6, 23, 255) representing #020617
pub fn get_background_color() -> (u8, u8, u8, u8) {
    DARK_BACKGROUND
}

/// Log background color configuration
pub fn log_background_color(color: (u8, u8, u8, u8)) {
    tracing::info!(
        "Set WebView background color to #{:02x}{:02x}{:02x}",
        color.0,
        color.1,
        color.2
    );
}
