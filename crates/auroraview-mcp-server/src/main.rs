//! AuroraView MCP Sidecar Server - CLI Entry Point
//!
//! This is the main entry point for the MCP Sidecar process.
//! It handles CLI arguments, connects to the main process via IPC,
//! and starts the MCP HTTP server.
//!
//! ## Lifecycle Binding
//!
//! By default, the sidecar is strictly bound to the parent process:
//! - If `--parent-pid` is provided, it monitors that process
//! - If not provided, it auto-detects the parent PID
//! - When the parent dies, the sidecar automatically exits
//!
//! To run as a standalone service (not bound to parent), use `--daemon` flag.

use anyhow::{Context, Result};
use auroraview_mcp_server::{HttpServer, HttpServerConfig, IpcClient, ParentMonitor};
use clap::Parser;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

/// AuroraView MCP Sidecar Server
///
/// A standalone MCP server that communicates with the main AuroraView
/// process via IPC and exposes tools to AI agents.
#[derive(Parser, Debug)]
#[command(name = "auroraview-mcp-server")]
#[command(about = "AuroraView MCP Sidecar Server")]
#[command(version)]
struct Args {
    /// MCP server port (0 for auto-assign)
    #[arg(short, long, default_value = "0")]
    port: u16,

    /// IPC channel name for connecting to main process
    #[arg(long)]
    ipc: String,

    /// Authentication token for IPC handshake
    #[arg(long)]
    token: String,

    /// Parent process PID to monitor (auto-detected if not specified)
    ///
    /// When the parent process exits, this sidecar will automatically exit.
    /// Use --daemon to disable this behavior.
    #[arg(long)]
    parent_pid: Option<u32>,

    /// Run as a standalone daemon (do not exit when parent dies)
    ///
    /// By default, the sidecar is strictly bound to its parent process
    /// and will exit when the parent exits. Use this flag to run as
    /// a persistent service.
    #[arg(long, default_value = "false")]
    daemon: bool,

    /// Parent process check interval in milliseconds
    #[arg(long, default_value = "500")]
    check_interval_ms: u64,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(&args.log_level)?;

    tracing::info!("AuroraView MCP Sidecar starting...");
    tracing::info!("  IPC channel: {}", args.ipc);
    tracing::info!("  MCP port: {}", args.port);
    tracing::info!("  Daemon mode: {}", args.daemon);

    // Build async runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Failed to create Tokio runtime")?;

    runtime.block_on(async_main(args))
}

