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
//! # Pack mode - Package frontend + Python backend
//! auroraview pack --frontend ./dist --backend "myapp:main" --output my-app
//!
//! # Show help
//! auroraview --help
//! ```

use anyhow::Result;
use auroraview_pack::is_packed;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use auroraview_cli::cli::{run_icon, run_info, run_pack, run_webview, IconArgs, PackArgs, RunArgs};
use auroraview_cli::packed;

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

fn main() -> Result<()> {
    // Check if this is a packed executable first
    if is_packed() {
        return packed::run_packed_app();
    }

    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.debug);

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

/// Initialize logging with appropriate level and local time
fn init_logging(debug: bool) {
    let log_level = if debug { "debug" } else { "info" };

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
}
