//! AuroraView CLI - Standalone WebView launcher and packager
//!
//! This CLI tool allows users to:
//! - Launch a WebView window from the command line
//! - Package applications into standalone executables
//!
//! ## Usage
//!
//! ```bash
//! # Run mode - Load a URL
//! auroraview run --url https://example.com
//!
//! # Run mode - Load a local HTML file
//! auroraview run --html /path/to/file.html
//!
//! # Legacy mode (backward compatible)
//! auroraview --url https://example.com
//!
//! # Pack mode - Package URL into standalone app
//! auroraview pack --url www.baidu.com --output my-app
//!
//! # Pack mode - Package frontend into standalone app
//! auroraview pack --frontend ./dist --output my-app
//!
//! # Pack mode - Package frontend + Python backend (requires PyOxidizer)
//! auroraview pack --frontend ./dist --backend "myapp:main" --output my-app
//!
//! # Show help
//! auroraview --help
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tao::event_loop::{ControlFlow, EventLoop};
use url::Url;
use wry::{WebContext, WebViewBuilder as WryWebViewBuilder};

mod protocol_handlers;

/// AuroraView - Standalone WebView launcher and packager
#[derive(Parser, Debug)]
#[command(
    name = "auroraview",
    version,
    about = "AuroraView - Rust-powered WebView launcher and packager",
    long_about = "Launch WebView windows or package applications into standalone executables.\n\n\
                  Use 'auroraview run' to launch a WebView, or 'auroraview pack' to create \
                  standalone executables similar to Pake."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    // Legacy arguments for backward compatibility (when no subcommand is used)
    /// URL to load in the WebView (legacy mode, use 'run --url' instead)
    #[arg(short, long, conflicts_with = "html", global = true)]
    url: Option<String>,

    /// Local HTML file to load (legacy mode, use 'run --html' instead)
    #[arg(short = 'f', long, conflicts_with = "url", global = true)]
    html: Option<PathBuf>,

    /// Assets root directory for local HTML files
    #[arg(long, requires = "html", global = true)]
    assets_root: Option<PathBuf>,

    /// Window title
    #[arg(short, long, default_value = "AuroraView", global = true)]
    title: String,

    /// Window width (set to 0 to maximize)
    #[arg(long, default_value = "800", global = true)]
    width: u32,

    /// Window height (set to 0 to maximize)
    #[arg(long, default_value = "600", global = true)]
    height: u32,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    debug: bool,

    /// Allow opening new windows
    #[arg(long, global = true)]
    allow_new_window: bool,

    /// Enable file:// protocol support
    #[arg(long, global = true)]
    allow_file_protocol: bool,

    /// Keep window always on top
    #[arg(long, global = true)]
    always_on_top: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a WebView window with a URL or local HTML file
    Run(RunArgs),

    /// Package an application into a standalone executable
    Pack(PackArgs),

    /// Icon utilities (compress, convert to ICO)
    Icon(IconArgs),

    /// Show version and environment information
    Info,
}

/// Arguments for the 'run' subcommand
#[derive(Parser, Debug)]
struct RunArgs {
    /// URL to load in the WebView
    #[arg(short, long, conflicts_with = "html")]
    url: Option<String>,

    /// Local HTML file to load in the WebView
    #[arg(short = 'f', long, conflicts_with = "url")]
    html: Option<PathBuf>,

    /// Assets root directory for local HTML files
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

    /// Allow opening new windows
    #[arg(long)]
    allow_new_window: bool,

    /// Enable file:// protocol support
    #[arg(long)]
    allow_file_protocol: bool,

    /// Keep window always on top
    #[arg(long)]
    always_on_top: bool,
}

/// Arguments for the 'pack' subcommand
#[derive(Parser, Debug)]
struct PackArgs {
    /// URL to pack into standalone app (e.g., www.baidu.com)
    #[arg(long, conflicts_with_all = ["frontend", "backend"])]
    url: Option<String>,

    /// Frontend directory or HTML file to embed
    #[arg(long)]
    frontend: Option<PathBuf>,

