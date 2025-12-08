//! Build script for AuroraView CLI
//!
//! Sets the Windows application icon for the compiled executable.

fn main() {
    // Only run on Windows
    #[cfg(target_os = "windows")]
    {
        // Embed Windows resource file with icon
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/icons/auroraview.ico");
        res.set("ProductName", "AuroraView");
        res.set("FileDescription", "AuroraView CLI - WebView launcher");
        res.set("LegalCopyright", "Copyright (c) AuroraView Authors");

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
            // Don't fail the build, just warn
        }
    }
}
