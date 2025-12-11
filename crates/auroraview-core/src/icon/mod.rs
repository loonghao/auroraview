//! Icon utilities for AuroraView
//!
//! Provides functionality for:
//! - Loading PNG images and converting to RGBA data (for window icons)
//! - Converting PNG to ICO format (for EXE icons)
//! - Compressing PNG images
//!
//! # Example
//!
//! ```no_run
//! use auroraview_core::icon::{load_icon_rgba, png_to_ico, compress_png};
//!
//! // Load PNG as RGBA for window icon
//! let icon_data = load_icon_rgba("icon.png").unwrap();
//! let (rgba, width, height) = (icon_data.rgba, icon_data.width, icon_data.height);
//!
//! // Convert PNG to ICO for EXE embedding
//! png_to_ico("icon.png", "icon.ico", &[16, 32, 48, 256]).unwrap();
//!
//! // Compress PNG
//! compress_png("large.png", "small.png", 9).unwrap();
//! ```

mod compress;
mod converter;
mod loader;

pub use compress::{compress_and_resize, compress_png, CompressionLevel, CompressionResult};
pub use converter::{png_bytes_to_ico, png_to_ico, IcoConfig};
pub use loader::{load_icon_rgba, IconData};

/// Default icon sizes for ICO files (Windows standard)
pub const DEFAULT_ICO_SIZES: &[u32] = &[16, 32, 48, 256];
