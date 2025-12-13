//! WebView2 Warmup/Preheat Module
//!
//! This module provides pre-initialization of WebView2 environment to reduce
//! cold-start latency in DCC applications (Maya, Houdini, Nuke, etc.).
//!
//! ## Problem
//! WebView2 first-time initialization takes 2-8 seconds due to:
//! - WebView2 runtime discovery
//! - Browser process spawning
//! - User data folder initialization
//!
//! ## Solution
//! Pre-initialize the WebView2 environment in background when:
//! - DCC application starts
//! - AuroraView module is imported
//!
//! This reduces first window creation time from 2-8s to <1s.
//!
//! NOTE: Functions in this module are exposed via PyO3 bindings (src/bindings/warmup.rs).
//! The dead_code warnings are expected when compiling without python-bindings feature.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Instant;

/// Global warmup state
static WARMUP_STATE: OnceLock<Arc<Mutex<WarmupState>>> = OnceLock::new();

/// Warmup progress stages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Some variants only used with win-webview2 feature
pub enum WarmupStage {
    /// Not started
    NotStarted,
    /// Initializing COM
    InitializingCom,
    /// Creating user data folder
    CreatingDataFolder,
    /// Loading WebView2 runtime
    LoadingRuntime,
    /// Creating WebView2 environment
    CreatingEnvironment,
    /// Warmup complete
    Complete,
    /// Warmup failed
    Failed,
}

impl WarmupStage {
    /// Get progress percentage (0-100)
    pub fn progress(&self) -> u8 {
        match self {
            WarmupStage::NotStarted => 0,
            WarmupStage::InitializingCom => 10,
            WarmupStage::CreatingDataFolder => 20,
            WarmupStage::LoadingRuntime => 40,
            WarmupStage::CreatingEnvironment => 70,
            WarmupStage::Complete => 100,
            WarmupStage::Failed => 0,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            WarmupStage::NotStarted => "Not started",
            WarmupStage::InitializingCom => "Initializing COM...",
            WarmupStage::CreatingDataFolder => "Creating data folder...",
            WarmupStage::LoadingRuntime => "Loading WebView2 runtime...",
            WarmupStage::CreatingEnvironment => "Creating WebView2 environment...",
            WarmupStage::Complete => "Ready",
            WarmupStage::Failed => "Failed",
        }
    }
}

/// Warmup state tracking
#[derive(Debug, Clone)]
pub struct WarmupState {
    /// Whether warmup has been initiated
    pub initiated: bool,
    /// Whether warmup is complete
    pub complete: bool,
    /// Current warmup stage
    pub stage: WarmupStage,
    /// Warmup duration in milliseconds (if complete)
    pub duration_ms: Option<u64>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// User data folder path (shared across all WebViews)
    pub user_data_folder: Option<PathBuf>,
}

impl Default for WarmupState {
    fn default() -> Self {
        Self {
            initiated: false,
            complete: false,
            stage: WarmupStage::NotStarted,
            duration_ms: None,
            error: None,
            user_data_folder: None,
        }
    }
}

/// Get current warmup progress (0-100)
pub fn get_warmup_progress() -> u8 {
    let state = get_warmup_state();
    let guard = match state.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.stage.progress()
}

/// Get current warmup stage description
pub fn get_warmup_stage_description() -> String {
    let state = get_warmup_state();
    let guard = match state.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.stage.description().to_string()
}

/// Get warmup state singleton
fn get_warmup_state() -> Arc<Mutex<WarmupState>> {
    WARMUP_STATE
        .get_or_init(|| Arc::new(Mutex::new(WarmupState::default())))
        .clone()
}

/// Get current warmup status
pub fn get_warmup_status() -> WarmupState {
    let state = get_warmup_state();
    let guard = match state.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.clone()
}

/// Check if warmup is complete
pub fn is_warmup_complete() -> bool {
    let state = get_warmup_state();
    let guard = match state.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.complete
}

/// Get shared user data folder path
pub fn get_shared_user_data_folder() -> Option<PathBuf> {
    let state = get_warmup_state();
    let guard = match state.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.user_data_folder.clone()
}

/// Start WebView2 warmup in background thread
///
/// This function initiates background pre-initialization of WebView2:
/// 1. Initializes COM in STA mode
/// 2. Creates WebView2 Environment (triggers runtime discovery)
/// 3. Pre-allocates shared user data folder
///
/// Call this early in your application startup (e.g., when importing auroraview).
///
/// # Arguments
/// * `user_data_folder` - Optional path for shared user data folder.
///   If None, uses system default (%LOCALAPPDATA%\AuroraView\WebView2)
pub fn start_warmup(user_data_folder: Option<PathBuf>) {
    let state = get_warmup_state();

    // Check if already initiated
    {
        let mut guard = match state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if guard.initiated {
            tracing::debug!("[warmup] Warmup already initiated, skipping");
            return;
        }
        guard.initiated = true;
    }

    // Determine user data folder
    let data_folder = user_data_folder.unwrap_or_else(|| {
        // Default: %LOCALAPPDATA%\AuroraView\WebView2
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(local_app_data)
            .join("AuroraView")
            .join("WebView2")
    });

    // Store the user data folder path
    {
        let mut guard = match state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.user_data_folder = Some(data_folder.clone());
    }

    tracing::info!(
        "[warmup] Starting WebView2 warmup (data_folder: {:?})",
        data_folder
    );

    // Spawn background thread for warmup
    let state_clone = state.clone();
    thread::Builder::new()
        .name("auroraview-webview2-warmup".to_string())
        .spawn(move || {
            let start = Instant::now();

            // Platform-specific warmup
            #[cfg(all(target_os = "windows", feature = "win-webview2"))]
            let result = warmup_webview2_windows(&data_folder);

            #[cfg(not(all(target_os = "windows", feature = "win-webview2")))]
            let result: Result<(), String> = Ok(()); // No-op on non-Windows

            let duration_ms = start.elapsed().as_millis() as u64;

            // Update state - handle poisoned mutex gracefully
            let mut guard = match state_clone.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            guard.complete = true;
            guard.duration_ms = Some(duration_ms);

            match result {
                Ok(()) => {
                    guard.stage = WarmupStage::Complete;
                    tracing::info!("[warmup] WebView2 warmup complete in {}ms", duration_ms);
                }
                Err(e) => {
                    guard.stage = WarmupStage::Failed;
                    guard.error = Some(e.clone());
                    tracing::warn!(
                        "[warmup] WebView2 warmup failed after {}ms: {}",
                        duration_ms,
                        e
                    );
                }
            }
        })
        .expect("Failed to spawn warmup thread");
}

