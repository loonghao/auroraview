//! Window builder for desktop mode

use crate::config::DesktopConfig;
use crate::error::{DesktopError, Result};
use crate::event_loop::UserEvent;
use crate::ipc::IpcRouter;
use crate::window::DesktopWindow;
use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoop;
use tao::window::WindowBuilder;
use tracing::{debug, info};
use wry::WebViewBuilder;

#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;

/// Create a desktop window with WebView
pub fn create_window(
    config: DesktopConfig,
    event_loop: &EventLoop<UserEvent>,
) -> Result<DesktopWindow> {
    create_window_with_router(config, event_loop, None)
}

/// Create a desktop window with WebView and shared IPC router
pub fn create_window_with_router(
    config: DesktopConfig,
    event_loop: &EventLoop<UserEvent>,
    router: Option<Arc<IpcRouter>>,
) -> Result<DesktopWindow> {
    // Clean up stale WebView user data directories from crashed processes
    // This runs once per process and prevents initialization issues
    match auroraview_core::cleanup::cleanup_stale_webview_dirs() {
        Ok(count) if count > 0 => {
            info!("[desktop] Cleaned up {} stale WebView directories", count);
        }
        Err(e) => {
            debug!("[desktop] Cleanup warning: {}", e);
        }
        _ => {}
    }

    info!(
        "[desktop] Creating window: title='{}', size={}x{}",
        config.title, config.width, config.height
    );

    // Initialize COM on Windows
    #[cfg(target_os = "windows")]
    init_com();

    // Build window
    let mut window_builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_resizable(config.resizable)
        .with_decorations(config.decorations)
        .with_transparent(config.transparent)
        .with_always_on_top(config.always_on_top)
        .with_visible(false); // Start hidden to avoid white flash

    // Set window size or maximize
    if config.width == 0 || config.height == 0 {
        window_builder = window_builder.with_maximized(true);
    } else {
        window_builder = window_builder
            .with_inner_size(tao::dpi::LogicalSize::new(config.width, config.height));
    }

    // Load window icon
    if let Some(icon) = load_window_icon(config.icon.as_ref()) {
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    let window = window_builder
        .build(event_loop)
        .map_err(|e| DesktopError::WindowCreation(e.to_string()))?;

    // Cache HWND
    #[cfg(target_os = "windows")]
    let hwnd = get_window_hwnd(&window);

    // Create WebContext
    let mut web_context = create_web_context(&config);

    // Build WebView
    let mut webview_builder = WebViewBuilder::new_with_web_context(&mut web_context);

    if config.devtools {
        webview_builder = webview_builder.with_devtools(true);
    }

    // Set background color
    if config.transparent {
        webview_builder = webview_builder
            .with_transparent(true)
            .with_background_color((0, 0, 0, 0));
    }

    // Set CDP debugging port
    #[cfg(target_os = "windows")]
    if config.debug_port > 0 {
        let args = format!("--remote-debugging-port={}", config.debug_port);
        webview_builder = webview_builder.with_additional_browser_args(&args);
    }

    // Load initial content
    if let Some(ref html) = config.html {
        webview_builder = webview_builder.with_html(html);
    } else if let Some(ref url) = config.url {
        webview_builder = webview_builder.with_url(url);
    }

    // Custom user agent
    if let Some(ref user_agent) = config.user_agent {
        webview_builder = webview_builder.with_user_agent(user_agent);
    }

    // Use shared router or create new one
    let ipc_router = router.unwrap_or_else(|| Arc::new(IpcRouter::new()));
    let router_for_ipc = ipc_router.clone();

    // Add IPC handler to WebView
    webview_builder = webview_builder.with_ipc_handler(move |request| {
        let body = request.body();
        debug!("[IPC] Message received: {}", body);

        if let Some(response) = router_for_ipc.handle(body) {
            debug!("[IPC] Response: {}", response);
            // TODO: Send response back to JS via eval
        }
    });

    // Build WebView
    let webview = webview_builder
        .build(&window)
        .map_err(|e| DesktopError::WebViewCreation(e.to_string()))?;

    let event_loop_proxy = event_loop.create_proxy();

    Ok(DesktopWindow {
        webview: Arc::new(Mutex::new(webview)),
        window,
        event_loop_proxy,
        config,
        router: ipc_router,
        #[cfg(target_os = "windows")]
        hwnd,
    })
}

/// Initialize COM on Windows
#[cfg(target_os = "windows")]
fn init_com() {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
}

/// Get window HWND on Windows
#[cfg(target_os = "windows")]
fn get_window_hwnd(window: &tao::window::Window) -> Option<u64> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    if let Ok(handle) = window.window_handle() {
        if let RawWindowHandle::Win32(h) = handle.as_raw() {
            return Some(h.hwnd.get() as u64);
        }
    }
    None
}

/// Create WebContext with appropriate data directory
fn create_web_context(config: &DesktopConfig) -> wry::WebContext {
    if let Some(ref data_dir) = config.data_dir {
        wry::WebContext::new(Some(data_dir.clone()))
    } else {
        #[cfg(target_os = "windows")]
        {
            let local_app_data =
                std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
            let pid = std::process::id();
            let cache_dir = std::path::PathBuf::from(local_app_data)
                .join("AuroraView")
                .join("WebView2")
                .join(format!("process_{}", pid));
            wry::WebContext::new(Some(cache_dir))
        }
        #[cfg(not(target_os = "windows"))]
        wry::WebContext::default()
    }
}

/// Load window icon
fn load_window_icon(custom_icon: Option<&std::path::PathBuf>) -> Option<tao::window::Icon> {
    use image::GenericImageView;

    // Default icon bytes
    const DEFAULT_ICON: &[u8] = include_bytes!("../../assets/icons/auroraview-32.png");

    // Try custom icon first
    if let Some(icon_path) = custom_icon {
        if icon_path.exists() {
            if let Ok(img) = image::open(icon_path) {
                let (width, height) = img.dimensions();
                let rgba = img.into_rgba8().into_raw();
                if let Ok(icon) = tao::window::Icon::from_rgba(rgba, width, height) {
                    return Some(icon);
                }
            }
        }
    }

    // Fall back to default
    let img = image::load_from_memory(DEFAULT_ICON).ok()?;
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    tao::window::Icon::from_rgba(rgba, width, height).ok()
}
