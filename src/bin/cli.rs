//! AuroraView CLI - Standalone WebView launcher
//!
//! This CLI tool allows users to launch a WebView window from the command line,
//! either with a URL or a local HTML file.
//!
//! ## Usage
//!
//! ```bash
//! # Load a URL
//! auroraview --url https://example.com
//!
//! # Load a local HTML file
//! auroraview --html /path/to/file.html
//!
//! # Show help
//! auroraview --help
//! ```

use anyhow::{Context, Result};
use auroraview_core::webview::protocol_handlers;
use clap::Parser;
use std::path::{Path, PathBuf};
use tao::event_loop::{ControlFlow, EventLoop};
use url::Url;
use wry::WebViewBuilder as WryWebViewBuilder;

/// AuroraView - Standalone WebView launcher
#[derive(Parser, Debug)]
#[command(
    name = "auroraview",
    version,
    about = "Launch a WebView window with a URL or local HTML file",
    long_about = None
)]
struct Args {
    /// URL to load in the WebView
    #[arg(short, long, conflicts_with = "html")]
    url: Option<String>,

    /// Local HTML file to load in the WebView
    #[arg(short = 'f', long, conflicts_with = "url")]
    html: Option<PathBuf>,

    /// Assets root directory for local HTML files (defaults to HTML file's directory)
    #[arg(long, requires = "html")]
    assets_root: Option<PathBuf>,

    /// Window title
    #[arg(short, long, default_value = "AuroraView")]
    title: String,

    /// Window width (set to 0 to maximize)
    #[arg(long, default_value = "800")]
    width: u32,

    /// Window height (set to 0 to maximize)
    #[arg(long, default_value = "600")]
    height: u32,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Allow opening new windows (e.g., via window.open)
    #[arg(long)]
    allow_new_window: bool,

    /// Enable file:// protocol support (allows loading local files from HTML)
    #[arg(long)]
    allow_file_protocol: bool,
}

