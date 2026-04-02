//! Tests for IPC router

use std::sync::{Arc, Mutex};

use auroraview_dcc::{IpcMessage, IpcResponse, IpcRouter};
use serde_json::json;

#[test]
fn test_ipc_router_new() {
    let router = IpcRouter::new();
    assert!(!router.has_handler("test"));
}

#[test]
fn test_ipc_router_register_and_check() {
    let router = IpcRouter::new();

    router.register("test.echo", |params| params);

    assert!(router.has_handler("test.echo"));
    assert!(!router.has_handler("test.other"));
}

#[test]
fn test_ipc_router_unregister() {
    let router = IpcRouter::new();

    router.register("test.method", |_| json!({}));
    assert!(router.has_handler("test.method"));

    let removed = router.unregister("test.method");
    assert!(removed);
    assert!(!router.has_handler("test.method"));

    // Unregister non-existent
    let removed = router.unregister("nonexistent");
    assert!(!removed);
}

#[test]
fn test_ipc_router_handle_call() {
    let router = IpcRouter::new();

    router.register("test.echo", |params| params);

    let message = json!({
        "type": "call",
        "id": "1",
        "method": "test.echo",
        "params": {"message": "hello"}
    });

    let result = router.handle(&message.to_string());
    assert!(result.is_some());

    let response: IpcResponse = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(response.ok);
    assert_eq!(response.id, "1");
}

#[test]
fn test_ipc_router_method_not_found() {
    let router = IpcRouter::new();

    let message = json!({
        "type": "call",
        "id": "2",
        "method": "nonexistent.method",
        "params": {}
    });

    let result = router.handle(&message.to_string());
    assert!(result.is_some());

    let response: IpcResponse = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(!response.ok);
    assert!(response.error.is_some());
}

#[test]
fn test_ipc_message_parsing() {
    let json_str = r#"{"type":"call","id":"123","method":"test","params":null}"#;
    let msg: IpcMessage = serde_json::from_str(json_str).unwrap();

    assert_eq!(msg.msg_type, "call");
    assert_eq!(msg.id, Some("123".to_string()));
    assert_eq!(msg.method, Some("test".to_string()));
}

#[test]
fn test_ipc_response_ok() {
    let response = IpcResponse::ok("1".to_string(), json!({"result": "success"}));
    assert!(response.ok);
    assert_eq!(response.id, "1");
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_ipc_response_err() {
    let response = IpcResponse::err("2".to_string(), "TestError", "Something went wrong");
    assert!(!response.ok);
    assert_eq!(response.id, "2");
    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.name, "TestError");
    assert_eq!(error.message, "Something went wrong");
}

// === Event listener tests ===

#[test]
fn test_ipc_router_on_event() {
    let router = IpcRouter::new();
    let received = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
    let received_clone = received.clone();

    router.on("scene.saved", move |detail| {
        received_clone.lock().unwrap().push(detail);
    });

    let message = json!({
        "type": "event",
        "event": "scene.saved",
        "detail": {"path": "/scene.mb"}
    });

    let result = router.handle(&message.to_string());
    // Events return None (no response)
    assert!(result.is_none());

    let data = received.lock().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["path"], "/scene.mb");
}

#[test]
fn test_ipc_router_on_event_multiple_listeners() {
    let router = IpcRouter::new();
    let count = Arc::new(Mutex::new(0u32));

    for _ in 0..3 {
        let c = count.clone();
        router.on("tick", move |_| {
            *c.lock().unwrap() += 1;
        });
    }

    let message = json!({
        "type": "event",
        "event": "tick",
        "detail": null
    });

    router.handle(&message.to_string());
    assert_eq!(*count.lock().unwrap(), 3);
}

#[test]
fn test_ipc_router_event_unknown_type_returns_none() {
    let router = IpcRouter::new();

    let message = json!({
        "type": "unknown_type",
        "id": "99"
    });

    let result = router.handle(&message.to_string());
    assert!(result.is_none());
}

#[test]
fn test_ipc_router_invalid_json_returns_none() {
    let router = IpcRouter::new();
    let result = router.handle("not valid json {{");
    assert!(result.is_none());
}

// === invoke tests ===

#[test]
fn test_ipc_router_handle_invoke() {
    let router = IpcRouter::new();
    router.register("plugin.init", |_| json!({"status": "ok"}));

    let message = json!({
        "type": "invoke",
        "id": "5",
        "cmd": "plugin.init",
        "args": {}
    });

    let result = router.handle(&message.to_string());
    assert!(result.is_some());

    let response: IpcResponse = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(response.ok);
    assert_eq!(response.id, "5");
    assert_eq!(response.result.as_ref().unwrap()["status"], "ok");
}

#[test]
fn test_ipc_router_invoke_not_found() {
    let router = IpcRouter::new();

    let message = json!({
        "type": "invoke",
        "id": "6",
        "cmd": "missing.cmd",
        "args": {}
    });

    let result = router.handle(&message.to_string());
    assert!(result.is_some());

    let response: IpcResponse = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(!response.ok);
    assert!(response.error.is_some());
}

// === methods() listing ===

