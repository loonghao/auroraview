//! AuroraView CLI Library
//!
//! This library exposes internal modules for testing purposes.
//! The main entry point is the binary crate.

/// CLI argument parsing and subcommand definitions.
pub mod cli;
/// Packed application support: Python embedding, overlay extraction, and runtime setup.
pub mod packed;
/// Custom URI protocol handler registration.
pub mod protocol_handlers;

/// Packed application utility functions re-exported for convenience.
pub use packed::{
    build_module_search_paths, build_packed_init_script_with_csp, escape_json_for_js,
    get_python_exe_path, get_runtime_cache_dir_with_hash, get_webview_data_dir,
    inject_environment_variables,
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

/// Normalize URL by adding https:// prefix if missing.
///
/// Delegates to [`auroraview_core::cli::normalize_url`] and converts the error
/// to [`anyhow::Error`] for convenience.
pub fn normalize_url(url_str: &str) -> anyhow::Result<String> {
    auroraview_core::cli::normalize_url(url_str).map_err(|e| anyhow::anyhow!("{}", e))
}