    /// Python backend entry point (e.g., "myapp.main:run")
    /// Requires PyOxidizer for Python runtime embedding
    #[arg(long, requires = "frontend")]
    backend: Option<String>,

    /// Output executable name (without extension)
    #[arg(short, long, default_value = "app")]
    output: String,

    /// Output directory
    #[arg(long, default_value = ".")]
    output_dir: PathBuf,

    /// Window title for the packed app
    #[arg(short, long, default_value = "AuroraView App")]
    title: String,

    /// Window width
    #[arg(long, default_value = "1024")]
    width: u32,

    /// Window height
    #[arg(long, default_value = "768")]
    height: u32,

    /// Enable debug mode in packed app
    #[arg(short, long)]
    debug: bool,

    /// Build the generated project after generation
    #[arg(long)]
    build: bool,

    /// Make window frameless (no title bar)
    #[arg(long)]
    frameless: bool,

    /// Make window always on top
    #[arg(long)]
    always_on_top: bool,

    /// Disable window resizing
    #[arg(long)]
    no_resize: bool,

    /// Custom user agent string
    #[arg(long)]
    user_agent: Option<String>,

    /// Clean up build directory after successful build (only with --build)
    #[arg(long)]
    clean: bool,
}

/// Arguments for the 'icon' subcommand
#[derive(Parser, Debug)]
struct IconArgs {
    #[command(subcommand)]
    command: IconCommands,
}

#[derive(Subcommand, Debug)]
enum IconCommands {
    /// Compress PNG image
    Compress {
        /// Input PNG file
        #[arg(short, long)]
        input: PathBuf,

        /// Output PNG file (defaults to input_compressed.png)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Compression level (1-9, higher = smaller file)
        #[arg(short, long, default_value = "9")]
        level: u8,

        /// Maximum size (width/height) for resizing
        #[arg(long)]
        max_size: Option<u32>,
    },

    /// Convert PNG to ICO format (for Windows EXE icons)
    ToIco {
        /// Input PNG file
        #[arg(short, long)]
        input: PathBuf,

        /// Output ICO file (defaults to input.ico)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Icon sizes to include (comma-separated, e.g., "16,32,48,256")
        #[arg(short, long, default_value = "16,32,48,256")]
        sizes: String,
    },

    /// Generate all icon assets from a source PNG
    Generate {
        /// Input PNG file (should be high-resolution, e.g., 512x512 or larger)
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory for generated icons
        #[arg(short, long, default_value = ".")]
        output_dir: PathBuf,

        /// Base name for output files (defaults to input filename)
        #[arg(short, long)]
        name: Option<String>,
    },
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
    let cli = Cli::parse();

    // Initialize logging with appropriate level and local time
    let log_level = if cli.debug { "debug" } else { "info" };

    // Try to use local time, fallback to UTC if local offset cannot be determined
    let local_time = tracing_subscriber::fmt::time::OffsetTime::local_rfc_3339();

    match local_time {
        Ok(timer) => {
            tracing_subscriber::fmt()
                .with_timer(timer)
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
                )
                .with_target(false)
                .init();
        }
        Err(_) => {
            // Fallback to default timer (UTC) if local time is not available
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
                )
                .with_target(false)
                .init();
        }
    }

    // Handle commands
    match cli.command {
        Some(Commands::Run(args)) => run_webview(args),
        Some(Commands::Pack(args)) => run_pack(args),
        Some(Commands::Icon(args)) => run_icon(args),
        Some(Commands::Info) => run_info(),
        None => {
            // Legacy mode: use top-level args
            let args = RunArgs {
                url: cli.url,
                html: cli.html,
                assets_root: cli.assets_root,
                title: cli.title,
                width: cli.width,
                height: cli.height,
                debug: cli.debug,
                allow_new_window: cli.allow_new_window,
                allow_file_protocol: cli.allow_file_protocol,
                always_on_top: cli.always_on_top,
            };
            run_webview(args)
        }
    }
}

