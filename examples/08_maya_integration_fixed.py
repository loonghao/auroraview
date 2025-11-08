#!/usr/bin/env python
"""
Example 08: Fixed Maya Integration - Non-Blocking WebView

This example demonstrates the CORRECT way to use WebView in Maya without blocking.

The key insight: We need to use Maya's parent window handle to create an embedded
WebView, which doesn't run its own event loop.

Usage in Maya:
    # In Maya Script Editor
    exec(open('examples/08_maya_integration_fixed.py').read())
"""

import logging
import threading
import time

# Setup path to import auroraview
import _setup_path  # noqa: F401

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


def get_maya_main_window_hwnd():
    """Get Maya main window HWND (Windows only)."""
    try:
        import maya.OpenMayaUI as omui
        import shiboken2
        from PySide2 import QtWidgets

        # Get Maya main window as QWidget
        maya_main_window_ptr = omui.MQtUtil.mainWindow()
        maya_main_window = shiboken2.wrapInstance(int(maya_main_window_ptr), QtWidgets.QWidget)

        # Get HWND
        hwnd = maya_main_window.winId()
        logger.info(f"Maya main window HWND: 0x{hwnd:x}")
        return hwnd
    except Exception as e:
        logger.error(f"Failed to get Maya main window HWND: {e}")
        return None


# JavaScript injection script
INJECTION_SCRIPT = """
(function() {
    console.log('[DCC-AI] Injection script starting...');
    
    window.DCCIntegration = {
        initialized: false,
        chatInput: null,
        
        init: function() {
            if (this.initialized) return;
            console.log('[DCC-AI] Initializing DCC integration...');
            
            this.injectUI();
            this.hookChatInterface();
            
            this.initialized = true;
            console.log('[DCC-AI] Initialization complete');
        },
        
        injectUI: function() {
            const toolbar = document.createElement('div');
            toolbar.id = 'dcc-toolbar';
            toolbar.innerHTML = `
                <div style="
                    position: fixed;
                    top: 60px;
                    right: 10px;
                    z-index: 999999;
                    background: white;
                    border-radius: 8px;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
                    padding: 10px;
                    font-family: Arial, sans-serif;
                ">
                    <div style="font-weight: bold; margin-bottom: 8px; color: #333;">
                        🎨 Maya Tools
                    </div>
                    <button id="dcc-get-selection" style="
                        width: 100%;
                        padding: 8px;
                        margin-bottom: 5px;
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white;
                        border: none;
                        border-radius: 4px;
                        cursor: pointer;
                        font-size: 12px;
                    ">📦 Get Selection</button>
                </div>
            `;
            document.body.appendChild(toolbar);
            
            document.getElementById('dcc-get-selection').onclick = () => {
                this.requestSceneInfo();
            };
            
            console.log('[DCC-AI] UI injected');
        },
        
        hookChatInterface: function() {
            const selectors = ['textarea', 'input[type="text"]', '[contenteditable="true"]'];
            
            for (const selector of selectors) {
                const input = document.querySelector(selector);
                if (input) {
                    this.chatInput = input;
                    console.log('[DCC-AI] Found chat input:', selector);
                    break;
                }
            }
        },
        
        requestSceneInfo: function() {
            console.log('[DCC-AI] Requesting scene info...');
            window.dispatchEvent(new CustomEvent('get_scene_info', {
                detail: { timestamp: Date.now() }
            }));
        },
        
        insertText: function(text) {
            if (!this.chatInput) {
                alert('Chat input not found');
                return;
            }
            
            if (this.chatInput.tagName === 'TEXTAREA' || this.chatInput.tagName === 'INPUT') {
                this.chatInput.value = text;
                this.chatInput.dispatchEvent(new Event('input', { bubbles: true }));
            } else {
                this.chatInput.textContent = text;
                this.chatInput.dispatchEvent(new Event('input', { bubbles: true }));
            }
            
            console.log('[DCC-AI] Text inserted');
        }
    };
    
    window.addEventListener('scene_info_response', (event) => {
        console.log('[DCC-AI] Received scene info:', event.detail);
        
        const info = event.detail;
        const text = `Maya Scene Information:\\n\\n` +
            `Selected Objects: ${info.selection_count}\\n\\n` +
            info.selection.map(obj => `- ${obj}`).join('\\n');
        
        window.DCCIntegration.insertText(text);
    });
    
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            setTimeout(() => window.DCCIntegration.init(), 1000);
        });
    } else {
        setTimeout(() => window.DCCIntegration.init(), 1000);
    }
    
    console.log('[DCC-AI] Script loaded, waiting for page ready...');
})();
"""


