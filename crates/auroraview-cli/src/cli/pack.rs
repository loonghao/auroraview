//! Pack command - Package applications into standalone executables

use anyhow::{Context, Result};
use auroraview_pack::progress::{PackProgress, ProgressExt};
use clap::Parser;
use std::path::{Path, PathBuf};

/// Arguments for the 'pack' subcommand
#[derive(Parser, Debug)]
pub struct PackArgs {
    /// Path to manifest file (auroraview.pack.toml)
    /// If provided, other options will override manifest values
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// URL to pack into standalone app (e.g., www.baidu.com)
    #[arg(long, conflicts_with_all = ["frontend", "backend"])]
    pub url: Option<String>,

    /// Frontend directory or HTML file to embed
    #[arg(long)]
    pub frontend: Option<PathBuf>,

    /// Python backend entry point (e.g., "myapp.main:run")
    /// Requires PyOxidizer for Python runtime embedding
    #[arg(long, requires = "frontend")]
    pub backend: Option<String>,

    /// Output executable name (without extension)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Output directory
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Window title for the packed app
    #[arg(short, long)]
    pub title: Option<String>,

    /// Window width
    #[arg(long)]
    pub width: Option<u32>,

    /// Window height
    #[arg(long)]
    pub height: Option<u32>,

    /// Enable debug mode in packed app
    #[arg(short, long)]
    pub debug: bool,

    /// Build the generated project after generation
    #[arg(long)]
    pub build: bool,

    /// Make window frameless (no title bar)
    #[arg(long)]
    pub frameless: bool,

    /// Make window always on top
    #[arg(long)]
    pub always_on_top: bool,

    /// Disable window resizing
    #[arg(long)]
    pub no_resize: bool,

    /// Custom user agent string
    #[arg(long)]
    pub user_agent: Option<String>,

    /// Clean up build directory after successful build (only with --build)
    #[arg(long)]
    pub clean: bool,

    /// Path to custom icon file (.ico on Windows)
    /// Overrides the icon specified in manifest
    #[arg(long)]
    pub icon: Option<PathBuf>,

    /// Show console window when running the packed app (Windows only)
    /// By default, console is hidden for GUI applications
    #[arg(long)]
    pub console: bool,

    /// Hide console window (default behavior, can be used to override manifest)
    #[arg(long, conflicts_with = "console")]
    pub no_console: bool,
}

