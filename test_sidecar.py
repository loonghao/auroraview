"""Test MCP Sidecar startup."""
import sys
import os

# Test 1: Check if SidecarBridge is available
print("=" * 50)
print("Test 1: Check SidecarBridge availability")
print("=" * 50)
try:
    from auroraview.core import SidecarBridge
    bridge = SidecarBridge()
    print("✓ SidecarBridge available!")
    print(f"  Channel: {bridge.channel_name}")
    print(f"  Token: {bridge.auth_token[:10]}...")
except ImportError as e:
    print(f"✗ SidecarBridge not available: {e}")
    sys.exit(1)

# Test 2: Check if sidecar binary exists
print("\n" + "=" * 50)
print("Test 2: Check sidecar binary")
print("=" * 50)
from auroraview.mcp.sidecar import McpSidecar
binary_path = McpSidecar.get_binary_path()
if binary_path:
    print(f"✓ Sidecar binary found: {binary_path}")
else:
    print("✗ Sidecar binary not found!")
    sys.exit(1)

# Test 3: Start sidecar and register a tool
print("\n" + "=" * 50)
print("Test 3: Start sidecar with tool")
print("=" * 50)

def echo_handler(args):
    """Echo tool handler."""
    return {"echo": args.get("message", ""), "status": "ok"}

sidecar = McpSidecar(log_level="debug")
sidecar.register_tool("echo", "Echo back input message", echo_handler)

try:
    port = sidecar.start()
    print(f"✓ Sidecar started on port {port}")
    print(f"  IPC Channel: {sidecar.channel_name}")
    print(f"  Is alive: {sidecar.is_alive()}")
    
    # Keep running briefly
    import time
    print("\nWaiting 3 seconds...")
    time.sleep(3)
    
    print(f"  Still alive: {sidecar.is_alive()}")
    
finally:
    print("\nStopping sidecar...")
    sidecar.stop()
    print("✓ Sidecar stopped")

print("\n" + "=" * 50)
print("All tests passed!")
print("=" * 50)
