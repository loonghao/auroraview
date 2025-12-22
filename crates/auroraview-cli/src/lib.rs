//! AuroraView CLI Library
//!
//! This library exposes internal modules for testing purposes.
//! The main entry point is the binary crate.

pub mod cli;
pub mod packed;
pub mod protocol_handlers;

// Re-export utilities used by other modules
pub use packed::{
    build_module_search_paths, build_packed_init_script, escape_json_for_js, get_python_exe_path,
    get_runtime_cache_dir_with_hash, get_webview_data_dir, inject_environment_variables,
};

/// Embedded window icon (32x32 PNG) - default fallback
pub const ICON_PNG_BYTES: &[u8] = include_bytes!("../../../assets/icons/auroraview-32.png");

/// Load window icon from embedded PNG bytes (default icon)
pub fn load_window_icon() -> Option<tao::window::Icon> {
    load_window_icon_from_bytes(ICON_PNG_BYTES)
}

/// Load window icon from custom PNG bytes
/// Used by packed applications to load custom window icons
pub fn load_window_icon_from_bytes(png_bytes: &[u8]) -> Option<tao::window::Icon> {
    use ::image::GenericImageView;

    let img = ::image::load_from_memory(png_bytes).ok()?;
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
