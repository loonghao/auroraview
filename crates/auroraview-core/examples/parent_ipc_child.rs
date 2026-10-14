//! Headless AuroraView child process.
//!
//! A minimal, dependency-free **child** used to prove the parent/child IPC
//! protocol is language-agnostic: it opens no window, so it runs on a build
//! agent, and it is driven entirely by the `AURORAVIEW_*` environment and by
//! messages from whichever parent is on the other end of the socket.
//!
//! # Environment
//!
//! | Variable | Required | Meaning |
//! |---|---|---|
//! | `AURORAVIEW_PARENT_ID` | yes | puts the process in child mode |
//! | `AURORAVIEW_PARENT_PORT` | yes | TCP port of the parent |
//! | `AURORAVIEW_CHILD_ID` | no | reported as `child_id` on every frame |
//! | `AURORAVIEW_EXAMPLE_NAME` | no | reported in `child:hello` |
//! | `AURORAVIEW_CHILD_EXIT_ON_DISCONNECT` | no | exit when the parent drops the link |
//!
//! # Dialogue
//!
//! ```text
//! child -> parent  {"type":"hello",...}
//! child -> parent  {"type":"event","event":"child:ready",...}
//! child -> parent  {"type":"event","event":"child:hello","data":{"pid":...}}
//! parent -> child  {"type":"hello_ack","accepted":true}
//! parent -> child  {"type":"event","event":"parent:ping","data":{...}}
//! child -> parent  {"type":"event","event":"child:pong","data":{"echo":{...}}}
//! parent -> child  {"type":"event","event":"parent:command","data":{"command":"close"}}
//! child -> parent  {"type":"event","event":"child:closing",...}
//! ```
//!
//! Every received frame is echoed to stdout as `TAG <json>` so a parent script
//! can assert on it. `examples/parent_ipc/parent_host.ps1` is the matching
//! non-Python parent.
//!
//! Run it with:
//!
//! ```text
//! cargo run -p auroraview-core --example parent_ipc_child
//! ```

use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use auroraview_core::parent_ipc::{ChildInfo, HandshakeState, ParentBridge};
use serde_json::json;

/// How long to wait for the parent before giving up.
const RUN_TIMEOUT: Duration = Duration::from_secs(30);
/// How long to wait for `hello_ack`.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

fn main() {
    let info = ChildInfo::from_env();
    if !info.can_connect() {
        eprintln!(
            "parent_ipc_child: set AURORAVIEW_PARENT_ID and AURORAVIEW_PARENT_PORT before starting"
        );
        process::exit(2);
    }

    let bridge = match ParentBridge::connect(&info) {
        Ok(bridge) => bridge,
        Err(e) => {
            eprintln!("CONNECT_ERROR {}", e);
            process::exit(3);
        }
    };

    let state = bridge.wait_for_handshake(HANDSHAKE_TIMEOUT);
    // `as_str()` keeps the log machine-readable for parent-side assertions.
    println!("HANDSHAKE {}", state.as_str());
    if state == HandshakeState::Rejected {
        eprintln!("parent rejected this child");
        bridge.disconnect();
        process::exit(1);
    }

    if let Err(e) = bridge.send_event(
        "child:hello",
        json!({
            "pid": process::id(),
            "example_name": info.example_name,
            "handshake": state.as_str(),
        }),
    ) {
        eprintln!("SEND_ERROR {}", e);
    }

    let stop = Arc::new(AtomicBool::new(false));

    // Answer liveness probes. The handle is kept alive for the process
    // lifetime so the handler stays registered.
    let pong = bridge.clone();
    let _ping_handle = bridge.on_event("parent:ping", move |data| {
        println!("PING {}", data);
        if let Err(e) = pong.send_event("child:pong", json!({ "echo": data })) {
            eprintln!("SEND_ERROR {}", e);
        }
    });

    // Accept the standard close command from the parent.
    let stop_for_command = Arc::clone(&stop);
    let _command_handle = bridge.on_command(move |data| {
        println!("COMMAND {}", data);
        if data.get("command").and_then(|v| v.as_str()) == Some("close") {
            println!("STOP reason=close");
            stop_for_command.store(true, Ordering::SeqCst);
        }
    });

    // Optional: treat a dropped link as "the parent is gone, exit".
    if ChildInfo::from_env().parent_id.is_some()
        && std::env::var("AURORAVIEW_CHILD_EXIT_ON_DISCONNECT")
            .map(|value| auroraview_core::constants::parse_truthy(&value).unwrap_or(false))
            .unwrap_or(false)
    {
        let stop_for_disconnect = Arc::clone(&stop);
        bridge.on_disconnect(move || {
            println!("DISCONNECTED");
            stop_for_disconnect.store(true, Ordering::SeqCst);
        });
    }

    let deadline = Instant::now() + RUN_TIMEOUT;
    while !stop.load(Ordering::SeqCst) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(25));
    }

    bridge.disconnect();
    println!("EXIT");
}
