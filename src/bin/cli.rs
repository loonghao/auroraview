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
use clap::Parser;
use std::path::PathBuf;
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

    /// Window title
    #[arg(short, long, default_value = "AuroraView")]
    title: String,

    /// Window width
    #[arg(long, default_value = "800")]
    width: u32,

    /// Window height
    #[arg(long, default_value = "600")]
    height: u32,
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    // Validate that at least one of url or html is provided
    if args.url.is_none() && args.html.is_none() {
        anyhow::bail!("Either --url or --html must be provided. Use --help for more information.");
    }

    // Determine the content to load
    let content_url = if let Some(url_str) = args.url {
        // Validate and parse URL
        Url::parse(&url_str)
            .with_context(|| format!("Invalid URL: {}", url_str))?
            .to_string()
    } else if let Some(html_path) = args.html {
        // Validate that the file exists
        if !html_path.exists() {
            anyhow::bail!("HTML file not found: {}", html_path.display());
        }

        // Convert to absolute path and create file:// URL
        let absolute_path = html_path
            .canonicalize()
            .with_context(|| format!("Failed to resolve path: {}", html_path.display()))?;

        #[cfg(windows)]
        let url = format!(
            "file:///{}",
            absolute_path.to_string_lossy().replace('\\', "/")
        );

        #[cfg(not(windows))]
        let url = format!("file://{}", absolute_path.to_string_lossy());

        url
    } else {
        unreachable!("Either url or html must be provided");
    };

    tracing::info!("Loading content: {}", content_url);

    // Create event loop and window
    let event_loop = EventLoop::new();
    let window = tao::window::WindowBuilder::new()
        .with_title(&args.title)
        .with_inner_size(tao::dpi::LogicalSize::new(args.width, args.height))
        .build(&event_loop)
        .context("Failed to create window")?;

    // Create WebView
    let _webview = WryWebViewBuilder::new()
        .with_url(&content_url)
        .build(&window)
        .context("Failed to create WebView")?;

    tracing::info!("WebView created successfully");

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

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