async fn async_main(args: Args) -> Result<()> {
    // Determine parent PID for monitoring
    // Priority: CLI arg > auto-detect > no monitoring (daemon mode only)
    let parent_pid = if args.daemon {
        tracing::info!("Running in daemon mode - parent process monitoring disabled");
        None
    } else {
        let pid = args.parent_pid.or_else(|| {
            // Auto-detect parent PID
            let detected = get_parent_pid();
            if let Some(p) = detected {
                tracing::info!("Auto-detected parent PID: {}", p);
            }
            detected
        });

        if pid.is_none() {
            tracing::warn!(
                "No parent PID specified or detected. \
                Use --parent-pid to specify, or --daemon to run as standalone service."
            );
        }
        pid
    };

    // Start parent process monitor
    let parent_monitor = parent_pid.map(|pid| {
        tracing::info!(
            "Parent process binding enabled: PID {} (check interval: {}ms)",
            pid,
            args.check_interval_ms
        );
        ParentMonitor::new(pid)
            .with_interval(Duration::from_millis(args.check_interval_ms))
            .start()
    });

    // Set up shutdown signal
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        tracing::info!("Received shutdown signal");
        running_clone.store(false, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl+C handler")?;

    // Connect to main process via IPC
    tracing::info!("Connecting to main process via IPC...");
    let ipc_client = Arc::new(IpcClient::new(&args.ipc, &args.token));

    if let Err(e) = ipc_client.connect() {
        tracing::error!("Failed to connect to main process: {}", e);
        return Err(anyhow::anyhow!("IPC connection failed: {}", e));
    }

    tracing::info!("Connected to main process");

    // Get tool list from main process
    let tools = ipc_client.get_tool_list().unwrap_or_else(|e| {
        tracing::warn!("Failed to get tool list: {}", e);
        vec![]
    });

    tracing::info!("Loaded {} tools from main process", tools.len());
    for tool in &tools {
        tracing::debug!("  - {}", tool.name);
    }

    // Create HTTP server config
    let http_config = HttpServerConfig {
        host: "127.0.0.1".to_string(),
        port: args.port,
        name: "auroraview-mcp-sidecar".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        heartbeat_interval: 30,
    };

    // Start MCP HTTP server
    let http_server = Arc::new(HttpServer::new(http_config, Arc::clone(&ipc_client)));
    let actual_port = http_server.start().await.map_err(|e| anyhow::anyhow!(e))?;

    tracing::info!("MCP Sidecar ready on port {}", actual_port);

    // Notify main process that we're ready with the actual port
    let _ = ipc_client.notify("lifecycle.ready", serde_json::json!({ "port": actual_port }));

    // IPC health check interval (less frequent than parent check)
    let ipc_check_interval = Duration::from_secs(5);
    let mut last_ipc_check = std::time::Instant::now();

    // Main loop: check for shutdown conditions
    while running.load(Ordering::SeqCst) {
        // Check 1: Parent process is still alive
        if let Some(ref monitor) = parent_monitor {
            if !monitor.is_parent_alive() {
                tracing::info!("Parent process exited, shutting down...");
                break;
            }
        }

        // Check 2: IPC connection is still alive (periodic health check)
        if last_ipc_check.elapsed() >= ipc_check_interval {
            if !ipc_client.is_connected() {
                tracing::info!("IPC connection lost, shutting down...");
                break;
            }
            // Optional: perform active health check
            // Disabled by default as it may interfere with normal operation
            // if !ipc_client.health_check() {
            //     tracing::info!("IPC health check failed, shutting down...");
            //     break;
            // }
            last_ipc_check = std::time::Instant::now();
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Cleanup
    tracing::info!("Shutting down...");

    // Stop HTTP server
    http_server.stop().await;

    if let Some(monitor) = parent_monitor {
        monitor.stop();
    }

    // Notify main process that we're shutting down
    let _ = ipc_client.notify("lifecycle.bye", serde_json::json!({}));

    tracing::info!("Goodbye!");
    Ok(())
}

fn init_logging(level: &str) -> Result<()> {
    let filter = EnvFilter::try_new(level)
        .or_else(|_| EnvFilter::try_new("info"))
        .context("Failed to parse log level")?;

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .init();

    Ok(())
}

/// Get the parent process ID
#[cfg(windows)]
fn get_parent_pid() -> Option<u32> {
    use std::mem::MaybeUninit;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
        TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let current_pid = std::process::id();

        let mut entry = MaybeUninit::<PROCESSENTRY32>::zeroed();
        (*entry.as_mut_ptr()).dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snapshot, entry.as_mut_ptr()).is_ok() {
            loop {
                let entry_ref = entry.assume_init_ref();
                if entry_ref.th32ProcessID == current_pid {
                    let parent_pid = entry_ref.th32ParentProcessID;
                    let _ = CloseHandle(snapshot);
                    return Some(parent_pid);
                }
                if Process32Next(snapshot, entry.as_mut_ptr()).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
        None
    }
}

#[cfg(unix)]
fn get_parent_pid() -> Option<u32> {
    use nix::unistd::getppid;
    Some(getppid().as_raw() as u32)
}

#[cfg(not(any(windows, unix)))]
fn get_parent_pid() -> Option<u32> {
    tracing::warn!("Platform not supported for parent PID detection");
    None
}