/// Helper to update warmup stage
#[allow(dead_code)] // Only used with win-webview2 feature
fn update_warmup_stage(stage: WarmupStage) {
    let state = get_warmup_state();
    let mut guard = match state.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.stage = stage;
    tracing::debug!(
        "[warmup] Stage: {} ({}%)",
        stage.description(),
        stage.progress()
    );
}

/// Windows-specific WebView2 warmup implementation
#[cfg(all(target_os = "windows", feature = "win-webview2"))]
fn warmup_webview2_windows(data_folder: &PathBuf) -> Result<(), String> {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

    // Stage 1: Initialize COM
    update_warmup_stage(WarmupStage::InitializingCom);
    unsafe {
        let result = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if result.is_ok() {
            tracing::debug!("[warmup] COM initialized in STA mode");
        } else {
            tracing::debug!("[warmup] COM already initialized: {:?}", result);
        }
    }

    // Stage 2: Create user data folder
    update_warmup_stage(WarmupStage::CreatingDataFolder);
    if let Err(e) = std::fs::create_dir_all(data_folder) {
        tracing::warn!("[warmup] Failed to create user data folder: {}", e);
        // Don't fail - the folder might already exist or we'll use default
    }

    // Touch the folder to ensure Windows file system caches the path
    // This pre-warms file system caches
    let _ = std::fs::read_dir(data_folder);

    // Stage 3: Load WebView2 runtime
    update_warmup_stage(WarmupStage::LoadingRuntime);

    // Stage 4: Create WebView2 environment
    update_warmup_stage(WarmupStage::CreatingEnvironment);
    warmup_webview2_runtime()?;

    Ok(())
}

/// Pre-load WebView2 runtime to trigger DLL loading and cache warming
#[cfg(all(target_os = "windows", feature = "win-webview2"))]
fn warmup_webview2_runtime() -> Result<(), String> {
    use std::sync::mpsc;
    use std::time::{Duration, Instant};
    use webview2::Environment;
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    };

    tracing::debug!("[warmup] Creating WebView2 Environment...");

    let (tx, rx) = mpsc::channel();
    let builder = Environment::builder();

    // Trigger environment creation (this loads WebView2 runtime)
    if let Err(e) = builder.build(move |res| {
        let _ = tx.send(res.map(|_| ()));
        Ok(())
    }) {
        return Err(format!("Environment::builder().build failed: {}", e));
    }

    // Wait for completion with message pump
    let start = Instant::now();
    let timeout = Duration::from_secs(30);

    loop {
        if let Ok(res) = rx.try_recv() {
            return res.map_err(|e| e.to_string());
        }

        // Pump Windows messages (required for WebView2 callbacks)
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        std::thread::sleep(Duration::from_millis(10));

        if start.elapsed() > timeout {
            return Err("Timeout waiting for WebView2 Environment".to_string());
        }
    }
}

/// Synchronous warmup - blocks until WebView2 is ready
///
/// Use this if you need to ensure WebView2 is fully initialized before
/// creating windows. For most cases, prefer `start_warmup()` for
/// non-blocking background initialization.
///
/// # Arguments
/// * `user_data_folder` - Optional path for shared user data folder
/// * `timeout_ms` - Maximum time to wait (default: 30000ms)
pub fn warmup_sync(
    user_data_folder: Option<PathBuf>,
    timeout_ms: Option<u64>,
) -> Result<(), String> {
    let timeout = std::time::Duration::from_millis(timeout_ms.unwrap_or(30000));
    let start = std::time::Instant::now();

    // Start warmup if not already started
    start_warmup(user_data_folder);

    // Wait for completion
    loop {
        if is_warmup_complete() {
            let state = get_warmup_status();
            if let Some(error) = state.error {
                return Err(error);
            }
            return Ok(());
        }

        if start.elapsed() > timeout {
            return Err("Warmup timeout".to_string());
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warmup_state_default() {
        let state = WarmupState::default();
        assert!(!state.initiated);
        assert!(!state.complete);
        assert!(state.duration_ms.is_none());
        assert!(state.error.is_none());
    }

    #[test]
    fn test_get_warmup_status() {
        let status = get_warmup_status();
        // Initial state: either not initiated, or if initiated, may or may not be complete
        // The assertion just checks that the status struct is valid
        assert!(!status.initiated || status.complete);
    }
}
