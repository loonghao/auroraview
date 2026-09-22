//! Run command - Launch a WebView window

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
use wry::{WebContext, WebViewBuilder as WryWebViewBuilder};

use auroraview_core::cli::rewrite_html_for_custom_protocol;
use auroraview_core::parent_ipc::{ChildInfo, ParentBridge};

use crate::cli::parent_window::{self, ParentTargets};
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

    /// Force-enable IPC `file_drop_*` events instead of native HTML5 drag-drop.
    ///
    /// Note: enabling this disables HTML5 dragover/drop inside the WebView
    /// due to a wry/WebView2 upstream limitation. The standalone `run`
    /// command has no IPC backend wired up, so dispatched events are
    /// logged but otherwise dropped — this flag is mostly useful for
    /// parity testing.
    #[arg(
        long = "capture-file-drop",
        action = clap::ArgAction::SetTrue,
        overrides_with = "no_capture_file_drop"
    )]
    pub capture_file_drop: bool,

    /// Force-disable IPC `file_drop_*` events.
    ///
    /// Both flags use `SetTrue`. `--no-capture-file-drop` stores `true`
    /// in `no_capture_file_drop` and `resolve_capture_file_drop` reads
    /// it as "user asked for false". Using `SetFalse` here would invert
    /// the resolver because clap's `SetFalse` defaults to `true` when
    /// the flag is absent.
    #[arg(
        long = "no-capture-file-drop",
        action = clap::ArgAction::SetTrue,
        overrides_with = "capture_file_drop"
    )]
    pub no_capture_file_drop: bool,

    /// Keep window always on top
    #[arg(long)]
    pub always_on_top: bool,

    /// Watch for changes and reload automatically.
    /// For --html: watches the file on disk.
    /// For --url: polls the server and reloads when it comes back online.
    #[arg(long)]
    pub watch: bool,

    /// Poll interval in milliseconds for URL-mode hot reload (default: 1500)
    #[arg(long, default_value = "1500", requires = "watch")]
    pub poll_interval_ms: u64,

    /// Attach the window as a child of this native window handle.
    ///
    /// On Windows this is an `HWND`, accepted as decimal or `0x`-prefixed
    /// hexadecimal. The window gets `WS_CHILD` and is clipped to the parent's
    /// client area, which is what an embedding host (Unity, Unreal, Qt, ...)
    /// wants for an in-viewport panel.
    ///
    /// Defaults to `AURORAVIEW_PARENT_HWND` when the flag is omitted.
    #[arg(
        long = "parent-hwnd",
        value_name = "HWND",
        conflicts_with = "owner_hwnd"
    )]
    pub parent_hwnd: Option<String>,

    /// Attach the window as an owned top-level window of this native handle.
    ///
    /// Unlike `--parent-hwnd`, an owned window is a separate top-level window:
    /// it floats above its owner, is hidden when the owner is minimized, and
    /// is destroyed with it. Use this for tool panels rather than embedded
    /// widgets.
    #[arg(long = "owner-hwnd", value_name = "HWND")]
    pub owner_hwnd: Option<String>,

    /// Exit when the parent IPC channel drops.
    ///
    /// Only meaningful in child mode (`AURORAVIEW_PARENT_ID` is set). Lets a
    /// host reap child processes by closing the socket instead of hunting for
    /// PIDs. Off by default so Gallery's existing lifecycle is unchanged.
    #[arg(long = "exit-on-parent-disconnect")]
    pub exit_on_parent_disconnect: bool,
}

/// Resolve the user's `--capture-file-drop` / `--no-capture-file-drop`
/// choice into a tri-state value.
///
/// - both flags absent → `None` (defer to lower layer / code default).
/// - `--capture-file-drop` only → `Some(true)`.
/// - `--no-capture-file-drop` only → `Some(false)`.
pub fn resolve_capture_file_drop(args: &RunArgs) -> Option<bool> {
    super::resolve_flag_pair(args.capture_file_drop, args.no_capture_file_drop)
}

/// Drag-drop sink for the standalone `run` command.
///
/// The `run` command renders a single WebView for visual / parity smoke
/// testing. It does **not** register an IPC handler with the WebView
/// (unlike `packed` / `embedded` paths), so any event we successfully
/// dispatch has no consumer on the JavaScript side. We log each event
/// at `debug` level so users who pass `--capture-file-drop` with
/// `RUST_LOG=debug` can confirm wry actually delivered the event.
struct RunDragDropSink;