/// Normalize URL by adding https:// prefix if missing
fn normalize_url(url_str: &str) -> Result<String> {
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

/// Rewrite HTML to use auroraview:// protocol for relative paths
fn rewrite_html_for_custom_protocol(html: &str) -> String {
    use regex::Regex;

    let mut result = html.to_string();

    // Helper function to check if a path is relative (not absolute URL or data URI)
    fn is_relative_path(path: &str) -> bool {
        !path.starts_with("http://")
            && !path.starts_with("https://")
            && !path.starts_with("data:")
            && !path.starts_with("//")  // Protocol-relative URLs
            && !path.starts_with("auroraview://") // Already rewritten
    }

    // Rewrite link href - match any href attribute
    let link_re = Regex::new(r#"<link\s+([^>]*)href="([^"]+)""#).unwrap();
    result = link_re
        .replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let path = &caps[2];
            if is_relative_path(path) {
                format!(r#"<link {}href="auroraview://{}""#, attrs, path)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    // Rewrite script src
    let script_re = Regex::new(r#"<script\s+([^>]*)src="([^"]+)""#).unwrap();
    result = script_re
        .replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let path = &caps[2];
            if is_relative_path(path) {
                format!(r#"<script {}src="auroraview://{}""#, attrs, path)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    // Rewrite img src
    let img_re = Regex::new(r#"<img\s+([^>]*)src="([^"]+)""#).unwrap();
    result = img_re
        .replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let path = &caps[2];
            if is_relative_path(path) {
                format!(r#"<img {}src="auroraview://{}""#, attrs, path)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    // Rewrite CSS url()
    let css_url_re = Regex::new(r#"url\(["']?([^"':)]+)["']?\)"#).unwrap();
    result = css_url_re
        .replace_all(&result, |caps: &regex::Captures| {
            let path = &caps[1];
            if is_relative_path(path) {
                format!(r#"url("auroraview://{}")"#, path)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    result
}

/// Detect assets root directory based on browser save patterns
///
/// Browsers save web pages with different patterns:
/// - Chrome: "page.html" + "page_files/" folder
/// - Firefox: "page.html" + "page_files/" folder
/// - Edge: "page.html" + "page_files/" folder
/// - Safari: "page.html" + "page_files/" folder (on macOS)
///
/// This function checks for these patterns and returns the appropriate assets root.
fn detect_assets_root(html_path: &Path) -> Result<PathBuf> {
    let parent = html_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine parent directory of HTML file"))?;

    let file_stem = html_path
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine HTML file name"))?
        .to_string_lossy();

    // Common browser save patterns
    let patterns = vec![
        format!("{}_files", file_stem),    // Chrome, Firefox, Edge (English)
        format!("{}.files", file_stem),    // Alternative pattern
        format!("{} 文件", file_stem),     // Chrome (Chinese)
        format!("{}_fichiers", file_stem), // Firefox (French)
        format!("{}_archivos", file_stem), // Firefox (Spanish)
        format!("{}_dateien", file_stem),  // Firefox (German)
    ];

    // Check each pattern
    for pattern in patterns {
        let candidate = parent.join(&pattern);
        if candidate.exists() && candidate.is_dir() {
            tracing::info!(
                "Found browser assets folder: {} (pattern: {})",
                candidate.display(),
                pattern
            );
            return Ok(candidate);
        }
    }

    // No assets folder found, use HTML file's parent directory
    tracing::info!("No browser assets folder found, using parent directory");
    Ok(parent.to_path_buf())
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging with appropriate level
    let log_level = if args.debug { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .with_target(false)
        .init();

    // Validate that at least one of url or html is provided
    if args.url.is_none() && args.html.is_none() {
        anyhow::bail!("Either --url or --html must be provided. Use --help for more information.");
    }

    // Determine the content to load and assets root
    let (html_content, assets_root) = if let Some(url_str) = &args.url {
        // For URLs, just normalize and no assets root
        let url = normalize_url(url_str)?;
        tracing::info!("Loading URL: {}", url);
        (None, None)
    } else if let Some(html_path) = &args.html {
        // Validate that the file exists
        if !html_path.exists() {
            anyhow::bail!("HTML file not found: {}", html_path.display());
        }

        // Determine assets root
        let assets_root = if let Some(root) = args.assets_root {
            tracing::info!("Using custom assets root: {}", root.display());
            root
        } else {
            // Auto-detect assets root based on browser save patterns
            let auto_root = detect_assets_root(html_path)?;
            tracing::info!("Auto-detected assets root: {}", auto_root.display());
            auto_root
        };

        // Read HTML content
        let html_content = std::fs::read_to_string(html_path)
            .with_context(|| format!("Failed to read HTML file: {}", html_path.display()))?;

        tracing::info!("Read HTML file ({} bytes)", html_content.len());

        // Rewrite HTML to use auroraview:// protocol for relative paths
        let rewritten_html = rewrite_html_for_custom_protocol(&html_content);

        tracing::info!("Rewrote HTML for auroraview:// protocol");
        tracing::info!("Assets root: {}", assets_root.display());

        (Some(rewritten_html), Some(assets_root))
    } else {
        unreachable!("Either url or html must be provided");
    };

    // Create event loop and window
    let event_loop = EventLoop::new();
    let mut window_builder = tao::window::WindowBuilder::new()
        .with_title(&args.title)
        .with_visible(false); // Start hidden to avoid white flash

    // If width or height is 0, maximize the window; otherwise set the size
    if args.width == 0 || args.height == 0 {
        tracing::info!("[CLI] Maximizing window (width or height is 0)");
        window_builder = window_builder.with_maximized(true);
    } else {
        window_builder =
            window_builder.with_inner_size(tao::dpi::LogicalSize::new(args.width, args.height));
    }

    let window = window_builder
        .build(&event_loop)
        .context("Failed to create window")?;

    // Create WebView with custom protocol support
    let mut webview_builder = WryWebViewBuilder::new();

    // Register auroraview:// protocol if assets_root is set
    // Use the project's protocol handler from protocol_handlers module
    if let Some(asset_root) = assets_root.clone() {
        tracing::info!(
            "[CLI] Registering auroraview:// protocol with asset_root: {}",
            asset_root.display()
        );
        webview_builder = webview_builder.with_custom_protocol(
            "auroraview".into(),
            move |_webview_id, request| {
                protocol_handlers::handle_auroraview_protocol(&asset_root, request)
            },
        );
    }

    // Register file:// protocol if enabled
    if args.allow_file_protocol {
        tracing::info!("[CLI] Enabling file:// protocol support");
        webview_builder = webview_builder
            .with_custom_protocol("file".into(), |_webview_id, request| {
                protocol_handlers::handle_file_protocol(request)
            });
    }

    // Enable DevTools in debug mode
    if args.debug {
        tracing::info!("[CLI] Enabling DevTools (debug mode)");
        webview_builder = webview_builder.with_devtools(true);
    }

    // Configure new window handler
    if args.allow_new_window {
        tracing::info!("[CLI] Allowing new windows");
        webview_builder = webview_builder.with_new_window_req_handler(|url, _features| {
            tracing::info!("[CLI] New window requested: {}", url);
            wry::NewWindowResponse::Allow
        });
    } else {
        tracing::info!("[CLI] Blocking new windows");
        webview_builder = webview_builder.with_new_window_req_handler(|url, _features| {
            tracing::info!("[CLI] Blocked new window request: {}", url);
            wry::NewWindowResponse::Deny
        });
    }

    // Load HTML content or URL
    let webview = if let Some(html) = html_content {
        tracing::info!("[CLI] Loading HTML content via with_html()");
        webview_builder
            .with_html(html)
            .build(&window)
            .context("Failed to create WebView with HTML content")?
    } else if let Some(url_str) = &args.url {
        let url = normalize_url(url_str)?;
        tracing::info!("[CLI] Loading URL: {}", url);
        webview_builder
            .with_url(&url)
            .build(&window)
            .context("Failed to create WebView with URL")?
    } else {
        unreachable!("Either html_content or url must be set");
    };

    tracing::info!("WebView created successfully");

    // Show window immediately - content is already loaded via data URL or direct URL
    window.set_visible(true);
    tracing::info!("Window shown");

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Keep webview alive
        let _ = &webview;

        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            tracing::info!("Window close requested");
            *control_flow = ControlFlow::Exit;
        }
    });
}
