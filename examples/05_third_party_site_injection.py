#!/usr/bin/env python
"""
Example 05: Third-Party Website JavaScript Injection

This example demonstrates how to inject JavaScript into third-party websites
and establish bidirectional communication between the website and Python/DCC.

This is useful for integrating with AI chat websites, web-based tools, etc.
where you don't control the source code but want to interact with them.

Features:
- JavaScript injection into third-party sites
- Intercept and modify website behavior
- Send data from DCC to website
- Receive data from website to DCC
- Hook into website's JavaScript functions

Usage:
    python examples/05_third_party_site_injection.py
"""

import logging
import sys
from datetime import datetime
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


def main():
    """Main function demonstrating third-party site JavaScript injection."""
    logger.info("=" * 60)
    logger.info("AuroraView - Example 05: Third-Party Site Injection")
    logger.info("=" * 60)
    logger.info("")

    # Create WebView instance
    logger.info("Creating WebView instance...")
    webview = WebView(
        title="AuroraView - Third-Party Site Integration",
        width=1200,
        height=900,
        debug=True,  # Enable DevTools for debugging
    )
    logger.info("[OK] WebView created")
    logger.info("")

    # Register event handlers for communication with the website
    logger.info("Registering event handlers...")

    @webview.on("dcc_get_selection")
    def handle_get_selection(data):
        """Handle request for DCC selection from website."""
        logger.info(f"[RECV] Website requested DCC selection: {data}")

        # Simulate getting selection from DCC (Maya, Houdini, etc.)
        selection_data = {
            "objects": [
                {"name": "pCube1", "type": "mesh", "vertices": 8},
                {"name": "pSphere1", "type": "mesh", "vertices": 382},
                {"name": "camera1", "type": "camera", "fov": 45.0},
            ],
            "count": 3,
            "timestamp": datetime.now().isoformat(),
        }

        # Send selection data back to website
        webview.emit("dcc_selection_result", selection_data)
        logger.info(f"[SEND] Sent selection data: {len(selection_data['objects'])} objects")

    @webview.on("dcc_execute_code")
    def handle_execute_code(data):
        """Handle code execution request from website (e.g., AI-generated code)."""
        logger.info("[RECV] Website sent code to execute:")
        code = data.get("code", "")
        language = data.get("language", "python")

        logger.info(f"Language: {language}")
        logger.info(f"Code:\n{code}")

        # In a real DCC integration, you would execute this code
        # For example, in Maya:
        # import maya.cmds as cmds
        # exec(code)

        # Simulate execution result
        result = {
            "status": "success",
            "message": f"Executed {len(code)} characters of {language} code",
            "output": "Created 3 objects successfully",
            "timestamp": datetime.now().isoformat(),
        }

        # Send result back to website
        webview.emit("dcc_execution_result", result)
        logger.info(f"[SEND] Execution result: {result['status']}")

    @webview.on("ai_response_intercepted")
    def handle_ai_response(data):
        """Handle intercepted AI responses from the website."""
        logger.info("[RECV] Intercepted AI response:")
        logger.info(f"Message: {data.get('message', '')[:100]}...")

        # You can process the AI response here
        # For example, extract code blocks, parse commands, etc.

    logger.info("[OK] Event handlers registered")
    logger.info("")

    # JavaScript injection code
    # This will be injected into the third-party website
    injection_script = """
    (function() {
        console.log('[AuroraView] Injection script loaded');
        
        // Create a namespace for our injected code
        window.AuroraViewDCC = {
            // Send DCC selection to website
            sendSelection: function() {
                console.log('[AuroraView] Requesting DCC selection...');
                window.dispatchEvent(new CustomEvent('dcc_get_selection', {
                    detail: { timestamp: Date.now() }
                }));
            },
            
            // Send code to DCC for execution
            executeInDCC: function(code, language = 'python') {
                console.log('[AuroraView] Sending code to DCC:', code);
                window.dispatchEvent(new CustomEvent('dcc_execute_code', {
                    detail: { code: code, language: language }
                }));
            },
            
            // Inject a button into the page
            injectButton: function() {
                const button = document.createElement('button');
                button.textContent = '🔗 Get DCC Selection';
                button.style.cssText = `
                    position: fixed;
                    top: 10px;
                    right: 10px;
                    z-index: 999999;
                    padding: 10px 20px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                    border: none;
                    border-radius: 6px;
                    font-weight: bold;
                    cursor: pointer;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                    font-size: 14px;
                `;
                button.onclick = () => this.sendSelection();
                document.body.appendChild(button);
                console.log('[AuroraView] Button injected');
            },
            
            // Hook into website's input/output
            hookChatInput: function() {
                // Try to find common chat input selectors
                const selectors = [
                    'textarea[placeholder*="message"]',
                    'textarea[placeholder*="Message"]',
                    'input[type="text"][placeholder*="message"]',
                    'textarea',
                    '.chat-input textarea',
                    '#chat-input'
                ];
                
                for (const selector of selectors) {
                    const input = document.querySelector(selector);
                    if (input) {
                        console.log('[AuroraView] Found chat input:', selector);
                        
                        // Add a helper function to insert text
                        this.chatInput = input;
                        this.insertText = function(text) {
                            input.value = text;
                            input.dispatchEvent(new Event('input', { bubbles: true }));
                            input.dispatchEvent(new Event('change', { bubbles: true }));
                            console.log('[AuroraView] Text inserted into chat');
                        };
                        
                        return true;
                    }
                }
                
                console.warn('[AuroraView] Could not find chat input');
                return false;
            },
            
            // Monitor for AI responses
            monitorResponses: function() {
                // Use MutationObserver to watch for new messages
                const observer = new MutationObserver((mutations) => {
                    mutations.forEach((mutation) => {
                        mutation.addedNodes.forEach((node) => {
                            if (node.nodeType === 1) { // Element node
                                // Look for message-like elements
                                const text = node.textContent || '';
                                if (text.length > 50) { // Likely a message
                                    console.log('[AuroraView] Detected new content');
                                    window.dispatchEvent(new CustomEvent('ai_response_intercepted', {
                                        detail: { message: text }
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
                
                console.log('[AuroraView] Response monitor started');
            }
        };
        
        // Listen for responses from DCC
        window.addEventListener('dcc_selection_result', (event) => {
            console.log('[AuroraView] Received DCC selection:', event.detail);
            
            // Format the selection data
            const objects = event.detail.objects || [];
            const text = `DCC Selection (${objects.length} objects):\\n` +
                objects.map(obj => `- ${obj.name} (${obj.type})`).join('\\n');
            
            // Try to insert into chat input
            if (window.AuroraViewDCC.chatInput) {
                window.AuroraViewDCC.insertText(text);
            } else {
                alert(text);
            }
        });
        
        window.addEventListener('dcc_execution_result', (event) => {
            console.log('[AuroraView] Code execution result:', event.detail);
            alert(`DCC Execution: ${event.detail.status}\\n${event.detail.message}`);
        });
        
        // Auto-initialize
        setTimeout(() => {
            window.AuroraViewDCC.injectButton();
            window.AuroraViewDCC.hookChatInput();
            window.AuroraViewDCC.monitorResponses();
            console.log('[AuroraView] Initialization complete');
        }, 1000);
        
    })();
    """

    logger.info("Loading test page (simulating third-party site)...")

    # For testing, we'll load a local HTML page that simulates a third-party site
    # In production, you would use: webview.load_url("https://third-party-site.com")
    test_html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Simulated Third-Party Site</title>
        <meta charset="UTF-8">
        <style>
            * { margin: 0; padding: 0; box-sizing: border-box; }
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: #f5f5f5;
                padding: 20px;
            }
            .container {
                max-width: 800px;
                margin: 0 auto;
                background: white;
                border-radius: 12px;
                padding: 30px;
                box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            }
            h1 { color: #333; margin-bottom: 20px; }
            .chat-area {
                background: #f9f9f9;
                border-radius: 8px;
                padding: 20px;
                min-height: 300px;
                margin-bottom: 20px;
            }
            .chat-input {
                width: 100%;
                padding: 15px;
                border: 2px solid #ddd;
                border-radius: 8px;
                font-size: 14px;
                resize: vertical;
            }
            .info {
                background: #e3f2fd;
                border-left: 4px solid #2196f3;
                padding: 15px;
                margin-top: 20px;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🌐 Simulated Third-Party Website</h1>
            <p style="color: #666; margin-bottom: 20px;">
                This simulates a third-party website (like an AI chat) where we inject JavaScript.
            </p>
            
            <div class="chat-area" id="messages">
                <p><strong>System:</strong> Welcome! This is a simulated chat interface.</p>
            </div>
            
            <textarea 
                class="chat-input" 
                placeholder="Type your message here..."
                rows="3"
            ></textarea>
            
            <div class="info">
                <strong>🔧 Injected Features:</strong>
                <ul style="margin-top: 10px; margin-left: 20px;">
                    <li>Button in top-right corner to get DCC selection</li>
                    <li>Automatic chat input detection</li>
                    <li>Response monitoring</li>
                    <li>Bidirectional communication with Python</li>
                </ul>
            </div>
        </div>
    </body>
    </html>
    """

    webview.load_html(test_html)
    logger.info("[OK] Test page loaded")
    logger.info("")

    # Inject JavaScript after page loads
    logger.info("Injecting JavaScript into the page...")
    import time

    time.sleep(0.5)  # Wait for page to load
    webview.eval_js(injection_script)
    logger.info("[OK] JavaScript injected")
    logger.info("")

    logger.info("=" * 60)
    logger.info("INSTRUCTIONS:")
    logger.info("1. Look for the '🔗 Get DCC Selection' button in top-right")
    logger.info("2. Click it to request DCC selection data")
    logger.info("3. The selection will be inserted into the chat input")
    logger.info("4. Open DevTools (F12) to see injection logs")
    logger.info("")
    logger.info("For real third-party sites:")
    logger.info("- Use webview.load_url('https://site.com')")
    logger.info("- Inject after page loads")
    logger.info("- Adapt selectors to match the site's structure")
    logger.info("=" * 60)
    logger.info("")

    try:
        webview.show()
        logger.info("WebView window opened")
    except Exception as e:
        logger.error(f"Error: {e}")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