impl auroraview_core::builder::DragDropIpcSink for RunDragDropSink {
    fn dispatch(
        &self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<(), auroraview_core::builder::DispatchError> {
        tracing::debug!(
            target: "auroraview::drag_drop",
            "[run] {} (no IPC handler registered; payload preview: {})",
            event_name,
            data
        );
        Ok(())
    }
}

/// User event sent from the file watcher thread to the event loop
#[derive(Debug, Clone)]
pub enum RunEvent {
    /// Reload the current page
    Reload,
    /// Evaluate a JavaScript string received from the parent over IPC
    Eval(String),
    /// Emit an event into the page, received from the parent over IPC
    Emit(String, String),
    /// Close the window, received from the parent over IPC
    Close,
}

/// Commands a parent may send on the `parent:command` event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentCommand {
    /// `{"command": "close"}`
    Close,
    /// `{"command": "eval", "args": {"js": "..."}}`
    Eval,
    /// `{"command": "emit", "args": {"event": "...", "data": ...}}`
    Emit,
}

impl ParentCommand {
    /// Parse the `command` discriminator of a `parent:command` payload.
    pub fn parse(name: Option<&str>) -> Option<Self> {
        match name {
            Some("close") => Some(Self::Close),
            Some("eval") => Some(Self::Eval),
            Some("emit") => Some(Self::Emit),
            _ => None,
        }
    }
}

/// Build the JavaScript that delivers a parent `emit` into the page.
///
/// Uses `window.auroraview.trigger()`, the single event entry point the
/// frontend SDK installs, so parent-originated events land in the same place
/// as host-originated ones.
fn emit_script(event: &str, data: &str) -> String {
    let payload = if data.trim().is_empty() {
        "null".to_string()
    } else {
        data.to_string()
    };
    format!(
        "window.auroraview.trigger({}, {});",
        serde_json::to_string(event).unwrap_or_else(|_| format!("{:?}", event)),
        payload
    )
}

/// Resolve the targets of a `parent:command` payload into a [`RunEvent`].
fn resolve_parent_command(data: &serde_json::Value) -> Option<RunEvent> {
    let command = ParentCommand::parse(data.get("command").and_then(|v| v.as_str()))?;
    let args = data.get("args").cloned().unwrap_or(serde_json::Value::Null);

    match command {
        ParentCommand::Close => Some(RunEvent::Close),
        ParentCommand::Eval => args
            .get("js")
            .and_then(|v| v.as_str())
            .map(|js| RunEvent::Eval(js.to_string())),
        ParentCommand::Emit => args.get("event").and_then(|v| v.as_str()).map(|event| {
            let data = args
                .get("data")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string());
            RunEvent::Emit(event.to_string(), data)
        }),
    }
}

/// Spawn a background thread that polls `url` (HTTP HEAD) every `interval`.
///
/// When the server transitions from unreachable → reachable, a
/// [`RunEvent::Reload`] is sent through `proxy`.
/// The `stop` flag lets the caller terminate the thread gracefully.
fn start_url_watcher(
    url: String,
    proxy: EventLoopProxy<RunEvent>,
    interval: Duration,
    stop: Arc<AtomicBool>,
) {
    std::thread::Builder::new()
        .name("url-hot-reload".into())
        .spawn(move || {
            // Determine the probe URL: for non-http(s) fall back to a no-op check.
            let probe_url = if url.starts_with("http://") || url.starts_with("https://") {
                url.clone()
            } else {
                tracing::debug!("[hot-reload] Non-HTTP URL, URL watcher is a no-op");
                return;
            };

            tracing::info!(
                "[hot-reload] URL watcher started for {} (poll interval: {:?})",
                probe_url,
                interval
            );

            // Track last reachability so we only reload on transitions.
            let mut last_reachable = probe_url_reachable(&probe_url);

            loop {
                if stop.load(Ordering::Relaxed) {
                    tracing::debug!("[hot-reload] URL watcher stopping");
                    break;
                }

                std::thread::sleep(interval);

                if stop.load(Ordering::Relaxed) {
                    break;
                }

                let now_reachable = probe_url_reachable(&probe_url);
                if now_reachable && !last_reachable {
                    tracing::info!("[hot-reload] Server back online, reloading: {}", probe_url);
                    if proxy.send_event(RunEvent::Reload).is_err() {
                        // Event loop has closed.
                        break;
                    }
                }
                last_reachable = now_reachable;
            }
        })
        .expect("failed to spawn url-hot-reload thread");
}

