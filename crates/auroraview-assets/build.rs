//! Build script for auroraview-assets.
//!
//! Ensures that `frontend/dist/` exists at compile time so that the
//! `RustEmbed` derive macro never fails when the frontend bundle has not
//! been pre-built (for example on CI jobs that only check the Rust
//! workspace, or on a fresh clone before `just assets-build` has run).
//!
//! Real production builds always run `just assets-build` (or the Vite
//! pipeline) before `cargo build`, so the embedded assets will be the
//! genuine ones. This script only provides a safe fallback so that
//! `cargo check` / `cargo clippy` / `cargo test` keep working without a
//! Node.js toolchain.

use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo");
    let dist_dir = Path::new(&manifest_dir).join("frontend").join("dist");

    if !dist_dir.exists() {
        if let Err(err) = fs::create_dir_all(&dist_dir) {
            // Don't fail the build — just print a warning. RustEmbed will
            // surface a clearer error if the directory truly cannot be
            // created.
            println!(
                "cargo:warning=auroraview-assets: failed to create placeholder dist dir at {}: {}",
                dist_dir.display(),
                err
            );
            return;
        }

        // RustEmbed needs at least one file matching the include patterns
        // for the embed list to be non-empty. Create a tiny placeholder
        // so that the macro expansion succeeds.
        let placeholder = dist_dir.join(".gitkeep");
        if !placeholder.exists() {
            let _ = fs::write(&placeholder, b"# auroraview-assets placeholder\n");
        }

        println!(
            "cargo:warning=auroraview-assets: frontend/dist/ was missing; created an empty \
             placeholder. Run `just assets-build` to embed the real frontend bundle."
        );
    }

    // Re-run this build script when the dist directory layout changes.
    println!("cargo:rerun-if-changed=frontend/dist");
    println!("cargo:rerun-if-changed=build.rs");
}
