# AuroraView Signals

A cross-language signal-slot system with middleware and event bus support.

## Features

- **Type-safe signals**: `Signal<T>` with compile-time type checking
- **Dynamic signals**: `SignalRegistry` for runtime-named signals
- **Event bus**: Unified event distribution with middleware pipeline
- **Middleware**: Logging, filtering, and transformation middleware
- **Event bridges**: Cross-platform event forwarding (WebView, Python, IPC)
- **Python bindings**: PyO3-based Python API with identical interface

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
auroraview-signals = "0.4"
```

## Quick Start

```rust
use auroraview_signals::prelude::*;

// Create a typed signal
let signal: Signal<String> = Signal::new();

// Connect a handler
let conn = signal.connect(|msg| {
    println!("Received: {}", msg);
});

// Emit a value
signal.emit("Hello, World!".to_string());

// Disconnect when done
signal.disconnect(conn);
```

## Dynamic Signals with Registry

```rust
use auroraview_signals::prelude::*;
use serde_json::json;

let registry = SignalRegistry::new();

// Connect to a named signal (auto-created if not exists)
let conn = registry.connect("process:stdout", |data| {
    println!("stdout: {:?}", data);
});

// Emit to the signal
registry.emit("process:stdout", json!({"pid": 123, "data": "hello"}));
```

## Event Bus with Middleware

```rust
use auroraview_signals::prelude::*;

let bus = EventBus::new();

// Add logging middleware
bus.use_middleware(LoggingMiddleware::new(LogLevel::Debug));

// Subscribe to events
let conn = bus.on("app:ready", |data| {
    println!("App ready: {:?}", data);
});

// Emit events (goes through middleware pipeline)
bus.emit("app:ready", serde_json::json!({"version": "1.0"}));
```

## Python Bindings

Enable with the `python` feature:

```toml
[dependencies]
auroraview-signals = { version = "0.4", features = ["python"] }
```

## License

MIT License - see [LICENSE](LICENSE) for details.
