#!/usr/bin/env python
"""Debug script to diagnose HWND mode issues.

This script creates a minimal HWND mode test to verify:
1. WebView creation in background thread
2. API binding and IPC communication
3. JavaScript -> Python call flow
4. Python -> JavaScript result return

Usage:
    python scripts/debug_hwnd_mode.py
"""
import logging
import threading
import time

# Setup logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s [%(levelname)s] %(name)s: %(message)s'
)
logger = logging.getLogger(__name__)


def inject_api_methods_js(webview, api_methods: "list[str]") -> None:
    """Inject API method wrappers into JavaScript."""
    methods_js = ", ".join(f"'{m}'" for m in api_methods)
    js_code = f"""
    (function() {{
        console.log('[Debug] Injecting API methods...');

        function tryRegister() {{
            if (window.auroraview && window.auroraview._registerApiMethods) {{
                window.auroraview._registerApiMethods('api', [{methods_js}]);
                console.log('[Debug] API methods registered: {len(api_methods)} methods');
                return true;
            }}
            return false;
        }}

        if (!tryRegister()) {{
            var attempts = 0;
            var interval = setInterval(function() {{
                attempts++;
                if (tryRegister() || attempts > 50) {{
                    clearInterval(interval);
                    if (attempts > 50) {{
                        console.error('[Debug] Failed to register API methods after 50 attempts');
                    }}
                }}
            }}, 100);
        }}
    }})();
    """
    webview.eval_js(js_code)


def test_hwnd_mode():
    """Test HWND mode with a simple API."""
    from auroraview import WebView

    # Simple test API
    class TestAPI:
        def __init__(self):
            self.call_count = 0

        def get_config(self, **kwargs):
            """Return test config."""
            self.call_count += 1
            logger.info(f"get_config called! (count: {self.call_count})")
            return {
                "shelves": [
                    {
                        "name": "Test Shelf",
                        "category": "Test",
                        "buttons": [
                            {
                                "id": "test_button",
                                "name": "Test Button",
                                "label": "Test",
                                "tooltip": "A test button",
                            }
                        ],
                    }
                ],
                "banner": {
                    "title": "HWND Mode Test",
                    "subtitle": "Testing IPC",
                    "enabled": True,
                },
                "currentHost": "standalone",
            }

        def launch_tool(self, button_id: str = "", **kwargs):
            """Test tool launch."""
            logger.info(f"launch_tool called with button_id: {button_id}")
            return {"success": True, "message": f"Launched: {button_id}"}

    # Ready event
    ready_event = threading.Event()
    api = TestAPI()

    def run_webview():
        """Run WebView in background thread."""
        try:
            logger.info("Creating WebView...")
            webview = WebView(
                title="HWND Mode Debug",
                width=800,
                height=600,
                debug=True,
                context_menu=True,
            )

            # Bind API
            logger.info("Binding API...")
            webview.bind_api(api, namespace="api")
            logger.info("API bound successfully")

            # Load test HTML with debug output
            html = """<!DOCTYPE html>
<html>
<head><title>HWND Debug</title>
<style>
body { font-family: Arial; padding: 20px; background: #1a1a2e; color: #eee; }
#log { background: #0f0f23; padding: 10px; height: 300px; overflow-y: auto; 
       font-family: monospace; font-size: 12px; border-radius: 5px; }
.log-entry { margin: 2px 0; padding: 2px 5px; }
.success { color: #4ade80; }
.error { color: #f87171; }
.info { color: #60a5fa; }
</style></head>
<body>
<h1>HWND Mode Debug</h1>
<div id="status">Checking AuroraView...</div>
<button onclick="testGetConfig()">Test get_config()</button>
<button onclick="testLaunchTool()">Test launch_tool()</button>
<h3>Log:</h3>
<div id="log"></div>
<script>
function log(msg, type = 'info') {
    const logDiv = document.getElementById('log');
    const entry = document.createElement('div');
    entry.className = 'log-entry ' + type;
    entry.textContent = new Date().toISOString() + ' | ' + msg;
    logDiv.appendChild(entry);
    logDiv.scrollTop = logDiv.scrollHeight;
    console.log('[HWND Debug]', msg);
}

function checkAuroraView() {
    log('Checking window.auroraview...');
    if (window.auroraview) {
        log('window.auroraview exists!', 'success');
        log('  .call: ' + typeof window.auroraview.call);
        log('  .api: ' + JSON.stringify(Object.keys(window.auroraview.api || {})));
        document.getElementById('status').textContent = 'AuroraView Ready!';
        document.getElementById('status').style.color = '#4ade80';
        return true;
    } else {
        log('window.auroraview not found', 'error');
        return false;
    }
}

async function testGetConfig() {
    log('Calling window.auroraview.api.get_config()...');
    try {
        const result = await window.auroraview.api.get_config();
        log('Result: ' + JSON.stringify(result), 'success');
    } catch (e) {
        log('Error: ' + e.message, 'error');
    }
}

async function testLaunchTool() {
    log('Calling window.auroraview.api.launch_tool({button_id: "test"})...');
    try {
        const result = await window.auroraview.api.launch_tool({button_id: 'test'});
        log('Result: ' + JSON.stringify(result), 'success');
    } catch (e) {
        log('Error: ' + e.message, 'error');
    }
}

// Poll for AuroraView
let attempts = 0;
const pollInterval = setInterval(() => {
    attempts++;
    if (checkAuroraView() || attempts > 50) {
        clearInterval(pollInterval);
        if (attempts > 50) log('Timeout waiting for AuroraView', 'error');
        else testGetConfig();  // Auto-test on ready
    }
}, 100);
</script>
</body></html>"""
            
            webview.load_html(html)

            # Inject API methods after a short delay
            def delayed_injection():
                time.sleep(0.5)
                logger.info("Injecting API methods...")
                inject_api_methods_js(webview, ["get_config", "launch_tool"])
                logger.info("API methods injected")

            injection_thread = threading.Thread(target=delayed_injection, daemon=True)
            injection_thread.start()

            ready_event.set()
            logger.info("Starting WebView event loop (blocking)...")
            webview.show_blocking()
            logger.info("WebView closed")

        except Exception as e:
            logger.error(f"Error in WebView thread: {e}", exc_info=True)
            ready_event.set()

    # Start background thread
    thread = threading.Thread(target=run_webview, name="HWND-Debug", daemon=True)
    thread.start()
    
    # Wait for ready
    logger.info("Main thread waiting for WebView...")
    if ready_event.wait(timeout=30):
        logger.info("WebView ready! Main thread is free.")
    else:
        logger.error("Timeout waiting for WebView")
        return
    
    # Keep main thread alive
    logger.info("Press Ctrl+C to exit...")
    try:
        while thread.is_alive():
            time.sleep(0.5)
    except KeyboardInterrupt:
        logger.info("Interrupted, exiting...")


if __name__ == "__main__":
    test_hwnd_mode()

