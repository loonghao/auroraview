//! Inspect command - Examine packed executable's overlay data
//!
//! This command reads the overlay data from a packed AuroraView executable
//! and displays the assets and configuration for debugging purposes.

use anyhow::{Context, Result};
use auroraview_pack::{OverlayReader, PackMode};
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct InspectArgs {
    /// Path to the packed executable to inspect
    #[arg(value_name = "PACKED_EXE")]
    pub exe_path: PathBuf,

    /// Show full asset content (for small assets only)
    #[arg(long, default_value = "false")]
    pub show_content: bool,

    /// Filter assets by pattern (e.g., "*.html", "index*")
    #[arg(long)]
    pub filter: Option<String>,
}

/// Run the inspect command
pub fn run_inspect(args: InspectArgs) -> Result<()> {
    let exe_path = &args.exe_path;

    if !exe_path.exists() {
        anyhow::bail!("File not found: {}", exe_path.display());
    }

    println!("Inspecting: {}", exe_path.display());
    println!("{}", "=".repeat(60));

    // Read overlay data
    let overlay = OverlayReader::read(exe_path)
        .with_context(|| format!("Failed to read overlay from: {}", exe_path.display()))?
        .ok_or_else(|| anyhow::anyhow!("No overlay data found in: {}", exe_path.display()))?;

    // Display configuration
    println!("\n[Configuration]");
    println!("  Window Title: {}", overlay.config.window.title);
    println!("  Mode: {:?}", overlay.config.mode);
    println!("  Debug: {}", overlay.config.debug);
    println!(
        "  Window Size: {}x{}",
        overlay.config.window.width, overlay.config.window.height
    );

    // Display Python backend info if FullStack mode
    if let PackMode::FullStack { ref python, .. } = overlay.config.mode {
        println!("\n[Python Backend]");
        println!("  Entry Point: {:?}", python.entry_point);
        println!("  Include Paths: {:?}", python.include_paths);
        println!("  Strategy: {:?}", python.strategy);
    }

    // Display assets
    println!("\n[Assets] ({} total)", overlay.assets.len());

    let mut assets: Vec<_> = overlay.assets.iter().collect();
    assets.sort_by(|a, b| a.0.cmp(&b.0));

    // Filter if pattern provided
    let filter_pattern = args.filter.as_ref().and_then(|f| {
        let pattern = f.replace('*', ".*");
        regex::Regex::new(&pattern).ok()
    });

    let mut shown_count = 0;
    for (path, content) in &assets {
        // Apply filter
        if let Some(ref pattern) = filter_pattern {
            if !pattern.is_match(path) {
                continue;
            }
        }

        let size = content.len();
        let size_str = if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.2} KB", size as f64 / 1024.0)
        } else {
            format!("{:.2} MB", size as f64 / 1024.0 / 1024.0)
        };

        // Check if it's the main index.html
        let marker = if *path == "index.html" || *path == "frontend/index.html" {
            " [MAIN]"
        } else {
            ""
        };

        println!("  {} ({}){}", path, size_str, marker);

        // Show content for small HTML/JS/CSS files if requested
        if args.show_content
            && size < 2000
            && (path.ends_with(".html") || path.ends_with(".js") || path.ends_with(".css"))
        {
            if let Ok(text) = std::str::from_utf8(content) {
                let preview = text.chars().take(500).collect::<String>();
                println!("    Content Preview:");
                for line in preview.lines().take(10) {
                    println!("      {}", line);
                }
                if text.len() > 500 {
                    println!("      ...");
                }
                println!();
            }
        }

        shown_count += 1;
    }

    if filter_pattern.is_some() {
        println!(
            "\n  Showing {} of {} assets (filtered)",
            shown_count,
            overlay.assets.len()
        );
    }

    // Check for common issues
    println!("\n[Diagnostics]");

    // Check for index.html
    let has_index = assets.iter().any(|(path, _)| {
        *path == "index.html" || *path == "frontend/index.html" || path.ends_with("/index.html")
    });

    if has_index {
        println!("  ✓ index.html found");
    } else {
        println!("  ✗ WARNING: No index.html found! This will cause a white screen.");
        println!("    Available HTML files:");
        for (path, _) in &assets {
            if path.ends_with(".html") {
                println!("      - {}", path);
            }
        }
    }

    // Check for loading HTML
    let has_loading = assets
        .iter()
        .any(|(path, _)| *path == "loading/index.html" || path.contains("loading"));

    if has_loading {
        println!("  ✓ Loading page assets found");
    } else {
        println!("  ✗ No loading page assets (will be generated dynamically)");
    }

    // Check for Python files if fullstack
    let has_python = assets
        .iter()
        .any(|(path, _)| path.ends_with(".py") || path.ends_with(".pyc"));
    if matches!(overlay.config.mode, PackMode::FullStack { .. }) {
        if has_python {
            println!("  ✓ Python files found for FullStack mode");
        } else {
            println!("  ⚠ FullStack mode configured but no Python files in assets");
            println!("    (Python files may be loaded from external paths)");
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Inspection complete.");

    Ok(())
}