/// Run the WebView window
fn run_webview(args: RunArgs) -> Result<()> {
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
    // On Windows WebView2, NewWindowResponse::Allow may not work reliably.
    // Instead, we open external links in the system default browser.
    if args.allow_new_window {
        tracing::info!("[CLI] Allowing new windows (opens in system browser)");
        webview_builder = webview_builder.with_new_window_req_handler(|url, _features| {
            tracing::info!("[CLI] New window requested: {}", url);
            // Open in system default browser instead of creating a new WebView window
            if let Err(e) = open::that(&url) {
                tracing::error!("[CLI] Failed to open URL in browser: {}", e);
            }
            // Deny creating a new WebView window since we opened in external browser
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

/// Display environment and version information
fn run_info() -> Result<()> {
    println!("🌟 AuroraView CLI Information\n");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Rust Version: {}", rustc_version());
    println!();

    // Check for required tools
    println!("📦 Dependencies:");

    // Check cargo
    let cargo_ok = std::process::Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    println!(
        "  Cargo: {}",
        if cargo_ok {
            "✅ Available"
        } else {
            "❌ Not found"
        }
    );

    // Check PyOxidizer
    let pyoxidizer_ok = std::process::Command::new("pyoxidizer")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    println!(
        "  PyOxidizer: {} (required for fullstack mode)",
        if pyoxidizer_ok {
            "✅ Available"
        } else {
            "⚠️ Not found"
        }
    );

    println!();
    println!("🎯 Available Commands:");
    println!("  run   - Launch a WebView window");
    println!("  pack  - Package an application into a standalone executable");
    println!("  info  - Show this information");
    println!();
    println!("📖 Examples:");
    println!("  auroraview run --url https://example.com");
    println!("  auroraview run --html ./index.html");
    println!("  auroraview pack --url www.baidu.com --output my-app --build");
    println!("  auroraview pack --frontend ./dist --output my-app");
    println!();

    Ok(())
}

/// Get rustc version (simplified)
fn rustc_version() -> String {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string()
}

/// Run the pack command
fn run_pack(args: PackArgs) -> Result<()> {
    use auroraview_pack::{PackConfig, PackGenerator};

    tracing::info!("Starting pack operation...");

    // Create pack configuration based on mode
    let mut config = if let Some(url) = args.url {
        PackConfig::url(url)
    } else if let Some(frontend) = args.frontend {
        if let Some(backend) = args.backend {
            PackConfig::fullstack(frontend, backend)
        } else {
            PackConfig::frontend(frontend)
        }
    } else {
        anyhow::bail!(
            "Either --url or --frontend must be provided.\n\
            Examples:\n  \
            auroraview pack --url www.baidu.com --output my-app\n  \
            auroraview pack --frontend ./dist --output my-app"
        );
    };

    // Apply additional configuration
    config = config
        .with_output(&args.output)
        .with_title(&args.title)
        .with_size(args.width, args.height)
        .with_debug(args.debug)
        .with_frameless(args.frameless)
        .with_always_on_top(args.always_on_top)
        .with_resizable(!args.no_resize);

    // Apply optional user agent
    if let Some(user_agent) = args.user_agent {
        config = config.with_user_agent(user_agent);
    }

    let output_dir = args.output_dir.clone();
    config.output_dir = args.output_dir;

    // Generate the pack project
    let generator = PackGenerator::new(config);
    let project_dir = generator.generate()?;

    tracing::info!("Pack project generated at: {}", project_dir.display());

    // Detect if this is a fullstack project (has pyoxidizer.bzl)
    let is_fullstack = project_dir.join("pyoxidizer.bzl").exists();

    // Build if requested
    if args.build {
        if is_fullstack {
            build_fullstack_project(&project_dir, &args.output, &output_dir, args.clean)?;
        } else {
            build_pack_project(&project_dir, &args.output, &output_dir, args.clean)?;
        }
    } else if is_fullstack {
        println!(
            "\n✨ Pack project generated successfully!\n\n\
            To build the executable with embedded Python:\n  \
            cd {}\n  \
            pyoxidizer build --release\n\n\
            The executable will be at:\n  \
            {}/build/*/release/install/\n\n\
            Or use --build flag to build automatically:\n  \
            auroraview pack ... --build",
            project_dir.display(),
            project_dir.display()
        );
    } else {
        println!(
            "\n✨ Pack project generated successfully!\n\n\
            To build the executable:\n  \
            cd {}\n  \
            cargo build --release\n\n\
            The executable will be at:\n  \
            {}/target/release/{}.exe\n\n\
            Or use --build flag to build automatically:\n  \
            auroraview pack ... --build",
            project_dir.display(),
            project_dir.display(),
            args.output
        );
    }

    Ok(())
}

/// Build a fullstack project with embedded Python using PyOxidizer
fn build_fullstack_project(
    project_dir: &std::path::Path,
    output_name: &str,
    output_dir: &std::path::Path,
    clean: bool,
) -> anyhow::Result<()> {
    use std::fs;

    println!("\n🔨 Building fullstack project with PyOxidizer...\n");

    // Check if pyoxidizer is available
    let pyoxidizer_version = std::process::Command::new("pyoxidizer")
        .arg("--version")
        .output();

    if pyoxidizer_version.is_err() {
        anyhow::bail!(
            "PyOxidizer not found. Please install it:\n\
            cargo install pyoxidizer\n\n\
            Or from the maintained fork:\n\
            cargo install --git https://github.com/loonghao/PyOxidizer --branch auroraview-maintained pyoxidizer\n\n\
            Manual build:\n  \
            cd {}\n  \
            pyoxidizer build --release",
            project_dir.display()
        );
    }

    // Run pyoxidizer build
    let status = std::process::Command::new("pyoxidizer")
        .args(["build", "--release"])
        .current_dir(project_dir)
        .status()
        .context("Failed to run pyoxidizer build")?;

    if !status.success() {
        anyhow::bail!("PyOxidizer build failed with status: {}", status);
    }

    // PyOxidizer outputs to build/<target>/release/install/
    // Find the output directory
    let build_dir = project_dir.join("build");
    if !build_dir.exists() {
        anyhow::bail!(
            "PyOxidizer build directory not found at: {}",
            build_dir.display()
        );
    }

    // Find the target triple directory (e.g., x86_64-pc-windows-msvc)
    let target_dir = fs::read_dir(&build_dir)?
        .filter_map(|e| e.ok())
        .find(|e| e.path().is_dir())
        .map(|e| e.path())
        .ok_or_else(|| anyhow::anyhow!("No target directory found in build/"))?;

    let install_dir = target_dir.join("release").join("install");
    if !install_dir.exists() {
        anyhow::bail!(
            "PyOxidizer install directory not found at: {}",
            install_dir.display()
        );
    }

    // Determine executable name based on platform
    #[cfg(target_os = "windows")]
    let exe_name = format!("{}.exe", output_name);
    #[cfg(not(target_os = "windows"))]
    let exe_name = output_name.to_string();

    // Source executable path
    let src_exe = install_dir.join(&exe_name);

    if !src_exe.exists() {
        // Try to find any executable in the install directory
        let found_exe = fs::read_dir(&install_dir)?
            .filter_map(|e| e.ok())
            .find(|e| {
                let path = e.path();
                path.is_file() && path.extension().is_some_and(|ext| ext == "exe")
            })
            .map(|e| e.path());

        if let Some(found) = found_exe {
            // Copy the found executable with the expected name
            let dst_exe = output_dir.join(&exe_name);
            fs::copy(&found, &dst_exe).context("Failed to copy executable to output directory")?;

            let file_size = fs::metadata(&dst_exe)?.len();
            let size_mb = file_size as f64 / (1024.0 * 1024.0);

            println!("\n✅ Build completed successfully!\n");
            println!("📦 Executable: {}", dst_exe.display());
            println!("📊 Size: {:.2} MB", size_mb);
            if clean {
                println!("\n🧹 Cleaning up build directory...");
                if let Err(e) = fs::remove_dir_all(project_dir) {
                    tracing::warn!("Failed to clean up build directory: {}", e);
                } else {
                    println!("✓ Build directory removed: {}", project_dir.display());
                }
            } else {
                println!("\n💡 Tip: Use --clean flag to automatically remove the build directory.");
            }
            return Ok(());
        }

        anyhow::bail!("Built executable not found at: {}", src_exe.display());
    }

    // Destination executable path
    let dst_exe = output_dir.join(&exe_name);

    // Copy executable to output directory
    fs::copy(&src_exe, &dst_exe).context("Failed to copy executable to output directory")?;

    // Get file size
    let file_size = fs::metadata(&dst_exe)?.len();
    let size_mb = file_size as f64 / (1024.0 * 1024.0);

    println!("\n✅ Build completed successfully!\n");
    println!("📦 Executable: {}", dst_exe.display());
    println!("📊 Size: {:.2} MB", size_mb);

    // Clean up build directory if requested
    if clean {
        println!("\n🧹 Cleaning up build directory...");
        if let Err(e) = fs::remove_dir_all(project_dir) {
            tracing::warn!("Failed to clean up build directory: {}", e);
            println!("⚠️  Failed to clean up: {}", e);
        } else {
            println!("✓ Build directory removed: {}", project_dir.display());
        }
    } else {
        println!("\n💡 Tip: Use --clean flag to automatically remove the build directory.");
    }

    Ok(())
}

/// Build the pack project and copy the executable to the output directory
/// Build a simple pack project (URL or Frontend mode) using cargo
fn build_pack_project(
    project_dir: &std::path::Path,
    output_name: &str,
    output_dir: &std::path::Path,
    clean: bool,
) -> anyhow::Result<()> {
    use std::fs;

    println!("\n🔨 Building pack project...\n");

    // Check if cargo is available
    let cargo_version = std::process::Command::new("cargo")
        .arg("--version")
        .output();

    if cargo_version.is_err() {
        anyhow::bail!(
            "Cargo not found. Please install Rust toolchain:\n\
            https://rustup.rs/\n\n\
            Or build manually:\n  \
            cd {}\n  \
            cargo build --release",
            project_dir.display()
        );
    }

    // Run cargo build with progress output
    let status = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(project_dir)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("Build failed with status: {}", status);
    }

    // Determine executable name based on platform
    #[cfg(target_os = "windows")]
    let exe_name = format!("{}.exe", output_name);
    #[cfg(not(target_os = "windows"))]
    let exe_name = output_name.to_string();

    // Source executable path
    let src_exe = project_dir.join("target").join("release").join(&exe_name);

    if !src_exe.exists() {
        anyhow::bail!("Built executable not found at: {}", src_exe.display());
    }

    // Destination executable path
    let dst_exe = output_dir.join(&exe_name);

    // Copy executable to output directory
    fs::copy(&src_exe, &dst_exe).context("Failed to copy executable to output directory")?;

    // Get file size
    let file_size = fs::metadata(&dst_exe)?.len();
    let size_mb = file_size as f64 / (1024.0 * 1024.0);

    println!("\n✅ Build completed successfully!\n");
    println!("📦 Executable: {}", dst_exe.display());
    println!("📊 Size: {:.2} MB", size_mb);

    // Clean up build directory if requested
    if clean {
        println!("\n🧹 Cleaning up build directory...");
        if let Err(e) = fs::remove_dir_all(project_dir) {
            tracing::warn!("Failed to clean up build directory: {}", e);
            println!("⚠️  Failed to clean up: {}", e);
        } else {
            println!("✓ Build directory removed: {}", project_dir.display());
        }
    } else {
        println!("\n💡 Tip: Use --clean flag to automatically remove the build directory.");
    }

    Ok(())
}

/// Run the icon command
fn run_icon(args: IconArgs) -> Result<()> {
    use auroraview_core::icon::{compress_and_resize, compress_png, png_to_ico};

    match args.command {
        IconCommands::Compress {
            input,
            output,
            level,
            max_size,
        } => {
            if !input.exists() {
                anyhow::bail!("Input file not found: {}", input.display());
            }

            let output_path = output.unwrap_or_else(|| {
                let stem = input.file_stem().unwrap_or_default().to_string_lossy();
                let parent = input.parent().unwrap_or(Path::new("."));
                parent.join(format!("{}_compressed.png", stem))
            });

            let result = if let Some(max) = max_size {
                compress_and_resize(&input, &output_path, max, level)
                    .context("Failed to compress and resize PNG")?
            } else {
                compress_png(&input, &output_path, level).context("Failed to compress PNG")?
            };

            println!("\nPNG Compression Complete");
            println!("  Input:  {}", input.display());
            println!("  Output: {}", output_path.display());
            println!(
                "  Size:   {} -> {} ({:.1}% reduction)",
                format_bytes(result.original_size),
                format_bytes(result.compressed_size),
                result.reduction_percent()
            );
            if max_size.is_some() {
                println!("  Dimensions: {}x{}", result.width, result.height);
            }

            Ok(())
        }

        IconCommands::ToIco {
            input,
            output,
            sizes,
        } => {
            if !input.exists() {
                anyhow::bail!("Input file not found: {}", input.display());
            }

            let size_vec: Vec<u32> = sizes
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            if size_vec.is_empty() {
                anyhow::bail!(
                    "Invalid sizes format. Use comma-separated numbers, e.g., '16,32,48,256'"
                );
            }

            let output_path = output.unwrap_or_else(|| input.with_extension("ico"));

            png_to_ico(&input, &output_path, &size_vec).context("Failed to convert PNG to ICO")?;

            println!("\nICO Conversion Complete");
            println!("  Input:  {}", input.display());
            println!("  Output: {}", output_path.display());
            println!("  Sizes:  {:?}", size_vec);

            Ok(())
        }

        IconCommands::Generate {
            input,
            output_dir,
            name,
        } => {
            if !input.exists() {
                anyhow::bail!("Input file not found: {}", input.display());
            }

            std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

            let base_name = name.unwrap_or_else(|| {
                input
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });

            println!("\nGenerating icon assets from: {}", input.display());
            println!("Output directory: {}", output_dir.display());

            let ico_path = output_dir.join(format!("{}.ico", base_name));
            png_to_ico(&input, &ico_path, &[16, 32, 48, 256]).context("Failed to generate ICO")?;
            println!("  Created: {}", ico_path.display());

            let png_256_path = output_dir.join(format!("{}-256.png", base_name));
            compress_and_resize(&input, &png_256_path, 256, 9)
                .context("Failed to generate 256px PNG")?;
            println!("  Created: {}", png_256_path.display());

            let png_32_path = output_dir.join(format!("{}-32.png", base_name));
            compress_and_resize(&input, &png_32_path, 32, 9)
                .context("Failed to generate 32px PNG")?;
            println!("  Created: {}", png_32_path.display());

            let png_64_path = output_dir.join(format!("{}-64.png", base_name));
            compress_and_resize(&input, &png_64_path, 64, 9)
                .context("Failed to generate 64px PNG")?;
            println!("  Created: {}", png_64_path.display());

            println!("\nAll icon assets generated successfully!");

            Ok(())
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Get the WebView2 user data directory in AppData
///
/// Returns a path like: `%LOCALAPPDATA%/AuroraView/WebView2`
/// This prevents WebView2 from creating data folders in the current directory.
fn get_webview_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AuroraView")
        .join("WebView2")
}

/// Embedded window icon (32x32 PNG)
const ICON_PNG_BYTES: &[u8] = include_bytes!("../../../assets/icons/auroraview-32.png");

/// Load window icon from embedded PNG bytes
fn load_window_icon() -> Option<tao::window::Icon> {
    use ::image::GenericImageView;

    let img = ::image::load_from_memory(ICON_PNG_BYTES).ok()?;
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();

    tao::window::Icon::from_rgba(rgba, width, height).ok()
}
