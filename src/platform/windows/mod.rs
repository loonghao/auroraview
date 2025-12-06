// Windows-specific platform modules
#[cfg(feature = "win-webview2")]
pub mod webview2;

// WebView2 warmup/preheat module for reducing cold-start latency
pub mod warmup;