/// Issue an HTTP HEAD request and return `true` if the server responds with
/// any HTTP status code (we treat any response as "reachable").
fn probe_url_reachable(url: &str) -> bool {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    // Parse host:port from URL
    let (host, port, path) = match parse_http_url(url) {
        Some(parts) => parts,
        None => return false,
    };

    let addr = format!("{}:{}", host, port);
    let sock_addr = match addr.parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let stream = TcpStream::connect_timeout(&sock_addr, Duration::from_millis(800));

    let mut stream = match stream {
        Ok(s) => s,
        Err(_) => return false,
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(800)));
    let request = format!(
        "HEAD {} HTTP/1.0\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }

    let mut buf = [0u8; 16];
    // Any bytes back means the server is alive.
    matches!(stream.read(&mut buf), Ok(n) if n > 0)
}

/// Parse an HTTP/HTTPS URL into (host, port, path).
fn parse_http_url(url: &str) -> Option<(String, u16, String)> {
    let (scheme, rest) = if let Some(s) = url.strip_prefix("https://") {
        ("https", s)
    } else {
        let s = url.strip_prefix("http://")?;
        ("http", s)
    };

    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], rest[i..].to_string()),
        None => (rest, "/".to_string()),
    };

    let (host, port) = if let Some(i) = authority.rfind(':') {
        let p: u16 = authority[i + 1..].parse().ok()?;
        (authority[..i].to_string(), p)
    } else {
        let p = if scheme == "https" { 443 } else { 80 };
        (authority.to_string(), p)
    };

    Some((host, port, path))
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

