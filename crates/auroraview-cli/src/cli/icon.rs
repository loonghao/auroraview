//! Icon command - Icon utilities (compress, convert to ICO)

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

/// Arguments for the 'icon' subcommand
#[derive(Parser, Debug)]
pub struct IconArgs {
    #[command(subcommand)]
    pub command: IconCommands,
}

#[derive(Subcommand, Debug)]
pub enum IconCommands {
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

/// Run the icon command
pub fn run_icon(args: IconArgs) -> Result<()> {
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


