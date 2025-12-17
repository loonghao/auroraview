//! Python backend process and IPC communication
//!
//! Uses ipckit's `ShutdownState` for graceful shutdown coordination,
//! preventing "EventLoopClosed" errors when the WebView is closing.

use anyhow::{Context, Result};
use auroraview_pack::{BundleStrategy, OverlayData, PackedMetrics, PythonBundleConfig};
use ipckit::graceful::ShutdownState;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use tao::event_loop::EventLoopProxy;

use super::events::UserEvent;
use super::extract::{extract_resources_parallel, extract_standalone_python};
use super::utils::{build_module_search_paths, get_python_extract_dir_with_hash};

/// Python backend handle for IPC communication
///
/// Uses ipckit's `ShutdownState` for graceful shutdown coordination.
pub struct PythonBackend {
    process: Mutex<Child>,
    stdin: Arc<Mutex<ChildStdin>>,
    /// Shutdown state from ipckit for graceful shutdown coordination
    shutdown_state: Arc<ShutdownState>,
}

impl PythonBackend {
    /// Check if Python process is still running
    pub fn is_alive(&self) -> bool {
        if let Ok(mut process) = self.process.lock() {
            match process.try_wait() {
                Ok(None) => true, // Still running
                Ok(Some(status)) => {
                    tracing::warn!("Python process exited with status: {:?}", status);
                    false
                }
                Err(e) => {
                    tracing::error!("Failed to check Python process status: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Send a JSON-RPC request to Python backend
    pub fn send_request(&self, request: &str) -> Result<()> {
        // Check if shutdown has been initiated
        if self.shutdown_state.is_shutdown() {
            return Err(anyhow::anyhow!("Python backend is shutting down"));
        }

        // Check if process is still alive before sending
        if !self.is_alive() {
            return Err(anyhow::anyhow!("Python backend process has exited"));
        }

        let mut stdin = self
            .stdin
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        writeln!(stdin, "{}", request)?;
        stdin.flush()?;
        Ok(())
    }

    /// Initiate graceful shutdown
    ///
    /// Signals all background threads to stop and waits for pending operations.
    pub fn shutdown(&self) {
        tracing::info!("[Rust] Initiating Python backend shutdown...");
        self.shutdown_state.shutdown();

        // Wait for pending operations with timeout
        if let Err(e) = self
            .shutdown_state
            .wait_for_drain(Some(std::time::Duration::from_secs(2)))
        {
            tracing::warn!("[Rust] Shutdown drain timeout: {:?}", e);
        }
    }

    /// Check if shutdown has been initiated
    #[allow(dead_code)]
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_state.is_shutdown()
    }
}

/// Start Python backend process for FullStack mode with IPC support
pub fn start_python_backend_with_ipc(
    overlay: &OverlayData,
    python_config: &PythonBundleConfig,
    proxy: EventLoopProxy<UserEvent>,
    metrics: &mut PackedMetrics,
) -> Result<PythonBackend> {
    let func_start = Instant::now();

    // Helper to send loading update
    let send_loading_update = |proxy: &EventLoopProxy<UserEvent>,
                               progress: Option<i32>,
                               text: Option<&str>,
                               step_id: Option<&str>,
                               step_text: Option<&str>,
                               step_status: Option<&str>| {
        let _ = proxy.send_event(UserEvent::LoadingUpdate {
            progress,
            text: text.map(|s| s.to_string()),
            step_id: step_id.map(|s| s.to_string()),
            step_text: step_text.map(|s| s.to_string()),
            step_status: step_status.map(|s| s.to_string()),
        });
    };

    // Initial loading state
    send_loading_update(
        &proxy,
        Some(5),
        Some("Initializing environment..."),
        Some("init"),
        Some("Initializing environment"),
        Some("active"),
    );

    // Determine Python executable path based on strategy
    let python_exe = match python_config.strategy {
        BundleStrategy::Standalone => {
            // Update loading: extracting Python runtime
            send_loading_update(
                &proxy,
                Some(10),
                Some("Extracting Python runtime..."),
                Some("init"),
                Some("Initializing environment"),
                Some("completed"),
            );
            send_loading_update(
                &proxy,
                None,
                None,
                Some("python_runtime"),
                Some("Extracting Python runtime"),
                Some("active"),
            );

            // Extract embedded Python runtime
            let runtime_start = Instant::now();
            let exe = extract_standalone_python(overlay)?;
            metrics.add_phase("python_runtime_extract", runtime_start.elapsed());
            metrics.mark_python_runtime_extract();

            send_loading_update(
                &proxy,
                Some(40),
                Some("Python runtime ready"),
                Some("python_runtime"),
                Some("Extracting Python runtime"),
                Some("completed"),
            );
            exe
        }
        _ => {
            // Use system Python for other strategies
            send_loading_update(
                &proxy,
                Some(20),
                Some("Using system Python..."),
                Some("init"),
                Some("Initializing environment"),
                Some("completed"),
            );
            PathBuf::from("python")
        }
    };

    // Determine cache directory based on content hash
    // Uses hash-based caching (like uv) for:
    // - Cache reuse: Same content → same hash → skip extraction
    // - Conflict avoidance: Different content → different hash → new directory
    // - Multi-version support: Multiple versions can coexist
    let hash = &overlay.content_hash;
    let hash_dir = get_python_extract_dir_with_hash(hash);
    let marker_file = hash_dir.join(".cache_valid");

    // Check if cache is already valid
    let cache_valid = marker_file.exists();
    if cache_valid {
        tracing::info!(
            "Using cached Python files (hash: {}): {}",
            hash,
            hash_dir.display()
        );
    } else {
        tracing::info!(
            "Extracting Python files (hash: {}): {}",
            hash,
            hash_dir.display()
        );
    }
    let temp_dir = hash_dir;

    fs::create_dir_all(&temp_dir)?;

    tracing::info!("Python files directory: {}", temp_dir.display());

    // Collect Python files to extract
    let python_assets: Vec<_> = overlay
        .assets
        .iter()
        .filter(|(path, _)| path.starts_with("python/"))
        .collect();

    if !python_assets.is_empty() && !cache_valid {
        // Update loading: extracting Python files
        send_loading_update(
            &proxy,
            Some(50),
            Some("Extracting application files..."),
            Some("python_files"),
            Some("Extracting application files"),
            Some("active"),
        );

        let extract_start = Instant::now();

        // Pre-create all directories in batch (collect unique parent dirs)
        let dirs: HashSet<PathBuf> = python_assets
            .iter()
            .filter_map(|(path, _)| {
                let rel_path = path.strip_prefix("python/").unwrap_or(path);
                temp_dir.join(rel_path).parent().map(|p| p.to_path_buf())
            })
            .collect();

        for dir in &dirs {
            fs::create_dir_all(dir)?;
        }

        metrics.add_phase("python_dirs_create", extract_start.elapsed());

        // Parallel file extraction using rayon
        // Handle file locking gracefully - if file exists and is locked (e.g., .pyd files),
        // check if content matches and skip if identical
        let write_start = Instant::now();
        let results: Vec<Result<String, anyhow::Error>> = python_assets
            .par_iter()
            .map(|(path, content)| {
                let rel_path = path.strip_prefix("python/").unwrap_or(path);
                let dest_path = temp_dir.join(rel_path);

                // Try to write the file
                match fs::write(&dest_path, content) {
                    Ok(_) => Ok(rel_path.to_string()),
                    Err(e) => {
                        // Check if it's a file locking error (os error 32 on Windows)
                        let os_error = e.raw_os_error();
                        if os_error == Some(32) {
                            // File is locked, check if existing file has same content
                            match fs::read(&dest_path) {
                                Ok(existing_content) => {
                                    if existing_content.as_slice() == content.as_slice() {
                                        tracing::info!(
                                            "File {} is locked but content matches, skipping",
                                            rel_path
                                        );
                                        return Ok(rel_path.to_string());
                                    }
                                    tracing::warn!(
                                        "File {} is locked and content differs (existing: {} bytes, new: {} bytes)",
                                        rel_path, existing_content.len(), content.len()
                                    );
                                    Err(anyhow::anyhow!(
                                        "File {} is locked by another process and content differs",
                                        dest_path.display()
                                    ))
                                }
                                Err(read_err) => {
                                    tracing::warn!(
                                        "File {} is locked and cannot be read: {}",
                                        rel_path,
                                        read_err
                                    );
                                    Err(anyhow::anyhow!(
                                        "File {} is locked and cannot be read: {}",
                                        dest_path.display(),
                                        read_err
                                    ))
                                }
                            }
                        } else {
                            Err(e)
                                .with_context(|| format!("Failed to write: {}", dest_path.display()))
                        }
                    }
                }
            })
            .collect();

        // Check for errors
        let mut python_files = Vec::with_capacity(results.len());
        for result in results {
            python_files.push(result?);
        }

        metrics.add_phase("python_files_write", write_start.elapsed());
        metrics.mark_python_files_extract();

        tracing::info!(
            "Extracted {} Python files in {:.2}ms",
            python_files.len(),
            extract_start.elapsed().as_secs_f64() * 1000.0
        );

        send_loading_update(
            &proxy,
            Some(60),
            Some("Application files ready"),
            Some("python_files"),
            Some("Extracting application files"),
            Some("completed"),
        );
    } else if cache_valid {
        // Cache is valid, skip extraction
        send_loading_update(
            &proxy,
            Some(60),
            Some("Using cached application files"),
            Some("python_files"),
            Some("Using cached files"),
            Some("completed"),
        );
        tracing::info!(
            "Skipped extraction: {} Python files (cache valid)",
            python_assets.len()
        );
    }

    // Extract resource directories (examples, etc.) from overlay assets
    // Only extract if cache is not valid
    let resources_dir = if cache_valid {
        // Use cached resources directory
        let cached_resources = temp_dir.join("resources");
        send_loading_update(
            &proxy,
            Some(70),
            Some("Using cached resources"),
            Some("resources"),
            Some("Using cached resources"),
            Some("completed"),
        );
        tracing::info!("Using cached resources: {}", cached_resources.display());
        cached_resources
    } else {
        send_loading_update(
            &proxy,
            Some(65),
            Some("Extracting resources..."),
            Some("resources"),
            Some("Extracting resources"),
            Some("active"),
        );

        let resources_start = Instant::now();
        let resources_dir = extract_resources_parallel(overlay, &temp_dir)?;
        metrics.add_phase("resources_extract", resources_start.elapsed());
        metrics.mark_resources_extract();

        send_loading_update(
            &proxy,
            Some(70),
            Some("Resources ready"),
            Some("resources"),
            Some("Extracting resources"),
            Some("completed"),
        );

        // Write cache marker after successful extraction
        let marker_file = temp_dir.join(".cache_valid");
        if let Err(e) = fs::write(&marker_file, "1") {
            tracing::warn!("Failed to write cache marker: {}", e);
        } else {
            tracing::info!("Cache marker written: {}", marker_file.display());
        }

        resources_dir
    };

    // Parse entry point (format: "module:function" or "file.py")
    let entry_point = &python_config.entry_point;
    let (module, function) = if entry_point.contains(':') {
        let parts: Vec<&str> = entry_point.split(':').collect();
        (parts[0], Some(parts.get(1).copied().unwrap_or("main")))
    } else {
        (entry_point.as_str(), None)
    };

    // Build module search paths from configuration
    let site_packages_dir = temp_dir.join("site-packages");
    let module_paths = build_module_search_paths(
        &python_config.module_search_paths,
        &temp_dir,
        &resources_dir,
        &site_packages_dir,
    );

    // Build Python command with module paths
    // Use runpy.run_path() to properly set __file__ and __name__ variables,
    // so developers don't need to handle packed mode specially in their code.
    let script_path = temp_dir.join(module);
    let python_code = if let Some(func) = function {
        // Import module and call function
        format!(
            "import sys; sys.path.insert(0, r'{}'); from {} import {}; {}()",
            temp_dir.display(),
            module.replace(['/', '\\'], ".").trim_end_matches(".py"),
            func,
            func
        )
    } else {
        // Use runpy.run_path() which properly sets __file__, __name__, etc.
        // This allows developers to use `if __name__ == "__main__"` and
        // Path(__file__).parent without any packed-mode specific handling.
        format!(
            r#"import sys; sys.path.insert(0, r'{}'); import runpy; runpy.run_path(r'{}', run_name='__main__')"#,
            temp_dir.display(),
            script_path.display()
        )
    };

    // Update loading: starting Python
    send_loading_update(
        &proxy,
        Some(80),
        Some("Starting Python backend..."),
        Some("python_start"),
        Some("Starting Python backend"),
        Some("active"),
    );

    tracing::debug!("[Rust] Starting Python backend: {}", entry_point);
    tracing::debug!("[Rust] Using Python: {}", python_exe.display());
    tracing::debug!("[Rust] Python code: {}", python_code);
    tracing::debug!("[Rust] Module search paths: {:?}", module_paths);

    // Build PYTHONPATH from module search paths
    let pythonpath = module_paths.join(if cfg!(windows) { ";" } else { ":" });

    // Get Python home directory (parent of python.exe)
    // This is required for python-build-standalone to find its standard library
    let python_home = python_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get Python home directory"))?;

    // Create isolation context for environment isolation
    use super::utils::{
        apply_isolation_to_command, store_isolation_context_in_env, IsolationContext,
    };

    let isolation_context = IsolationContext {
        extract_dir: temp_dir.clone(),
        resources_dir: resources_dir.clone(),
        python_home: python_home.to_path_buf(),
        site_packages_dir: site_packages_dir.clone(),
        pythonpath: pythonpath.clone(),
    };

    // Log isolation config values for debugging
    tracing::info!(
        "[Rust] Isolation config from overlay: isolate_path={}, isolate_pythonpath={}",
        python_config.isolation.isolate_path,
        python_config.isolation.isolate_pythonpath
    );

    // Store isolation context in environment for child processes (ProcessPlugin.spawn_ipc)
    store_isolation_context_in_env(&isolation_context, &python_config.isolation);

    // Set basic environment variables in the current Rust process
    // These are needed for ProcessPlugin.spawn_ipc to inherit
    std::env::set_var("AURORAVIEW_PACKED", "1");
    std::env::set_var("AURORAVIEW_RESOURCES_DIR", &resources_dir);
    std::env::set_var("AURORAVIEW_PYTHON_PATH", &pythonpath);
    std::env::set_var("AURORAVIEW_PYTHON_EXE", &python_exe);
    tracing::debug!("[Rust] Set AURORAVIEW_PYTHON_PATH={}", pythonpath);
    tracing::debug!(
        "[Rust] Set AURORAVIEW_RESOURCES_DIR={}",
        resources_dir.display()
    );
    tracing::debug!("[Rust] Set AURORAVIEW_PYTHON_EXE={}", python_exe.display());

    // Log isolation settings
    tracing::info!(
        "[Rust] Environment isolation: PATH={}, PYTHONPATH={}",
        if python_config.isolation.isolate_path {
            "isolated"
        } else {
            "inherited"
        },
        if python_config.isolation.isolate_pythonpath {
            "isolated"
        } else {
            "inherited"
        }
    );

    // Start Python process with environment isolation
    let spawn_start = Instant::now();
    let mut cmd = Command::new(&python_exe);
    cmd.args(["-c", &python_code])
        .current_dir(&temp_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()); // Capture stderr for error diagnostics

    // Apply environment isolation (rez-style)
    apply_isolation_to_command(&mut cmd, &python_config.isolation, &isolation_context);

    // Set additional AuroraView-specific environment variables
    cmd.env("AURORAVIEW_PACKED", "1")
        .env("AURORAVIEW_RESOURCES_DIR", &resources_dir)
        .env("AURORAVIEW_PYTHON_PATH", &pythonpath)
        .env("AURORAVIEW_PYTHON_EXE", &python_exe)
        .env("PYTHONUNBUFFERED", "1"); // Ensure Python stderr is not buffered

    // Windows: hide console window unless show_console is enabled
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_CONSOLE: u32 = 0x00000010;

        if python_config.show_console {
            tracing::debug!("Python console window enabled");
            cmd.creation_flags(CREATE_NEW_CONSOLE);
        } else {
            // Use DETACHED_PROCESS instead of CREATE_NO_WINDOW
            // CREATE_NO_WINDOW (0x08000000) can cause issues with some Python operations
            cmd.creation_flags(DETACHED_PROCESS);
        }
    }

    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to start Python backend: {}", python_exe.display()))?;

    metrics.add_phase("python_spawn", spawn_start.elapsed());
    metrics.mark_python_start();

    // Update loading: Python process started
    send_loading_update(
        &proxy,
        Some(85),
        Some("Python process started, waiting for initialization..."),
        Some("python_start"),
        Some("Starting Python backend"),
        Some("completed"),
    );
    send_loading_update(
        &proxy,
        None,
        None,
        Some("python_init"),
        Some("Initializing application"),
        Some("active"),
    );

    tracing::info!(
        "Python backend started (PID: {}) in {:.2}ms",
        child.id(),
        func_start.elapsed().as_secs_f64() * 1000.0
    );

    // Take ownership of stdin, stdout, and stderr
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get stderr"))?;

    let stdin = Arc::new(Mutex::new(stdin));

    // Spawn thread to read stderr and log errors
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) if !line.is_empty() => {
                    tracing::error!("[Python stderr] {}", line);
                }
                Err(e) => {
                    tracing::debug!("Python stderr reader ended: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Create shutdown state for graceful shutdown coordination (using ipckit)
    let shutdown_state = Arc::new(ShutdownState::new());
    let shutdown_state_for_thread = Arc::clone(&shutdown_state);

    // Spawn thread to wait for Python ready signal and then read responses
    // This is non-blocking so WebView can show loading screen while waiting
    // Uses ipckit's ShutdownState for graceful shutdown
    let ready_start = Instant::now();
    let proxy_for_loading = proxy.clone();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut ready_line = String::new();

        // Read the first line - should be the ready signal
        match reader.read_line(&mut ready_line) {
            Ok(0) => {
                tracing::error!("Python process closed stdout before sending ready signal");
                return;
            }
            Ok(_) => {
                let ready_line_trimmed = ready_line.trim();
                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(ready_line_trimmed) {
                    if msg.get("type").and_then(|v| v.as_str()) == Some("ready") {
                        // Extract handler names from Python ready signal
                        let handlers: Vec<String> = msg
                            .get("handlers")
                            .and_then(|v| v.as_array())
                            .map(|a| {
                                a.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default();

                        tracing::info!(
                            "Python backend ready with {} handlers in {:.2}ms",
                            handlers.len(),
                            ready_start.elapsed().as_secs_f64() * 1000.0
                        );
                        tracing::debug!("Registered handlers: {:?}", handlers);

                        // Update loading: Python ready, navigating to app
                        let _ = proxy_for_loading.send_event(UserEvent::LoadingUpdate {
                            progress: Some(95),
                            text: Some("Application ready, loading UI...".to_string()),
                            step_id: Some("python_init".to_string()),
                            step_text: Some("Initializing application".to_string()),
                            step_status: Some("completed".to_string()),
                        });

                        // Notify WebView to navigate to actual content with handlers
                        if let Err(e) = proxy.send_event(UserEvent::PythonReady { handlers }) {
                            tracing::error!("Failed to send PythonReady event: {}", e);
                        }
                    } else {
                        tracing::warn!(
                            "Unexpected first message from Python (expected ready signal): {}",
                            ready_line_trimmed
                        );
                        // Still notify ready to avoid hanging on loading screen
                        let _ = proxy.send_event(UserEvent::PythonReady {
                            handlers: Vec::new(),
                        });
                    }
                } else {
                    tracing::warn!(
                        "Failed to parse Python ready signal: {}",
                        ready_line_trimmed
                    );
                    // Still notify ready to avoid hanging on loading screen
                    let _ = proxy.send_event(UserEvent::PythonReady {
                        handlers: Vec::new(),
                    });
                }
            }
            Err(e) => {
                tracing::error!("Failed to read Python ready signal: {}", e);
                return;
            }
        }

        // Continue reading Python stdout and forward responses
        // Check shutdown state before each send to avoid EventLoopClosed errors
        for line in reader.lines() {
            // Check if shutdown has been initiated (using ipckit)
            if shutdown_state_for_thread.is_shutdown() {
                tracing::debug!("[Rust] Shutdown detected, stopping Python stdout reader");
                break;
            }

            match line {
                Ok(response) => {
                    if response.trim().is_empty() {
                        continue;
                    }

                    // Use operation guard to track this send operation (ipckit)
                    let _guard = shutdown_state_for_thread.begin_operation();

                    tracing::debug!("Python response: {}", response);
                    // Send response to event loop
                    if let Err(e) = proxy.send_event(UserEvent::PythonResponse(response)) {
                        // Check if this is an EventLoopClosed error
                        if shutdown_state_for_thread.is_shutdown() {
                            tracing::debug!("Event loop closed during shutdown, stopping reader");
                        } else {
                            tracing::error!("Failed to send response to event loop: {}", e);
                        }
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading Python stdout: {}", e);
                    break;
                }
            }
        }
        tracing::info!("Python stdout reader thread exiting");
    });

    Ok(PythonBackend {
        process: Mutex::new(child),
        stdin,
        shutdown_state,
    })
}
