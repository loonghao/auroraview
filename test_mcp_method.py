"""Test if create_mcp_server method exists."""
from auroraview import WebView as PyWebView

# Get the underlying core WebView
view = PyWebView(title="Test", width=400, height=300)
core_view = view._core

# Check if method exists
if hasattr(core_view, 'create_mcp_server'):
    print("[OK] create_mcp_server exists on _core!")
    # Try calling it
    try:
        from auroraview.mcp import McpConfig
        config = McpConfig()
        config.direct_execution = False  # Route through message queue
        server = core_view.create_mcp_server(config)
        print(f"[OK] Server created: {server}")
        # Check dispatcher
        if hasattr(server, 'has_dispatcher'):
            print(f"[OK] Has dispatcher: {server.has_dispatcher()}")
        else:
            print("[WARN] has_dispatcher method not exposed to Python")
    except Exception as e:
        print(f"[ERROR] Failed to create server: {e}")
        import traceback
        traceback.print_exc()
else:
    print("[ERROR] create_mcp_server does NOT exist on _core!")
    print(f"Available methods: {[m for m in dir(core_view) if not m.startswith('_')]}")

view.close()
print("Done!")
