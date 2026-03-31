//! Run command - Launch a WebView window

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
use wry::{WebContext, WebViewBuilder as WryWebViewBuilder};

use auroraview_core::cli::rewrite_html_for_custom_protocol;

use crate::{get_webview_data_dir, load_window_icon, normalize_url, protocol_handlers};

/// Arguments for the 'run' subcommand
#[derive(Parser, Debug)]
pub struct RunArgs {
    /// URL to load in the WebView
    #[arg(short, long, conflicts_with = "html")]
    pub url: Option<String>,

    /// Local HTML file to load in the WebView
    #[arg(short = 'f', long, conflicts_with = "url")]
    pub html: Option<PathBuf>,

    /// Assets root directory for local HTML files
    #[arg(long, requires = "html")]
    pub assets_root: Option<PathBuf>,

    /// Window title
    #[arg(short, long, default_value = "AuroraView")]
    pub title: String,

    /// Window width (set to 0 to maximize)
    #[arg(long, default_value = "800")]
    pub width: u32,

    /// Window height (set to 0 to maximize)
    #[arg(long, default_value = "600")]
    pub height: u32,

    /// Enable debug logging
    #[arg(short, long)]
    pub debug: bool,

    /// Allow opening new windows
    #[arg(long)]
    pub allow_new_window: bool,

    /// Enable file:// protocol support
    #[arg(long)]
    pub allow_file_protocol: bool,

    /// Keep window always on top
    #[arg(long)]
    pub always_on_top: bool,

    /// Watch the HTML file for changes and reload automatically (hot reload)
    #[arg(long, requires = "html")]
    pub watch: bool,
}

/// User event sent from the file watcher thread to the event loop
#[derive(Debug, Clone)]
pub enum RunEvent {
    /// Reload the current page
    Reload,
}

/// Detect assets root directory based on browser save patterns
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

/// Spawn a background thread that watches `path` for modifications.
///
/// When a change is detected, a [`RunEvent::Reload`] is sent through `proxy`.
/// The watcher is kept alive for the lifetime of the returned handle; dropping
/// it stops the watch.
fn start_file_watcher(
    path: PathBuf,
    proxy: EventLoopProxy<RunEvent>,
) -> Result<notify::RecommendedWatcher> {
    use notify::{Config, EventKind, RecursiveMode, Watcher};

    let mut watcher = notify::RecommendedWatcher::new(
        move |result: notify::Result<notify::Event>| match result {
            Ok(event) => {
                if matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                ) {
                    tracing::debug!("[hot-reload] Change detected: {:?}", event.paths);
                    if proxy.send_event(RunEvent::Reload).is_err() {
                        // Event loop has closed; watcher will be dropped soon
                        tracing::debug!("[hot-reload] Event loop closed, stopping watcher");
                    }
                }
            }
            Err(e) => tracing::warn!("[hot-reload] Watch error: {}", e),
        },
        Config::default(),
    )
    .context("Failed to create file watcher")?;

    watcher
        .watch(&path, RecursiveMode::NonRecursive)
        .with_context(|| format!("Failed to watch path: {}", path.display()))?;

    tracing::info!("[hot-reload] Watching: {}", path.display());
    Ok(watcher)
}

/// Read and rewrite an HTML file for the auroraview:// custom protocol
fn load_html_file(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read HTML file: {}", path.display()))?;
    Ok(rewrite_html_for_custom_protocol(&content))
}

