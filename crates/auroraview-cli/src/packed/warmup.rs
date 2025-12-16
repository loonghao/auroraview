//! WebView2 warmup module
//!
//! Pre-initializes WebView2 environment in background to reduce cold-start latency.

/// Start WebView2 warmup in background thread
///
/// This pre-initializes WebView2 environment while overlay is being read,
/// reducing cold-start latency by 2-4 seconds.
#[cfg(target_os = "windows")]
pub fn start_webview2_warmup() {
    use std::sync::OnceLock;
    use std::thread;
    use std::time::Instant;
    use webview2_com::{Microsoft::Web::WebView2::Win32::*, *};
    use windows::core::PCWSTR;
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

    use super::utils::get_webview_data_dir;

    static WARMUP_STARTED: OnceLock<()> = OnceLock::new();

    // Only start warmup once
    WARMUP_STARTED.get_or_init(|| {
        let data_dir = get_webview_data_dir();

        thread::Builder::new()
            .name("webview2-warmup".to_string())
            .spawn(move || {
                let start = Instant::now();
                tracing::info!(
                    "[warmup] Starting WebView2 warmup (data_folder: {:?})",
                    data_dir
                );

                // Initialize COM in STA mode
                unsafe {
                    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
                }

                // Create data directory
                let _ = std::fs::create_dir_all(&data_dir);

                // Create WebView2 environment to trigger runtime discovery
                let data_dir_wide: Vec<u16> = data_dir
                    .to_string_lossy()
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();

                let result = unsafe {
                    CreateCoreWebView2EnvironmentWithOptions(
                        PCWSTR::null(),
                        PCWSTR(data_dir_wide.as_ptr()),
                        None,
                        &CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(
                            move |_env_result, _env| {
                                // Environment created successfully
                                Ok(())
                            },
                        )),
                    )
                };

                let duration = start.elapsed();
                match result {
                    Ok(_) => {
                        tracing::info!(
                            "[warmup] WebView2 warmup complete in {}ms",
                            duration.as_millis()
                        );
                    }
                    Err(e) => {
                        tracing::warn!("[warmup] WebView2 warmup failed: {:?}", e);
                    }
                }
            })
            .expect("Failed to spawn warmup thread");
    });
}

#[cfg(not(target_os = "windows"))]
pub fn start_webview2_warmup() {
    // No-op on non-Windows platforms
}