#[test]
fn test_ipc_router_methods_listing() {
    let router = IpcRouter::new();
    router.register("a.foo", |p| p);
    router.register("b.bar", |p| p);

    let mut methods = router.methods();
    methods.sort();
    assert_eq!(methods, vec!["a.foo", "b.bar"]);

    router.unregister("a.foo");
    assert_eq!(router.methods(), vec!["b.bar"]);
}

// === Extended DCC-specific tests ===

#[test]
fn test_ipc_router_default() {
    let router = IpcRouter::default();
    assert!(router.methods().is_empty());
}

#[test]
fn test_ipc_router_call_null_params() {
    let router = IpcRouter::new();
    router.register("tool.apply", |params| {
        if params.is_null() {
            json!({"applied": false})
        } else {
            json!({"applied": true})
        }
    });

    let msg = json!({"type": "call", "id": "a1", "method": "tool.apply", "params": null});
    let result = router.handle(&msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&result).unwrap();
    assert!(resp.ok);
    assert_eq!(resp.result.unwrap()["applied"], false);
}

#[test]
fn test_ipc_router_call_array_params() {
    let router = IpcRouter::new();
    router.register("scene.select", |params| {
        let count = params.as_array().map(|a| a.len()).unwrap_or(0);
        json!({"selected": count})
    });

    let msg = json!({"type": "call", "id": "a2", "method": "scene.select", "params": ["mesh1", "mesh2", "mesh3"]});
    let result = router.handle(&msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&result).unwrap();
    assert!(resp.ok);
    assert_eq!(resp.result.unwrap()["selected"], 3);
}

#[test]
fn test_ipc_router_overwrite_handler() {
    let router = IpcRouter::new();
    router.register("cmd", |_| json!({"version": 1}));
    router.register("cmd", |_| json!({"version": 2})); // overwrite

    let msg = json!({"type": "call", "id": "ow1", "method": "cmd", "params": {}});
    let result = router.handle(&msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&result).unwrap();
    assert_eq!(resp.result.unwrap()["version"], 2);
}

#[test]
fn test_ipc_router_event_no_listeners_no_panic() {
    let router = IpcRouter::new();
    let msg = json!({"type": "event", "event": "unregistered.event", "detail": {"x": 1}});
    let result = router.handle(&msg.to_string());
    assert!(result.is_none()); // no panic, no response
}

#[test]
fn test_ipc_router_event_null_detail() {
    let router = IpcRouter::new();
    let received = Arc::new(Mutex::new(serde_json::Value::Bool(false)));
    let r = received.clone();

    router.on("ping", move |detail| {
        *r.lock().unwrap() = detail;
    });

    let msg = json!({"type": "event", "event": "ping"});
    router.handle(&msg.to_string());

    assert!(received.lock().unwrap().is_null());
}

#[test]
fn test_ipc_response_ok_null_result() {
    let resp = IpcResponse::ok("id1".to_string(), serde_json::Value::Null);
    assert!(resp.ok);
    assert!(resp.result.unwrap().is_null());
}

#[test]
fn test_ipc_response_err_fields() {
    let resp = IpcResponse::err("err1".to_string(), "NotFound", "Resource missing");
    assert!(!resp.ok);
    let e = resp.error.unwrap();
    assert_eq!(e.name, "NotFound");
    assert_eq!(e.message, "Resource missing");
}

#[test]
fn test_ipc_message_missing_optional_fields() {
    // Minimal valid message
    let json_str = r#"{"type":"event"}"#;
    let msg: IpcMessage = serde_json::from_str(json_str).unwrap();
    assert_eq!(msg.msg_type, "event");
    assert!(msg.id.is_none());
    assert!(msg.method.is_none());
}

#[test]
fn test_ipc_router_concurrent_register_and_call() {
    use std::thread;

    let router = Arc::new(IpcRouter::new());
    // Pre-register handlers
    for i in 0..10 {
        let r = router.clone();
        r.register(&format!("dcc.method{i}"), move |_| json!({"index": i}));
    }

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let r = router.clone();
            thread::spawn(move || {
                let msg = json!({
                    "type": "call",
                    "id": format!("c{i}"),
                    "method": format!("dcc.method{i}"),
                    "params": {}
                });
                let result = r.handle(&msg.to_string()).unwrap();
                let resp: IpcResponse = serde_json::from_str(&result).unwrap();
                assert!(resp.ok);
                resp.result.unwrap()["index"].as_u64().unwrap()
            })
        })
        .collect();

    let mut indices: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    indices.sort();
    assert_eq!(indices, (0u64..10).collect::<Vec<_>>());
}