/// Run the WebView window
pub fn run_webview(args: RunArgs) -> Result<()> {
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
        let html_content = load_html_file(html_path)?;
        tracing::info!("Read HTML file ({} bytes)", html_content.len());
        tracing::info!("Assets root: {}", assets_root.display());

        (Some(html_content), Some(assets_root))
    } else {
        anyhow::bail!("Either --url or --html must be provided");
    };

    // Create event loop — generic over RunEvent for hot-reload signalling
    let event_loop: EventLoop<RunEvent> = EventLoopBuilder::<RunEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let mut window_builder = tao::window::WindowBuilder::new()
        .with_title(&args.title)
        .with_visible(false); // Start hidden to avoid white flash

    // Set window icon from embedded ICO
    if let Some(icon) = load_window_icon() {
        window_builder = window_builder.with_window_icon(Some(icon));
        tracing::info!("[CLI] Window icon set");
    }

    // If width or height is 0, maximize the window; otherwise set the size
    if args.width == 0 || args.height == 0 {
        tracing::info!("[CLI] Maximizing window (width or height is 0)");
        window_builder = window_builder.with_maximized(true);
    } else {
        window_builder =
            window_builder.with_inner_size(tao::dpi::LogicalSize::new(args.width, args.height));
    }

    // Set always on top if requested
    if args.always_on_top {
        tracing::info!("[CLI] Setting window to always on top");
        window_builder = window_builder.with_always_on_top(true);
    }

    let window = window_builder
        .build(&event_loop)
        .context("Failed to create window")?;

    // Create WebContext with user data in AppData (not current directory)
    let data_dir = get_webview_data_dir();
    tracing::info!("[CLI] WebView2 user data directory: {}", data_dir.display());
    let mut web_context = WebContext::new(Some(data_dir));

    // Create WebView with custom protocol support
    let mut webview_builder = WryWebViewBuilder::new_with_web_context(&mut web_context);

    // Register auroraview:// protocol if assets_root is set
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
        tracing::info!("[CLI] Allowing new windows (opens in system browser)");
        webview_builder = webview_builder.with_new_window_req_handler(|url, _features| {
            tracing::info!("[CLI] New window requested: {}", url);
            if let Err(e) = open::that(&url) {
                tracing::error!("[CLI] Failed to open URL in browser: {}", e);
            }
            wry::NewWindowResponse::Deny
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
        anyhow::bail!("Either html_content or url must be set");
    };

    tracing::info!("WebView created successfully");

    // Start file watcher for hot reload (only for --html --watch)
    // The watcher must outlive the event loop; we keep it in a local binding.
    let _watcher = if args.watch {
        if let Some(ref html_path) = args.html {
            let canonical = html_path
                .canonicalize()
                .unwrap_or_else(|_| html_path.clone());
            match start_file_watcher(canonical, proxy) {
                Ok(w) => {
                    tracing::info!("[hot-reload] File watcher started");
                    Some(w)
                }
                Err(e) => {
                    tracing::warn!("[hot-reload] Failed to start file watcher: {}", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // The html_path is captured for reload; clone it outside the closure
    let html_path_for_reload = args.html.clone();

    // Show window immediately
    window.set_visible(true);
    tracing::info!("Window shown");

    // Run event loop — never returns; process exits when window closes.
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Keep webview alive
        let _ = &webview;

        match event {
            tao::event::Event::WindowEvent {
                event: tao::event::WindowEvent::CloseRequested,
                ..
            } => {
                tracing::info!("Window close requested");
                *control_flow = ControlFlow::Exit;
            }
            tao::event::Event::UserEvent(RunEvent::Reload) => {
                tracing::info!("[hot-reload] Reloading WebView");
                if let Some(ref path) = html_path_for_reload {
                    match load_html_file(path) {
                        Ok(html) => {
                            if let Err(e) = webview.load_html(&html) {
                                tracing::warn!("[hot-reload] load_html failed: {}", e);
                                // Fall back to location.reload()
                                let _ = webview.evaluate_script("location.reload();");
                            }
                        }
                        Err(e) => {
                            tracing::warn!("[hot-reload] Failed to read HTML: {}", e);
                        }
                    }
                } else {
                    let _ = webview.evaluate_script("location.reload();");
                }
            }
            _ => {}
        }
    });

    // Unreachable: event_loop.run() never returns on most platforms.
    #[allow(unreachable_code)]
    Ok(())
}
