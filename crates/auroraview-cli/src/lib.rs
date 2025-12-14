//! AuroraView CLI Library
//!
//! This library exposes internal modules for testing purposes.
//! The main entry point is the binary crate.

pub mod cli;
pub mod packed;
pub mod protocol_handlers;

// Re-export utilities used by other modules
pub use packed::get_webview_data_dir;

/// Embedded window icon (32x32 PNG)
pub const ICON_PNG_BYTES: &[u8] = include_bytes!("../../../assets/icons/auroraview-32.png");

/// Load window icon from embedded PNG bytes
pub fn load_window_icon() -> Option<tao::window::Icon> {
    use ::image::GenericImageView;

    let img = ::image::load_from_memory(ICON_PNG_BYTES).ok()?;
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();

    tao::window::Icon::from_rgba(rgba, width, height).ok()
}

/// Normalize URL by adding https:// prefix if missing
pub fn normalize_url(url_str: &str) -> anyhow::Result<String> {
    use anyhow::Context;
    use url::Url;

    // If it already has a scheme, validate and return
    if url_str.contains("://") {
        let url = Url::parse(url_str).with_context(|| format!("Invalid URL: {}", url_str))?;
        return Ok(url.to_string());
    }

    // Add https:// prefix for URLs without scheme
    let with_scheme = format!("https://{}", url_str);
    let url = Url::parse(&with_scheme).with_context(|| format!("Invalid URL: {}", url_str))?;
    Ok(url.to_string())
}
