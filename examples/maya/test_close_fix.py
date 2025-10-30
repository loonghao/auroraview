# -*- coding: utf-8 -*-
"""
Test for window close fix with proper message processing.

This test verifies that the embedded window can be properly closed
by processing pending Windows messages after DestroyWindow().
"""

import sys
sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\python')

import maya.cmds as cmds
import maya.OpenMayaUI as omui
from auroraview import WebView
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QWidget

print("=" * 80)
print("🔧 WINDOW CLOSE FIX TEST")
print("=" * 80)
print("This test verifies the fix for embedded window close issue.")
print("The fix adds proper Windows message processing after DestroyWindow().")
print("=" * 80)

# Get Maya main window
def get_maya_main_window():
    """Get Maya main window as QWidget"""
    main_window_ptr = omui.MQtUtil.mainWindow()
    if main_window_ptr is not None:
        return wrapInstance(int(main_window_ptr), QWidget)
    return None

# Get Maya main window HWND
maya_window = get_maya_main_window()
if maya_window is None:
    print("❌ Failed to get Maya main window")
    raise RuntimeError("Cannot get Maya main window")

hwnd = maya_window.winId()
print(f"✅ Got Maya main window HWND: {hwnd}")

# HTML content with close button
html_content = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {
            font-family: Arial, sans-serif;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            margin: 0;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
        }
        .container {
            background: rgba(255, 255, 255, 0.1);
            padding: 30px;
            border-radius: 15px;
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.37);
            text-align: center;
        }
        h1 {
            margin-top: 0;
            font-size: 24px;
        }
        .info {
            margin: 20px 0;
            padding: 15px;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 8px;
            font-size: 14px;
        }
        button {
            padding: 12px 30px;
            font-size: 16px;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            margin: 10px;
            transition: all 0.3s ease;
        }
        .test-btn {
            background: #4CAF50;
            color: white;
        }
        .test-btn:hover {
            background: #45a049;
            transform: translateY(-2px);
            box-shadow: 0 4px 8px rgba(0,0,0,0.2);
        }
        .close-btn {
            background: #f44336;
            color: white;
        }
        .close-btn:hover {
            background: #da190b;
            transform: translateY(-2px);
            box-shadow: 0 4px 8px rgba(0,0,0,0.2);
        }
        #log {
            margin-top: 20px;
            padding: 15px;
            background: rgba(0, 0, 0, 0.3);
            border-radius: 8px;
            max-height: 200px;
            overflow-y: auto;
            text-align: left;
            font-family: 'Courier New', monospace;
            font-size: 12px;
        }
        .log-entry {
            margin: 5px 0;
            padding: 5px;
            border-left: 3px solid #4CAF50;
            padding-left: 10px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔧 Window Close Fix Test</h1>
        <div class="info">
            <p><strong>Fix Applied:</strong> Proper Windows message processing</p>
            <p>After DestroyWindow(), we now process WM_DESTROY and WM_NCDESTROY messages</p>
        </div>
        <div>
            <button class="test-btn" onclick="testEvent()">🧪 Test Event</button>
            <button class="close-btn" onclick="closeWindow()">✕ Close Window</button>
        </div>
        <div id="log"></div>
    </div>

    <script>
        function addLog(message) {
            const log = document.getElementById('log');
            const entry = document.createElement('div');
            entry.className = 'log-entry';
            entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
            log.appendChild(entry);
            log.scrollTop = log.scrollHeight;
        }

        function testEvent() {
            addLog('📤 Sending test event to Python...');
            if (window.auroraview) {
                window.auroraview.send_event('test', {
                    test: 'data',
                    timestamp: Date.now()
                });
                addLog('✅ Test event sent successfully');
            } else {
                addLog('❌ auroraview bridge not available');
            }
        }

        function closeWindow() {
            addLog('🔒 Sending close event to Python...');
            if (window.auroraview) {
                window.auroraview.send_event('close', {
                    source: 'close_button',
                    timestamp: Date.now()
                });
                addLog('✅ Close event sent');
                addLog('⏳ Waiting for window to close...');
            } else {
                addLog('❌ auroraview bridge not available');
            }
        }

        // Log when page loads
        window.addEventListener('DOMContentLoaded', () => {
            addLog('✅ Page loaded successfully');
            addLog('🔌 Event bridge ready');
        });
    </script>
</body>
</html>
"""

print("📄 Loading HTML...")
webview = WebView(
    title="Close Fix Test",
    width=500,
    height=600,
    parent_hwnd=hwnd
)

# Event handlers
def handle_test(data):
    print("=" * 80)
    print(f"📥 [handle_test] Test event received: {data}")
    print("=" * 80)

def handle_close(data):
    print("=" * 80)
    print("🔒 [handle_close] CLOSE EVENT RECEIVED!")
    print(f"🔒 [handle_close] Event data: {data}")
    print("=" * 80)

    # Queue close operation to Maya main thread
    print("🔒 [handle_close] Queueing close operation to Maya main thread...")
    cmds.evalDeferred(_do_close)

def _do_close():
    """Execute close operation on Maya main thread"""
    print("🔒 [_do_close] Starting close operation...")
    print(f"🔒 [_do_close] WebView object: {__main__.test_close_webview}")

    # Close the webview
    print("🔒 [_do_close] Calling webview.close()...")
    __main__.test_close_webview.close()
    print("✅ [_do_close] webview.close() completed")

    # Kill the timer
    print(f"🔒 [_do_close] Killing timer: {__main__.test_close_timer}")
    cmds.scriptJob(kill=__main__.test_close_timer)
    print("✅ [_do_close] Timer killed")

    # Delete the webview reference
    print("🔒 [_do_close] Deleting webview reference...")
    del __main__.test_close_webview
    print("✅ [_do_close] WebView reference deleted")

    print("✅ [_do_close] Close operation completed successfully")
    print("=" * 80)

# Register event handlers using register_callback
webview.register_callback('test', handle_test)
webview.register_callback('close', handle_close)

# Load HTML
webview.load_html(html_content)
print("✅ HTML loaded")

# Show window
print("🪟 Showing window...")
webview.show_async()
print("✅ Window shown")

# Store in __main__ to prevent garbage collection
import __main__
__main__.test_close_webview = webview
print("✅ WebView stored in __main__.test_close_webview")

# Create timer for event processing
print("⏱️ Creating event processing timer...")

def process_webview_events():
    """Process WebView events and check for close signal"""
    try:
        if hasattr(__main__, 'test_close_webview'):
            should_close = __main__.test_close_webview.process_events()

            if should_close:
                print("=" * 80)
                print("[process_webview_events] should_close = True")
                print("[process_webview_events] Window close signal detected!")
                print("[process_webview_events] Cleaning up resources...")
                print("=" * 80)

                # Stop timer
                if hasattr(__main__, 'test_close_timer'):
                    print(f"[process_webview_events] Stopping timer: {__main__.test_close_timer}")
                    cmds.scriptJob(kill=__main__.test_close_timer)
                    del __main__.test_close_timer
                    print("[process_webview_events] Timer stopped")

                # Delete WebView object
                print("[process_webview_events] Deleting WebView object...")
                del __main__.test_close_webview
                print("[process_webview_events] WebView object deleted")
                print("=" * 80)

    except Exception as e:
        print(f"[process_webview_events] Error: {e}")
        import traceback
        traceback.print_exc()

__main__.test_close_timer = cmds.scriptJob(
    event=["idle", process_webview_events],
    protected=True
)
print(f"✅ Event processing timer created (ID: {__main__.test_close_timer})")

print("=" * 80)
print("✅ Test window created!")
print()
print("📋 Instructions:")
print("1. Press F12 to open DevTools")
print("2. Go to Console tab")
print("3. Click '🧪 Test Event' button to verify event system works")
print("4. Click '✕ Close Window' button to test close functionality")
print("5. Watch the logs in:")
print("   - WebView window (on-screen log panel)")
print("   - Browser DevTools Console")
print("   - Maya Script Editor")
print()
print("🔧 Manual cleanup (if needed):")
print("  del __main__.test_close_webview")
print("  cmds.scriptJob(kill=__main__.test_close_timer)")
print("=" * 80)

