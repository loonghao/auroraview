#!/usr/bin/env python
"""
Example 07: Non-Blocking AI Chat Integration for DCC

This example demonstrates how to integrate with AI chat websites in DCC software
without blocking the main thread. This is the CORRECT way to use WebView in DCC.

Key Features:
- Non-blocking WebView (DCC remains responsive)
- JavaScript injection after page loads
- Bidirectional communication
- Safe for Maya, Houdini, and other DCC software

Usage:
    # In DCC Python console (Maya, Houdini, etc.)
    exec(open('examples/07_ai_chat_non_blocking.py').read())
"""

import logging
import sys
import time
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


# Simulated DCC functions
def get_scene_selection():
    """Get selected objects from DCC scene."""
    return [
        {"name": "pCube1", "type": "mesh", "transform": [0, 0, 0]},
        {"name": "pSphere1", "type": "mesh", "transform": [5, 0, 0]},
    ]


def execute_dcc_code(code, language="python"):
    """Execute code in DCC environment."""
    logger.info(f"Executing {language} code in DCC:")
    logger.info(code)
    logger.info("[DEMO] Code execution simulated (not actually executed)")
    return {"status": "success", "message": "Code executed successfully"}


# JavaScript injection script
INJECTION_SCRIPT = """
(function() {
    console.log('[DCC-AI] Injection script starting...');
    
    // Create DCC integration namespace
    window.DCCIntegration = {
        initialized: false,
        chatInput: null,
        
        init: function() {
            if (this.initialized) return;
            console.log('[DCC-AI] Initializing DCC integration...');
            
            this.injectUI();
            this.hookChatInterface();
            this.monitorResponses();
            
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
                        🎨 DCC Tools
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
                    <button id="dcc-execute-code" style="
                        width: 100%;
                        padding: 8px;
                        background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
                        color: white;
                        border: none;
                        border-radius: 4px;
                        cursor: pointer;
                        font-size: 12px;
                    ">▶️ Execute Code</button>
                </div>
            `;
            document.body.appendChild(toolbar);
            
            document.getElementById('dcc-get-selection').onclick = () => {
                this.requestSceneInfo();
            };
            
            document.getElementById('dcc-execute-code').onclick = () => {
                this.extractAndExecuteCode();
            };
            
            console.log('[DCC-AI] UI injected');
        },
        
        hookChatInterface: function() {
            const selectors = [
                'textarea',
                'input[type="text"]',
                '[contenteditable="true"]',
                '.chat-input',
                '#chat-input'
            ];
            
            for (const selector of selectors) {
                const input = document.querySelector(selector);
                if (input) {
                    this.chatInput = input;
                    console.log('[DCC-AI] Found chat input:', selector);
                    
                    input.addEventListener('keydown', (e) => {
                        if (e.key === 'Enter' && !e.shiftKey) {
                            const message = input.value || input.textContent;
                            window.dispatchEvent(new CustomEvent('ai_message_sent', {
                                detail: { message: message }
                            }));
                        }
                    });
                    
                    break;
                }
            }
        },
        
        monitorResponses: function() {
            const observer = new MutationObserver((mutations) => {
                mutations.forEach((mutation) => {
                    mutation.addedNodes.forEach((node) => {
                        if (node.nodeType === 1) {
                            const text = node.textContent || '';
                            if (text.length > 100) {
                                window.dispatchEvent(new CustomEvent('ai_response_received', {
                                    detail: { message: text, element: node }
                                }));
                            }
                        }
                    });
                });
            });
            
            observer.observe(document.body, {
                childList: true,
                subtree: true
            });
            
            console.log('[DCC-AI] Response monitor active');
        },
        
        requestSceneInfo: function() {
            console.log('[DCC-AI] Requesting scene info...');
            window.dispatchEvent(new CustomEvent('get_scene_info', {
                detail: { timestamp: Date.now() }
            }));
        },
        
        extractAndExecuteCode: function() {
            const messages = document.querySelectorAll('[class*="message"], [class*="response"]');
            let lastMessage = null;
            
            for (let i = messages.length - 1; i >= 0; i--) {
                const text = messages[i].textContent;
                if (text.includes('```')) {
                    lastMessage = text;
                    break;
                }
            }
            
            if (!lastMessage) {
                alert('No code block found in recent messages');
                return;
            }
            
            const codeBlockRegex = /```(\\w+)?\\n([\\s\\S]*?)```/g;
            const matches = [...lastMessage.matchAll(codeBlockRegex)];
            
            if (matches.length === 0) {
                alert('No code blocks found');
                return;
            }
            
            matches.forEach((match, index) => {
                const language = match[1] || 'python';
                const code = match[2].trim();
                
                console.log(`[DCC-AI] Executing code block ${index + 1} (${language})`);
                window.dispatchEvent(new CustomEvent('execute_code', {
                    detail: { code: code, language: language }
                }));
            });
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
        const text = `Current DCC Scene Information:\\n\\n` +
            `DCC: ${info.dcc}\\n` +
            `Selected Objects: ${info.selection_count}\\n\\n` +
            info.selection.map(obj => 
                `- ${obj.name} (${obj.type}) at [${obj.transform.join(', ')}]`
            ).join('\\n');
        
        window.DCCIntegration.insertText(text);
    });
    
    window.addEventListener('execution_result', (event) => {
        console.log('[DCC-AI] Execution result:', event.detail);
        
        const result = event.detail;
        if (result.status === 'success') {
            alert('✅ Code executed successfully in DCC!');
        } else {
            alert('❌ Execution failed: ' + result.error);
        }
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
    """Inject JavaScript after a delay to ensure page is loaded."""

    def _inject():
        time.sleep(delay)
        logger.info("Injecting JavaScript...")
        try:
            webview.eval_js(script)
            logger.info("JavaScript injected successfully")
        except Exception as e:
            logger.error(f"Failed to inject JavaScript: {e}")

    import threading

    thread = threading.Thread(target=_inject, daemon=True)
    thread.start()


def main():
    """Main function for non-blocking AI chat integration."""
    logger.info("=" * 60)
    logger.info("Non-Blocking AI Chat Integration for DCC")
    logger.info("=" * 60)
    logger.info("")

    # Create WebView
    logger.info("Creating WebView...")
    webview = WebView(
        title="AI Chat - DCC Integration (Non-Blocking)", width=1200, height=800, debug=True
    )
    logger.info("WebView created")
    logger.info("")

    # Register event handlers BEFORE showing
    @webview.on("get_scene_info")
    def handle_get_scene_info(data):
        logger.info("[RECV] AI requested scene information")
        selection = get_scene_selection()
        webview.emit(
            "scene_info_response",
            {"selection": selection, "selection_count": len(selection), "dcc": "Maya 2024"},
        )
        logger.info(f"[SEND] Sent scene info: {len(selection)} objects")

    @webview.on("execute_code")
    def handle_execute_code(data):
        logger.info("[RECV] AI sent code to execute")
        code = data.get("code", "")
        language = data.get("language", "python")

        try:
            result = execute_dcc_code(code, language)
            webview.emit("execution_result", {"status": "success", "result": result})
            logger.info("[SEND] Execution successful")
        except Exception as e:
            logger.error(f"Execution failed: {e}")
            webview.emit("execution_result", {"status": "error", "error": str(e)})

    logger.info("Event handlers registered")
    logger.info("")

    # Load URL
    logger.info("Loading AI chat website...")

    # OPTION 1: Load real AI chat website (uncomment to use)
    # webview.load_url("https://knot.woa.com/chat?web_key=1c2a6b4568f24e00a58999c1b7cb0f6e")

    # OPTION 2: Load test page for demonstration
    test_page = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>AI Chat Demo</title>
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
            <h1>🤖 AI Chat - DCC Integration Demo</h1>
            <div class="info">
                <strong>Instructions:</strong>
                <ul>
                    <li>Look for the "DCC Tools" panel in the top-right corner</li>
                    <li>Click "Get Selection" to insert DCC scene info</li>
                    <li>Type a message with code blocks (```python ... ```)</li>
                    <li>Click "Execute Code" to run it in DCC</li>
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

    # Schedule JavaScript injection after page loads
    logger.info("Scheduling JavaScript injection (3 seconds delay)...")
    inject_script_delayed(webview, INJECTION_SCRIPT, delay=3.0)
    logger.info("")

    # Show WebView (NON-BLOCKING)
    logger.info("Showing WebView (non-blocking)...")
    webview.show()  # This is already non-blocking in the current implementation
    logger.info("WebView opened in background thread")
    logger.info("")

    logger.info("=" * 60)
    logger.info("SUCCESS! WebView is running in background")
    logger.info("Your DCC software should remain responsive")
    logger.info("")
    logger.info("To use with real AI chat:")
    logger.info("1. Uncomment the webview.load_url() line")
    logger.info("2. Adjust the injection delay if needed")
    logger.info("3. Open DevTools (F12) to see injection logs")
    logger.info("=" * 60)
    logger.info("")

    # IMPORTANT: Keep reference to prevent garbage collection
    # In DCC, you would do: __main__.ai_chat_webview = webview
    return webview


if __name__ == "__main__":
    # For standalone testing
    webview = main()

    # Keep the script running
    logger.info("Press Ctrl+C to exit...")
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        logger.info("Exiting...")
