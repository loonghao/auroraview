# IPC Performance Metrics

AuroraView provides built-in performance monitoring for IPC (Inter-Process Communication) between Python and JavaScript.

## Overview

The IPC metrics system tracks:
- **Message throughput**: Total messages sent, failed, and dropped
- **Success rate**: Percentage of successfully delivered messages
- **Latency**: Average message delivery time in microseconds
- **Queue health**: Peak queue length and retry attempts

## Python API

### Getting Metrics

```python
from auroraview import WebView

# Create WebView
webview = WebView(
    title="My App",
    width=800,
    height=600,
    html="<h1>Hello World</h1>"
)

# Show the window
webview.show()

# Send some messages
webview.eval_js("console.log('test')")
webview.emit_event("my_event", {"data": "value"})

# Get metrics snapshot
metrics = webview.get_ipc_metrics()

# Access individual metrics
print(f"Messages sent: {metrics.messages_sent}")
print(f"Messages failed: {metrics.messages_failed}")
print(f"Messages dropped: {metrics.messages_dropped}")
print(f"Success rate: {metrics.success_rate}%")
print(f"Average latency: {metrics.avg_latency_us}μs")
print(f"Peak queue length: {metrics.peak_queue_length}")
print(f"Messages received: {metrics.messages_received}")
print(f"Retry attempts: {metrics.retry_attempts}")

# Or use the formatted output
print(metrics.format())
```

### Output Example

```
IPC Metrics:
- Messages Sent: 1523
- Messages Failed: 2
- Messages Dropped: 0
- Retry Attempts: 2
- Success Rate: 99.87%
- Avg Latency: 145μs
- Peak Queue Length: 12
- Messages Received: 1523
```

### Resetting Metrics

```python
# Reset all metrics to zero
webview.reset_ipc_metrics()

# Perform operations
for i in range(1000):
    webview.eval_js(f"console.log({i})")

# Get metrics for just these operations
metrics = webview.get_ipc_metrics()
print(f"Sent {metrics.messages_sent} messages")
```

## Use Cases

### 1. Performance Monitoring

```python
import time

webview = WebView(...)
webview.show()

# Baseline metrics
start_metrics = webview.get_ipc_metrics()

# Perform heavy operations
start_time = time.time()
for i in range(10000):
    webview.eval_js(f"document.title = 'Count: {i}'")

elapsed = time.time() - start_time
end_metrics = webview.get_ipc_metrics()

# Calculate throughput
messages_sent = end_metrics.messages_sent - start_metrics.messages_sent
throughput = messages_sent / elapsed

print(f"Throughput: {throughput:.2f} messages/second")
print(f"Average latency: {end_metrics.avg_latency_us}μs")
```

### 2. Debugging Message Failures

```python
webview = WebView(...)
webview.show()

# ... application logic ...

metrics = webview.get_ipc_metrics()

if metrics.messages_failed > 0:
    print(f"⚠️ Warning: {metrics.messages_failed} messages failed!")
    print(f"Success rate: {metrics.success_rate}%")
    print(f"Retry attempts: {metrics.retry_attempts}")

if metrics.messages_dropped > 0:
    print(f"❌ Error: {metrics.messages_dropped} messages dropped!")
    print("Consider increasing queue capacity or reducing message rate")
```

### 3. Load Testing

```python
def stress_test(webview, num_messages=10000):
    """Stress test IPC performance"""
    webview.reset_ipc_metrics()
    
    start = time.time()
    for i in range(num_messages):
        webview.eval_js(f"console.log({i})")
    
    elapsed = time.time() - start
    metrics = webview.get_ipc_metrics()
    
    return {
        "throughput": metrics.messages_sent / elapsed,
        "success_rate": metrics.success_rate,
        "avg_latency_us": metrics.avg_latency_us,
        "peak_queue": metrics.peak_queue_length,
        "failures": metrics.messages_failed,
        "drops": metrics.messages_dropped,
    }

# Run test
results = stress_test(webview, 10000)
print(f"Throughput: {results['throughput']:.2f} msg/s")
print(f"Success rate: {results['success_rate']:.2f}%")
print(f"Avg latency: {results['avg_latency_us']}μs")
```

## Metrics Reference

| Metric | Type | Description |
|--------|------|-------------|
| `messages_sent` | `int` | Total messages successfully sent to WebView |
| `messages_failed` | `int` | Total messages that failed to send |
| `messages_dropped` | `int` | Messages dropped due to queue overflow |
| `retry_attempts` | `int` | Total number of retry attempts |
| `avg_latency_us` | `int` | Average message latency in microseconds |
| `peak_queue_length` | `int` | Maximum queue length observed |
| `messages_received` | `int` | Total messages received from WebView |
| `success_rate` | `float` | Percentage of successful sends (0-100) |

## Notes

- Metrics are tracked per WebView instance
- All counters are atomic and thread-safe
- Latency is measured from send to delivery
- `reset_ipc_metrics()` is currently a placeholder (not yet implemented)

