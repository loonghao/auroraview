//! Performance monitoring and optimization utilities

use std::time::{Duration, Instant};
use std::sync::Arc;
use parking_lot::Mutex;

/// Performance metrics for WebView initialization
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Time when WebView creation started
    pub creation_start: Instant,
    
    /// Time when window was created
    pub window_created: Option<Instant>,
    
    /// Time when WebView was created
    pub webview_created: Option<Instant>,
    
    /// Time when HTML was loaded
    pub html_loaded: Option<Instant>,
    
    /// Time when JavaScript initialized
    pub js_initialized: Option<Instant>,
    
    /// Time when first paint occurred
    pub first_paint: Option<Instant>,
    
    /// Time when window was shown
    pub window_shown: Option<Instant>,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new() -> Self {
        Self {
            creation_start: Instant::now(),
            window_created: None,
            webview_created: None,
            html_loaded: None,
            js_initialized: None,
            first_paint: None,
            window_shown: None,
        }
    }
    
    /// Mark window as created
    pub fn mark_window_created(&mut self) {
        self.window_created = Some(Instant::now());
    }
    
    /// Mark WebView as created
    pub fn mark_webview_created(&mut self) {
        self.webview_created = Some(Instant::now());
    }
    
    /// Mark HTML as loaded
    pub fn mark_html_loaded(&mut self) {
        self.html_loaded = Some(Instant::now());
    }
    
    /// Mark JavaScript as initialized
    pub fn mark_js_initialized(&mut self) {
        self.js_initialized = Some(Instant::now());
    }
    
    /// Mark first paint
    pub fn mark_first_paint(&mut self) {
        self.first_paint = Some(Instant::now());
    }
    
    /// Mark window as shown
    pub fn mark_window_shown(&mut self) {
        self.window_shown = Some(Instant::now());
    }
    
    /// Get time to window creation
    pub fn time_to_window(&self) -> Option<Duration> {
        self.window_created.map(|t| t.duration_since(self.creation_start))
    }
    
    /// Get time to WebView creation
    pub fn time_to_webview(&self) -> Option<Duration> {
        self.webview_created.map(|t| t.duration_since(self.creation_start))
    }
    
    /// Get time to HTML load
    pub fn time_to_html(&self) -> Option<Duration> {
        self.html_loaded.map(|t| t.duration_since(self.creation_start))
    }
    
    /// Get time to JavaScript initialization
    pub fn time_to_js(&self) -> Option<Duration> {
        self.js_initialized.map(|t| t.duration_since(self.creation_start))
    }
    
    /// Get time to first paint
    pub fn time_to_first_paint(&self) -> Option<Duration> {
        self.first_paint.map(|t| t.duration_since(self.creation_start))
    }
    
    /// Get time to window shown
    pub fn time_to_shown(&self) -> Option<Duration> {
        self.window_shown.map(|t| t.duration_since(self.creation_start))
    }
    
    /// Print performance report
    pub fn print_report(&self) {
        tracing::info!("=== Performance Report ===");
        
        if let Some(d) = self.time_to_window() {
            tracing::info!("⏱️  Window created: {:?}", d);
        }
        
        if let Some(d) = self.time_to_webview() {
            tracing::info!("⏱️  WebView created: {:?}", d);
        }
        
        if let Some(d) = self.time_to_html() {
            tracing::info!("⏱️  HTML loaded: {:?}", d);
        }
        
        if let Some(d) = self.time_to_js() {
            tracing::info!("⏱️  JavaScript initialized: {:?}", d);
        }
        
        if let Some(d) = self.time_to_first_paint() {
            tracing::info!("⏱️  First paint: {:?}", d);
        }
        
        if let Some(d) = self.time_to_shown() {
            tracing::info!("⏱️  Window shown: {:?}", d);
            tracing::info!("✅ Total time to interactive: {:?}", d);
        }
        
        tracing::info!("========================");
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe performance metrics tracker
pub type PerformanceTracker = Arc<Mutex<PerformanceMetrics>>;

/// Create a new performance tracker
pub fn create_tracker() -> PerformanceTracker {
    Arc::new(Mutex::new(PerformanceMetrics::new()))
}