/// Run the pack command
pub fn run_pack(args: PackArgs) -> Result<()> {
    use auroraview_pack::{Manifest, PackConfig, PackManager};

    let progress = PackProgress::new();
    let spinner = progress.spinner("Loading configuration...");

    // Track manifest and base_dir for running before_build commands
    let mut manifest_opt: Option<Manifest> = None;
    let mut base_dir_opt: Option<PathBuf> = None;

    // Create pack configuration - either from manifest or CLI args
    let mut config = if let Some(config_path) = &args.config {
        // Load from manifest file
        let manifest_path = if config_path.is_absolute() {
            config_path.clone()
        } else {
            std::env::current_dir()?.join(config_path)
        };

        if !manifest_path.exists() {
            spinner.finish_error(&format!(
                "Config file not found: {}",
                manifest_path.display()
            ));
            anyhow::bail!("Config file not found: {}", manifest_path.display());
        }

        let manifest = Manifest::from_file(&manifest_path)
            .with_context(|| format!("Failed to load manifest: {}", manifest_path.display()))?;

        manifest
            .validate()
            .with_context(|| "Invalid manifest configuration")?;

        let base_dir = manifest_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();
        spinner.finish_success(&format!(
            "Loaded manifest from: {}",
            manifest_path.display()
        ));

        let config = PackConfig::from_manifest(&manifest, &base_dir)?;
        manifest_opt = Some(manifest);
        base_dir_opt = Some(base_dir);
        config
    } else if let Some(url) = args.url {
        spinner.finish_success(&format!("Packing URL: {}", url));
        PackConfig::url(url)
    } else if let Some(frontend) = args.frontend {
        if let Some(backend) = args.backend {
            spinner.finish_success("Packing fullstack application");
            PackConfig::fullstack(frontend, backend)
        } else {
            spinner.finish_success("Packing frontend application");
            PackConfig::frontend(frontend)
        }
    } else {
        // Try to find manifest in current directory
        if let Some(manifest_path) = Manifest::find_in_dir(".") {
            let manifest = Manifest::from_file(&manifest_path)
                .with_context(|| format!("Failed to load manifest: {}", manifest_path.display()))?;

            manifest
                .validate()
                .with_context(|| "Invalid manifest configuration")?;

            let base_dir = manifest_path
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf();
            spinner.finish_success(&format!("Found manifest at: {}", manifest_path.display()));

            let config = PackConfig::from_manifest(&manifest, &base_dir)?;
            manifest_opt = Some(manifest);
            base_dir_opt = Some(base_dir);
            config
        } else {
            spinner.finish_error("No configuration provided");
            anyhow::bail!(
                "No configuration provided.\n\n\
                Options:\n  \
                1. Use --config to specify a manifest file:\n     \
                   auroraview pack --config auroraview.pack.toml --build\n\n  \
                2. Use --url to wrap a website:\n     \
                   auroraview pack --url www.baidu.com --output my-app\n\n  \
                3. Use --frontend to bundle local assets:\n     \
                   auroraview pack --frontend ./dist --output my-app\n\n  \
                4. Create an auroraview.pack.toml in the current directory"
            );
        }
    };

    // Run before commands if manifest has them
    if let (Some(ref manifest), Some(ref base_dir)) = (&manifest_opt, &base_dir_opt) {
        if !manifest.build.before.is_empty() {
            let build_spinner = progress.spinner("Running before build commands...");
            for cmd in &manifest.build.before {
                tracing::info!("Running before build: {}", cmd);

                // Run command in the base_dir (manifest directory)
                let status = std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" })
                    .args(if cfg!(windows) {
                        vec!["/C", cmd]
                    } else {
                        vec!["-c", cmd]
                    })
                    .current_dir(base_dir)
                    .status()
                    .with_context(|| format!("Failed to run before build command: {}", cmd))?;

                if !status.success() {
                    build_spinner.finish_error(&format!("before_build command failed: {}", cmd));
                    anyhow::bail!("before_build command failed: {}", cmd);
                }
            }
            build_spinner.finish_success("before_build commands completed");
        }
    }

    // Override with CLI arguments if provided
    if let Some(output) = args.output {
        config = config.with_output(&output);
    }
    if let Some(title) = args.title {
        config = config.with_title(&title);
    }
    if let Some(width) = args.width {
        config.window.width = width;
    }
    if let Some(height) = args.height {
        config.window.height = height;
    }
    if args.debug {
        config = config.with_debug(true);
    }
    if args.frameless {
        config = config.with_frameless(true);
    }
    if args.always_on_top {
        config = config.with_always_on_top(true);
    }
    if args.no_resize {
        config = config.with_resizable(false);
    }
    if let Some(user_agent) = args.user_agent {
        config = config.with_user_agent(user_agent);
    }
    if let Some(output_dir) = args.output_dir {
        config.output_dir = output_dir;
    }

    // Apply Windows resource overrides from CLI
    if let Some(icon) = args.icon {
        let icon_path = if icon.is_absolute() {
            icon
        } else {
            std::env::current_dir()?.join(icon)
        };
        config.windows_resource.icon = Some(icon_path);
    }
    if args.console {
        config.windows_resource.console = true;
    }
    if args.no_console {
        config.windows_resource.console = false;
    }

    let output_dir = config.output_dir.clone();
    let output_name = config.output_name.clone();

    // Pack the application with progress
    let pack_spinner = progress.spinner("Packing application...");
    let manager = PackManager::new();
    let output = match manager.pack(&config) {
        Ok(o) => {
            pack_spinner.finish_success("Pack completed");
            o
        }
        Err(e) => {
            pack_spinner.finish_error(&format!("Pack failed: {}", e));
            return Err(e.into());
        }
    };

    // Display success message based on mode
    let size_mb = output.size as f64 / (1024.0 * 1024.0);

    println!();
    progress.success("Pack completed successfully!");
    println!();
    println!("  Mode: {}", output.mode);
    println!("  Output: {}", output.executable.display());
    println!("  Size: {:.2} MB", size_mb);

    if output.asset_count > 0 {
        println!("  Assets: {} files", output.asset_count);
    }
    if output.python_file_count > 0 {
        println!("  Python: {} files", output.python_file_count);
    }

    // For portable/system modes, show additional info
    if output.mode.contains("portable") || output.mode.contains("system") {
        let _app_dir = output_dir.join(&output_name);
        println!();
        println!("  The application directory contains:");
        println!(
            "    - {} (launcher)",
            output
                .executable
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        );
        println!("    - frontend/ (web assets)");
        println!("    - backend/ (Python code)");
        if output.mode.contains("portable") {
            println!("    - lib/ (Python packages)");
        } else {
            println!("    - requirements.txt (install with: pip install -r requirements.txt)");
        }
        println!();
        println!("  Run the application:");
        println!("    {}", output.executable.display());
    } else {
        println!();
        println!("  Run the application:");
        println!("    {}", output.executable.display());
    }

    Ok(())
}