def inject_script_delayed(webview, script, delay=3.0):
    """Inject JavaScript after a delay."""

    def _inject():
        time.sleep(delay)
        logger.info("Injecting JavaScript...")
        try:
            webview.eval_js(script)
            logger.info("JavaScript injected successfully")
        except Exception as e:
            logger.error(f"Failed to inject JavaScript: {e}")

    thread = threading.Thread(target=_inject, daemon=True)
    thread.start()


def main():
    """Main function for Maya integration."""
    logger.info("=" * 60)
    logger.info("Maya Integration - Fixed Non-Blocking Version")
    logger.info("=" * 60)
    logger.info("")

    # Get Maya main window HWND
    logger.info("Getting Maya main window HWND...")
    parent = get_maya_main_window_hwnd()

    if parent_hwnd is None:
        logger.error("Failed to get Maya main window HWND")
        logger.error("This example requires Maya")
        return None

    # Create WebView with parent window (EMBEDDED MODE)
    logger.info("Creating WebView in embedded mode...")
    webview = WebView.maya(  # Use Maya shortcut!
        title="AI Chat - Maya Integration", width=1200, height=800, debug=True
    )
    logger.info("WebView created")
    logger.info("")

    # Register event handlers
    @webview.on("get_scene_info")
    def handle_get_scene_info(data):
        logger.info("[RECV] Website requested scene information")

        try:
            import maya.cmds as cmds

            selection = cmds.ls(selection=True)
        except:
            selection = ["pCube1", "pSphere1"]  # Fallback for testing

        webview.emit(
            "scene_info_response", {"selection": selection, "selection_count": len(selection)}
        )
        logger.info(f"[SEND] Sent {len(selection)} objects")

    logger.info("Event handlers registered")
    logger.info("")

    # Load URL
    logger.info("Loading AI chat website...")

    # OPTION 1: Load real AI chat website (uncomment to use)
    # webview.load_url("https://knot.woa.com/chat?web_key=1c2a6b4568f24e00a58999c1b7cb0f6e")

    # OPTION 2: Load test page
    test_page = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Maya AI Chat Demo</title>
        <meta charset="UTF-8">
        <style>
            body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #f0f0f0; }
            .container { max-width: 800px; margin: 0 auto; background: white; border-radius: 8px; padding: 20px; }
            h1 { color: #333; }
            .info { background: #e3f2fd; padding: 15px; border-radius: 4px; margin: 20px 0; }
            textarea { width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🎨 Maya AI Chat Integration</h1>
            <div class="info">
                <strong>Instructions:</strong>
                <ul>
                    <li>Look for the "Maya Tools" panel in the top-right corner</li>
                    <li>Click "Get Selection" to insert Maya scene info</li>
                    <li>Maya should remain fully responsive!</li>
                </ul>
            </div>
            <textarea rows="10" placeholder="Type your message here..."></textarea>
        </div>
    </body>
    </html>
    """

    webview.load_html(test_page)
    logger.info("Page loaded")
    logger.info("")

    # Schedule JavaScript injection
    logger.info("Scheduling JavaScript injection (3 seconds delay)...")
    inject_script_delayed(webview, INJECTION_SCRIPT, delay=3.0)
    logger.info("")

    # Show WebView (NON-BLOCKING in embedded mode!)
    logger.info("Showing WebView...")
    webview.show()
    logger.info("WebView opened")
    logger.info("")

    logger.info("=" * 60)
    logger.info("SUCCESS!")
    logger.info("")
    logger.info("✅ Maya should remain fully responsive")
    logger.info("✅ You can continue working in Maya")
    logger.info("✅ WebView runs in its own window")
    logger.info("")
    logger.info("To close: Just close the WebView window")
    logger.info("Maya will exit normally when you close it")
    logger.info("=" * 60)
    logger.info("")

    # IMPORTANT: Store reference in Maya's global namespace
    try:
        import __main__

        __main__.ai_chat_webview = webview
        logger.info("WebView reference stored in __main__.ai_chat_webview")
    except:
        pass

    return webview


if __name__ == "__main__":
    # For testing outside Maya
    logger.warning("This example is designed for Maya")
    logger.warning("Run it inside Maya Script Editor for best results")

    # For standalone testing, we can simulate
    webview = WebView(title="AI Chat - Standalone Test", width=1200, height=800, debug=True)

    webview.load_html("""
    <html>
    <body style="font-family: Arial; padding: 20px;">
        <h1>Standalone Test Mode</h1>
        <p>This example is designed for Maya.</p>
        <p>Run it inside Maya Script Editor for full functionality.</p>
    </body>
    </html>
    """)

    webview.show()

    logger.info("Press Ctrl+C to exit...")
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        logger.info("Exiting...")
