//! wry drag-drop -> IpcHandler bridge.
//!
//! High-level helper used by the entries that already own an [`IpcHandler`]
//! (standalone desktop / native backend). Other entries (tab manager, desktop
//! runtime, browser) drive the low-level helper directly because they route
//! events through their own channels.

use std::sync::Arc;

use crate::ipc::{IpcHandler, IpcMessage};

/// Conditionally install the default file-drop handler on a wry builder and
/// route the resulting events into the given [`IpcHandler`].
///
/// `use_default_file_drop` is opt-out (RFC 0013 revised):
/// - `true`  → keep the browser-native drag-drop behavior; do **not** emit
///   `file_drop_*` IPC events.
/// - `false` → install the wry handler and forward Enter/Drop/Leave to the
///   IpcHandler as `file_drop_hover` / `file_drop` / `file_drop_cancelled`.
pub fn install_default_file_drop_to_ipc<'a>(
    builder: wry::WebViewBuilder<'a>,
    ipc_handler: Arc<IpcHandler>,
    use_default_file_drop: bool,
    log_tag: &'static str,
) -> wry::WebViewBuilder<'a> {
    auroraview_core::builder::install_default_file_drop_with(
        builder,
        use_default_file_drop,
        log_tag,
        move |event_name, data| {
            let msg = IpcMessage {
                event: event_name.to_string(),
                data,
                id: None,
            };
            if let Err(e) = ipc_handler.handle_message(msg) {
                tracing::error!("[{}] Error handling {}: {}", log_tag, event_name, e);
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Build the closure body that `install_default_file_drop_to_ipc` registers
    /// internally, without driving wry. This lets us assert the IPC routing
    /// shape (event name + JSON payload) for each of the three drag-drop slugs.
    fn forward<F>(event_name: &str, data: serde_json::Value, ipc_handler: Arc<IpcHandler>, sink: F)
    where
        F: Fn(IpcMessage),
    {
        let msg = IpcMessage {
            event: event_name.to_string(),
            data,
            id: None,
        };
        sink(msg.clone());
        let _ = ipc_handler.handle_message(msg);
    }

    #[test]
    fn forwards_file_drop_hover_to_ipc() {
        let captured: Arc<Mutex<Vec<IpcMessage>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured.clone();
        let ipc = Arc::new(IpcHandler::new());

        let payload = serde_json::json!({
            "hovering": true,
            "paths": ["a.txt"],
            "position": {"x": 10.0, "y": 20.0}
        });
        forward("file_drop_hover", payload.clone(), ipc, move |m| {
            captured_clone.lock().unwrap().push(m);
        });

        let msgs = captured.lock().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].event, "file_drop_hover");
        assert_eq!(msgs[0].data, payload);
        assert!(msgs[0].id.is_none());
    }

    #[test]
    fn forwards_file_drop_to_ipc() {
        let captured: Arc<Mutex<Vec<IpcMessage>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured.clone();
        let ipc = Arc::new(IpcHandler::new());

        let payload = serde_json::json!({
            "paths": ["a.txt", "b.png"],
            "position": {"x": 1.0, "y": 2.0},
            "timestamp": 12345
        });
        forward("file_drop", payload.clone(), ipc, move |m| {
            captured_clone.lock().unwrap().push(m);
        });

        let msgs = captured.lock().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].event, "file_drop");
        assert_eq!(msgs[0].data["paths"].as_array().unwrap().len(), 2);
        assert_eq!(msgs[0].data["timestamp"], 12345);
    }

    #[test]
    fn forwards_file_drop_cancelled_to_ipc() {
        let captured: Arc<Mutex<Vec<IpcMessage>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured.clone();
        let ipc = Arc::new(IpcHandler::new());

        let payload = serde_json::json!({
            "hovering": false,
            "reason": "left_window"
        });
        forward("file_drop_cancelled", payload.clone(), ipc, move |m| {
            captured_clone.lock().unwrap().push(m);
        });

        let msgs = captured.lock().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].event, "file_drop_cancelled");
        assert_eq!(msgs[0].data["reason"], "left_window");
    }

    #[test]
    fn ipc_handler_receives_three_distinct_event_names() {
        let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let ipc = Arc::new(IpcHandler::new());
        let captured_clone = captured.clone();

        for name in &["file_drop_hover", "file_drop", "file_drop_cancelled"] {
            let captured_iter = captured_clone.clone();
            forward(
                name,
                serde_json::json!({}),
                ipc.clone(),
                move |m: IpcMessage| {
                    captured_iter.lock().unwrap().push(m.event);
                },
            );
        }

        let names = captured.lock().unwrap();
        assert_eq!(
            *names,
            vec![
                "file_drop_hover".to_string(),
                "file_drop".to_string(),
                "file_drop_cancelled".to_string(),
            ]
        );
    }

    /// End-to-end shape: register a Rust callback on the IpcHandler for each
    /// slug and verify that the production-style closure routes events to it.
    #[test]
    fn registered_rust_callback_receives_routed_events() {
        let ipc = Arc::new(IpcHandler::new());
        let hover_hits: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let drop_hits: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let cancel_hits: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

        let h = hover_hits.clone();
        ipc.on("file_drop_hover", move |_msg| {
            *h.lock().unwrap() += 1;
            Ok(serde_json::json!({"ok": true}))
        });
        let d = drop_hits.clone();
        ipc.on("file_drop", move |_msg| {
            *d.lock().unwrap() += 1;
            Ok(serde_json::json!({"ok": true}))
        });
        let c = cancel_hits.clone();
        ipc.on("file_drop_cancelled", move |_msg| {
            *c.lock().unwrap() += 1;
            Ok(serde_json::json!({"ok": true}))
        });

        // Mirror the closure that install_default_file_drop_to_ipc installs.
        let route = |event_name: &str, data: serde_json::Value, ipc: Arc<IpcHandler>| {
            let msg = IpcMessage {
                event: event_name.to_string(),
                data,
                id: None,
            };
            let _ = ipc.handle_message(msg);
        };

        route("file_drop_hover", serde_json::json!({}), ipc.clone());
        route("file_drop", serde_json::json!({}), ipc.clone());
        route("file_drop_cancelled", serde_json::json!({}), ipc.clone());

        assert_eq!(*hover_hits.lock().unwrap(), 1);
        assert_eq!(*drop_hits.lock().unwrap(), 1);
        assert_eq!(*cancel_hits.lock().unwrap(), 1);
    }
}