#[test]
fn test_ipc_router_concurrent_events() {
    use std::thread;

    let router = Arc::new(IpcRouter::new());
    let counter = Arc::new(Mutex::new(0u32));

    for _ in 0..5 {
        let c = counter.clone();
        router.on("dcc.frame_tick", move |_| {
            *c.lock().unwrap() += 1;
        });
    }

    // 4 threads each fire the event once
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let r = router.clone();
            thread::spawn(move || {
                let msg =
                    json!({"type": "event", "event": "dcc.frame_tick", "detail": null});
                r.handle(&msg.to_string());
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    // 5 listeners x 4 invocations = 20
    assert_eq!(*counter.lock().unwrap(), 20);
}

#[test]
fn test_ipc_router_dcc_maya_workflow() {
    // Simulate a Maya DCC IPC workflow: select, query, deselect
    let router = IpcRouter::new();
    let selected = Arc::new(Mutex::new(Vec::<String>::new()));

    let sel = selected.clone();
    router.register("maya.select", move |params| {
        let nodes = params["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        *sel.lock().unwrap() = nodes.clone();
        json!({"count": nodes.len()})
    });

    let sel = selected.clone();
    router.register("maya.get_selection", move |_| {
        json!({"nodes": *sel.lock().unwrap()})
    });

    let sel = selected.clone();
    router.register("maya.deselect_all", move |_| {
        sel.lock().unwrap().clear();
        json!({"ok": true})
    });

    // 1. Select nodes
    let msg = json!({"type":"call","id":"m1","method":"maya.select","params":{"nodes":["pCube1","pSphere1"]}});
    let result = router.handle(&msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&result).unwrap();
    assert_eq!(resp.result.unwrap()["count"], 2);

    // 2. Query selection
    let msg = json!({"type":"call","id":"m2","method":"maya.get_selection","params":{}});
    let result = router.handle(&msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&result).unwrap();
    assert_eq!(resp.result.unwrap()["nodes"].as_array().unwrap().len(), 2);

    // 3. Deselect all
    let msg = json!({"type":"call","id":"m3","method":"maya.deselect_all","params":{}});
    router.handle(&msg.to_string()).unwrap();
    assert!(selected.lock().unwrap().is_empty());
}

#[test]
fn test_ipc_router_dcc_houdini_workflow() {
    // Simulate Houdini cook/export workflow
    let router = IpcRouter::new();
    let cooked = Arc::new(Mutex::new(false));

    let c = cooked.clone();
    router.register("hou.cook_node", move |params| {
        let node = params["node"].as_str().unwrap_or("unknown");
        *c.lock().unwrap() = true;
        json!({"node": node, "status": "cooked"})
    });

    let c = cooked.clone();
    router.register("hou.export_bgeo", move |params| {
        let path = params["path"].as_str().unwrap_or("/tmp/out.bgeo");
        if *c.lock().unwrap() {
            json!({"exported": path, "ok": true})
        } else {
            json!({"ok": false, "reason": "not cooked"})
        }
    });

    let cook_msg = json!({"type":"call","id":"h1","method":"hou.cook_node","params":{"node":"/obj/geo1"}});
    let result = router.handle(&cook_msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&result).unwrap();
    assert_eq!(resp.result.as_ref().unwrap()["status"], "cooked");

    let export_msg = json!({"type":"call","id":"h2","method":"hou.export_bgeo","params":{"path":"/tmp/geo.bgeo"}});
    let result = router.handle(&export_msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&result).unwrap();
    assert_eq!(resp.result.unwrap()["ok"], true);
}

#[test]
fn test_ipc_router_blender_addon_workflow() {
    let router = IpcRouter::new();

    router.register("bpy.apply_modifier", |params| {
        let modifier = params["type"].as_str().unwrap_or("unknown");
        json!({"applied": modifier})
    });

    router.register("bpy.render_frame", |params| {
        let frame = params["frame"].as_u64().unwrap_or(1);
        json!({"rendered_frame": frame, "engine": "CYCLES"})
    });

    let mod_msg = json!({"type":"call","id":"b1","method":"bpy.apply_modifier","params":{"type":"SUBSURF"}});
    let result = router.handle(&mod_msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&result).unwrap();
    assert_eq!(resp.result.unwrap()["applied"], "SUBSURF");

    let render_msg = json!({"type":"call","id":"b2","method":"bpy.render_frame","params":{"frame":42}});
    let result = router.handle(&render_msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&result).unwrap();
    assert_eq!(resp.result.as_ref().unwrap()["rendered_frame"], 42);
    assert_eq!(resp.result.unwrap()["engine"], "CYCLES");
}

#[test]
fn test_ipc_router_call_response_echoes_id() {
    let router = IpcRouter::new();
    router.register("echo", |p| p);

    for id in &["abc-123", "uuid-xxxxxxxx", "0", "999999"] {
        let msg = json!({"type":"call","id": id, "method":"echo","params":{"x":1}});
        let result = router.handle(&msg.to_string()).unwrap();
        let resp: IpcResponse = serde_json::from_str(&result).unwrap();
        assert_eq!(&resp.id, id);
    }
}

#[test]
fn test_ipc_router_register_then_unregister_then_call() {
    let router = IpcRouter::new();
    router.register("transient", |_| json!({"ok": true}));
    router.unregister("transient");

    let msg = json!({"type":"call","id":"t1","method":"transient","params":{}});
    let result = router.handle(&msg.to_string()).unwrap();
    let resp: IpcResponse = serde_json::from_str(&result).unwrap();
    assert!(!resp.ok); // handler removed -> not found
}
