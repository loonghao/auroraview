//! Message processing utilities for WebView
//!
//! This module provides a unified message processor to reduce code duplication
//! between `process_events()` and `process_ipc_only()`.
//!
//! NOTE: This module is prepared for future refactoring. Currently the message
//! processing logic is duplicated in `event_loop.rs` and `backend/native.rs`.
//! Once stabilized, we can migrate to use this unified processor.

#![allow(dead_code)]

use crate::ipc::WebViewMessage;
use crate::webview::js_assets;
use std::sync::{Arc, Mutex};
use wry::WebView as WryWebView;

/// Process a single WebView message
///
/// This is the unified message handler used by both `process_events()` and
/// `process_ipc_only()`. It handles all message types consistently.
///
/// # Arguments
/// * `webview` - Reference to the locked WebView
/// * `message` - The message to process
/// * `context` - A string identifying the caller (for logging)
pub fn process_message(webview: &WryWebView, message: WebViewMessage, context: &str) {
    match message {
        WebViewMessage::EvalJs(script) => {
            tracing::debug!("[{}] Processing EvalJs: {}", context, script);
            if let Err(e) = webview.evaluate_script(&script) {
                tracing::error!("[{}] Failed to execute JavaScript: {}", context, e);
            }
        }
        WebViewMessage::EmitEvent { event_name, data } => {
            tracing::debug!(
                "[OK] [{}] Emitting event: {} with data: {}",
                context,
                event_name,
                data
            );
            let json_str = data.to_string();
            let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
            let script = format!(
                "window.dispatchEvent(new CustomEvent('{}', {{ detail: JSON.parse('{}') }}));",
                event_name, escaped_json
            );
            if let Err(e) = webview.evaluate_script(&script) {
                tracing::error!("[{}] Failed to emit event: {}", context, e);
            } else {
                tracing::debug!("[OK] [{}] Event emitted successfully", context);
            }
        }
        WebViewMessage::LoadUrl(url) => {
            // Use native WebView load_url() instead of JavaScript window.location.href
            // This is more reliable, especially after splash screen loading
            tracing::info!("[{}] Loading URL via native API: {}", context, url);
            if let Err(e) = webview.load_url(&url) {
                tracing::error!("[{}] Failed to load URL: {}", context, e);
            }
        }
        WebViewMessage::LoadHtml(html) => {
            tracing::debug!("[{}] Processing LoadHtml ({} bytes)", context, html.len());
            if let Err(e) = webview.load_html(&html) {
                tracing::error!("[{}] Failed to load HTML: {}", context, e);
            }
        }
        WebViewMessage::WindowEvent { event_type, data } => {
            let event_name = event_type.as_str();
            let json_str = data.to_string();
            let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
            let script = js_assets::build_emit_event_script(event_name, &escaped_json);
            tracing::debug!(
                "[WINDOW_EVENT] [{}] Emitting window event: {}",
                context,
                event_name
            );
            if let Err(e) = webview.evaluate_script(&script) {
                tracing::error!("[{}] Failed to emit window event: {}", context, e);
            }
        }
        WebViewMessage::SetVisible(_) => {
            // SetVisible is handled at window level, not in message processing
        }
        WebViewMessage::EvalJsAsync {
            script,
            callback_id,
        } => {
            let async_script = js_assets::build_eval_js_async_script(&script, callback_id);
            if let Err(e) = webview.evaluate_script(&async_script) {
                tracing::error!(
                    "[{}] Failed to execute async JavaScript (id={}): {}",
                    context,
                    callback_id,
                    e
                );
            }
        }
        WebViewMessage::Reload => {
            if let Err(e) = webview.evaluate_script("location.reload()") {
                tracing::error!("[{}] Failed to reload: {}", context, e);
            }
        }
        WebViewMessage::StopLoading => {
            if let Err(e) = webview.evaluate_script("window.stop()") {
                tracing::error!("[{}] Failed to stop loading: {}", context, e);
            }
        }
        WebViewMessage::Close => {
            // Close is handled at event loop level, not in message processing
            tracing::debug!(
                "[{}] Close message received (handled at event loop level)",
                context
            );
        }
    }
}

/// Process all messages in a queue using the unified handler
///
/// Returns the number of messages processed.
pub fn process_message_queue(
    webview: &Arc<Mutex<WryWebView>>,
    message_queue: &crate::ipc::MessageQueue,
    context: &str,
) -> usize {
    if let Ok(webview_guard) = webview.lock() {
        let count = message_queue.process_all(|message| {
            tracing::trace!("[{}] processing message: {:?}", context, message);
            process_message(&webview_guard, message, context);
        });

        if count > 0 {
            tracing::debug!("[{}] processed {} messages from queue", context, count);
        } else {
            tracing::trace!("[{}] no messages in queue", context);
        }

        count
    } else {
        tracing::error!("[{}] failed to lock WebView", context);
        0
    }
}

#[cfg(test)]
mod tests {
    // Tests would go here once we have mock WebView support
}