/// Release the resources that outlive the event loop.
///
/// Stops the hot-reload watcher and closes the parent IPC channel, announcing
/// `child:closing` so the host does not have to wait for a socket timeout to
/// notice the child is gone.
fn shutdown(stop: &std::sync::atomic::AtomicBool, bridge: Option<&ParentBridge>) {
    stop.store(true, Ordering::Relaxed);
    if let Some(bridge) = bridge {
        bridge.disconnect();
    }
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

    // Resolve --capture-file-drop / --no-capture-file-drop into the actual
    // boolean to apply. The `run` command has no manifest / env-var
    // override layer, so a tri-state `None` collapses straight to `false`
    // (the documented code default).
    let capture_file_drop = resolve_capture_file_drop(&args).unwrap_or(false);

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

    // Resolve the embedding target before the window is created: tao applies
    // parent/owner at build time, not afterwards.
    let targets = ParentTargets::resolve(args.parent_hwnd.as_deref(), args.owner_hwnd.as_deref());

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

    // Embed in (or attach to) a foreign window when the host asked for it.
    let window_builder = parent_window::apply(window_builder, &targets);

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

    // Construct the drag-drop sink once and share it across both branches
    // (URL / HTML). With `capture=false` the helper does not clone the
    // Arc; with `capture=true` it clones exactly once.
    let drag_drop_sink: Arc<RunDragDropSink> = Arc::new(RunDragDropSink);

    // Load HTML content or URL
    let webview = if let Some(html) = html_content {
        tracing::info!("[CLI] Loading HTML content via with_html()");
        let builder = webview_builder.with_html(html);
        auroraview_core::builder::attach_drag_drop_handler(
            builder,
            capture_file_drop,
            &drag_drop_sink,
        )
        .build(&window)
        .context("Failed to create WebView with HTML content")?
    } else if let Some(url_str) = &args.url {
        let url = normalize_url(url_str)?;
        tracing::info!("[CLI] Loading URL: {}", url);
        let builder = webview_builder.with_url(&url);
        auroraview_core::builder::attach_drag_drop_handler(
            builder,
            capture_file_drop,
            &drag_drop_sink,
        )
        .build(&window)
        .context("Failed to create WebView with URL")?
    } else {
        anyhow::bail!("Either html_content or url must be set");
    };

    tracing::info!("WebView created successfully");

    // Start hot reload watcher (--watch):
    //   --html: filesystem watcher via notify
    //   --url:  polling thread that detects server restart
    let url_watcher_stop = Arc::new(AtomicBool::new(false));
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
        } else if let Some(ref url_str) = args.url {
            let normalized = normalize_url(url_str).unwrap_or_else(|_| url_str.clone());
            let poll_ms = args.poll_interval_ms.max(200); // min 200 ms
            start_url_watcher(
                normalized,
                proxy,
                Duration::from_millis(poll_ms),
                Arc::clone(&url_watcher_stop),
            );
            tracing::info!("[hot-reload] URL watcher started (poll: {}ms)", poll_ms);
            None
        } else {
            None
        }
    } else {
        None
    };

    // The html_path is captured for reload; clone it outside the closure
    let html_path_for_reload = args.html.clone();

    // Child mode: connect back to the parent host over the loopback IPC
    // channel. This is the Rust half of `AURORAVIEW_PARENT_*` and needs no
    // Python on either side of the socket.
    let child_info = ChildInfo::from_env();
    let bridge = if child_info.can_connect() {
        match ParentBridge::connect(&child_info) {
            Ok(bridge) => {
                tracing::info!(
                    "[CLI] Child mode: parent={:?} child={:?} port={:?}",
                    child_info.parent_id,
                    child_info.child_id,
                    child_info.parent_port
                );
                let proxy = event_loop.create_proxy();

                // wry's WebView is !Sync, so parent commands are forwarded to
                // the event loop instead of being applied on the reader thread.
                // The handle is kept alive for the lifetime of the run.
                let _command_handle =
                    bridge.on_command(move |data| match resolve_parent_command(&data) {
                        Some(event) => {
                            if proxy.send_event(event).is_err() {
                                tracing::debug!("[CLI] Event loop gone, dropping parent command");
                            }
                        }
                        None => tracing::warn!("[CLI] Unsupported parent command: {}", data),
                    });

                if args.exit_on_parent_disconnect {
                    let proxy = event_loop.create_proxy();
                    bridge.on_disconnect(move || {
                        tracing::info!("[CLI] Parent disconnected, exiting");
                        let _ = proxy.send_event(RunEvent::Close);
                    });
                }

                Some(bridge)
            }
            Err(e) => {
                tracing::warn!("[CLI] Standalone: parent IPC unavailable ({})", e);
                None
            }
        }
    } else {
        None
    };

    // Show window immediately
    window.set_visible(true);
    tracing::info!("Window shown");

    // Report the handshake outcome without blocking window startup: a
    // pre-protocol parent never sends `hello_ack`, and waiting for one would
    // add its timeout to every child's start-up time.
    if let Some(ref bridge) = bridge {
        let bridge = bridge.clone();
        std::thread::Builder::new()
            .name("parent-ipc-handshake".into())
            .spawn(move || {
                let state = bridge.wait_for_handshake(Duration::from_secs(2));
                tracing::info!("[CLI] Parent IPC handshake: {:?}", state);
            })
            .ok();
    }

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
                shutdown(&url_watcher_stop, bridge.as_ref());
                *control_flow = ControlFlow::Exit;
            }
            tao::event::Event::UserEvent(RunEvent::Eval(js)) => {
                tracing::debug!("[IPC] Evaluating script from parent");
                if let Err(e) = webview.evaluate_script(&js) {
                    tracing::warn!("[IPC] evaluate_script failed: {}", e);
                }
            }
            tao::event::Event::UserEvent(RunEvent::Emit(event, data)) => {
                tracing::debug!("[IPC] Emitting '{}' from parent", event);
                if let Err(e) = webview.evaluate_script(&emit_script(&event, &data)) {
                    tracing::warn!("[IPC] emit failed: {}", e);
                }
            }
            tao::event::Event::UserEvent(RunEvent::Close) => {
                tracing::info!("[IPC] Close requested by parent");
                shutdown(&url_watcher_stop, bridge.as_ref());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_url_basic() {
        let result = parse_http_url("http://localhost:3000/");
        assert_eq!(result, Some(("localhost".into(), 3000, "/".into())));
    }

    #[test]
    fn test_parse_http_url_https_default_port() {
        let result = parse_http_url("https://example.com");
        assert_eq!(result, Some(("example.com".into(), 443, "/".into())));
    }

    #[test]
    fn test_parse_http_url_http_default_port() {
        let result = parse_http_url("http://example.com/path");
        assert_eq!(result, Some(("example.com".into(), 80, "/path".into())));
    }

    #[test]
    fn test_parse_http_url_explicit_port_https() {
        let result = parse_http_url("https://example.com:8443/api");
        assert_eq!(result, Some(("example.com".into(), 8443, "/api".into())));
    }

    #[test]
    fn test_parse_http_url_non_http_scheme() {
        assert_eq!(parse_http_url("file:///path/to/file"), None);
        assert_eq!(parse_http_url("auroraview://app/index.html"), None);
        assert_eq!(parse_http_url("ws://localhost:8080"), None);
    }

    #[test]
    fn test_parse_http_url_empty() {
        assert_eq!(parse_http_url(""), None);
    }

    #[test]
    fn test_probe_unreachable_address() {
        // Port 1 is almost certainly not in use.
        assert!(!probe_url_reachable("http://127.0.0.1:1/"));
    }
}
