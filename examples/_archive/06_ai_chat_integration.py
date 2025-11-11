#!/usr/bin/env python
"""
Example 06: AI Chat Website Integration (Knot.woa.com)

This example demonstrates integration with a specific AI chat website.
It shows how to:
1. Load the AI chat website
2. Inject JavaScript to hook into the chat interface
3. Send DCC scene data to the AI
4. Execute AI-generated code in DCC

This is a real-world example for DCC integration with AI assistants.

Usage:
    python examples/06_ai_chat_integration.py
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


# Simulated DCC functions (replace with real Maya/Houdini/etc. code)
def get_scene_selection():
    """Get selected objects from DCC scene."""
    # In Maya: import maya.cmds as cmds; return cmds.ls(selection=True)
    # In Houdini: return hou.selectedNodes()
    return [
        {"name": "pCube1", "type": "mesh", "transform": [0, 0, 0]},
        {"name": "pSphere1", "type": "mesh", "transform": [5, 0, 0]},
    ]


def execute_dcc_code(code, language="python"):
    """Execute code in DCC environment."""
    logger.info(f"Executing {language} code in DCC:")
    logger.info(code)

    # In Maya: exec(code) or maya.mel.eval(code) for MEL
    # In Houdini: exec(code)

    # For safety in this demo, we just log it
    logger.info("[DEMO] Code execution simulated (not actually executed)")
    return {"status": "success", "message": "Code executed successfully"}


def main():
    """Main function for AI chat integration."""
    logger.info("=" * 60)
    logger.info("AuroraView - AI Chat Integration")
    logger.info("=" * 60)
    logger.info("")

    # Create WebView
    logger.info("Creating WebView for AI chat...")
    webview = WebView(title="AI Chat - DCC Integration", width=1200, height=800, debug=True)
    logger.info("[OK] WebView created")
    logger.info("")

    # Event handlers for DCC ↔ AI communication
    @webview.on("get_scene_info")
    def handle_get_scene_info(data):
        """Send scene information to AI."""
        logger.info("[RECV] AI requested scene information")

        selection = get_scene_selection()
        scene_info = {
            "selection": selection,
            "selection_count": len(selection),
            "timestamp": datetime.now().isoformat(),
            "dcc": "Maya 2024",  # or detect actual DCC
        }

        webview.emit("scene_info_response", scene_info)
        logger.info(f"[SEND] Sent scene info: {len(selection)} objects")

    @webview.on("execute_code")
    def handle_execute_code(data):
        """Execute AI-generated code in DCC."""
        logger.info("[RECV] AI sent code to execute")

        code = data.get("code", "")
        language = data.get("language", "python")

        try:
            result = execute_dcc_code(code, language)
            webview.emit(
                "execution_result",
                {"status": "success", "result": result, "timestamp": datetime.now().isoformat()},
            )
            logger.info("[SEND] Execution successful")
        except Exception as e:
            logger.error(f"Execution failed: {e}")
            webview.emit(
                "execution_result",
                {"status": "error", "error": str(e), "timestamp": datetime.now().isoformat()},
            )

    @webview.on("ai_message_sent")
    def handle_ai_message(data):
        """Log when user sends message to AI."""
        logger.info(f"[INFO] User sent message to AI: {data.get('message', '')[:50]}...")

    @webview.on("ai_response_received")
    def handle_ai_response(data):
        """Process AI responses."""
        logger.info(f"[INFO] AI responded: {data.get('message', '')[:50]}...")

        # You could parse the response for code blocks, commands, etc.
        message = data.get("message", "")
        if "```python" in message:
            logger.info("[INFO] AI response contains Python code")
        if "```mel" in message:
            logger.info("[INFO] AI response contains MEL code")

    logger.info("Event handlers registered")
    logger.info("")

    # JavaScript injection for AI chat integration
    injection_script = """
    (function() {
        console.log('[DCC-AI] Injection script starting...');
        
        // Create DCC integration namespace
        window.DCCIntegration = {
            initialized: false,
            chatInput: null,
            
            // Initialize the integration
            init: function() {
                if (this.initialized) return;
                
                console.log('[DCC-AI] Initializing DCC integration...');
                
                // Inject UI elements
                this.injectUI();
                
                // Hook into chat interface
                this.hookChatInterface();
                
                // Monitor for AI responses
                this.monitorResponses();
                
                this.initialized = true;
                console.log('[DCC-AI] Initialization complete');
            },
            
            // Inject custom UI elements
            injectUI: function() {
                // Create floating toolbar
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
                
                // Add event listeners
                document.getElementById('dcc-get-selection').onclick = () => {
                    this.requestSceneInfo();
                };
                
                document.getElementById('dcc-execute-code').onclick = () => {
                    this.extractAndExecuteCode();
                };
                
                console.log('[DCC-AI] UI injected');
            },
            
            // Hook into chat interface
            hookChatInterface: function() {
                // Try to find chat input (adapt selectors for specific site)
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
                        
                        // Monitor user messages
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
            
            // Monitor for AI responses
            monitorResponses: function() {
                const observer = new MutationObserver((mutations) => {
                    mutations.forEach((mutation) => {
                        mutation.addedNodes.forEach((node) => {
                            if (node.nodeType === 1) {
                                const text = node.textContent || '';
                                // Filter out short texts (likely UI elements)
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
            
            // Request scene information from DCC
            requestSceneInfo: function() {
                console.log('[DCC-AI] Requesting scene info...');
                window.dispatchEvent(new CustomEvent('get_scene_info', {
                    detail: { timestamp: Date.now() }
                }));
            },
            
            // Extract code from AI response and execute
            extractAndExecuteCode: function() {
                // Find the last AI response
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
                
                // Extract code blocks
                const codeBlockRegex = /```(\\w+)?\\n([\\s\\S]*?)```/g;
                const matches = [...lastMessage.matchAll(codeBlockRegex)];
                
                if (matches.length === 0) {
                    alert('No code blocks found');
                    return;
                }
                
                // Execute each code block
                matches.forEach((match, index) => {
                    const language = match[1] || 'python';
                    const code = match[2].trim();
                    
                    console.log(`[DCC-AI] Executing code block ${index + 1} (${language})`);
                    window.dispatchEvent(new CustomEvent('execute_code', {
                        detail: { code: code, language: language }
                    }));
                });
            },
            
            // Insert text into chat input
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
        
        // Listen for responses from DCC
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
        
        // Auto-initialize after page loads
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

    # Load the AI chat website
    logger.info("Loading AI chat website...")

    # OPTION 1: Load the actual website (uncomment to use)
    # webview.load_url("https://knot.woa.com/chat?web_key=1c2a6b4568f24e00a58999c1b7cb0f6e")

    # OPTION 2: Load a test page for demonstration
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
    logger.info("[OK] Page loaded")
    logger.info("")

    # Inject JavaScript
    logger.info("Injecting DCC integration script...")
    import time

    time.sleep(0.5)
    webview.eval_js(injection_script)
    logger.info("[OK] Script injected")
    logger.info("")

    logger.info("=" * 60)
    logger.info("AI Chat Integration Ready!")
    logger.info("")
    logger.info("Features:")
    logger.info("✅ Get DCC scene selection")
    logger.info("✅ Send scene info to AI")
    logger.info("✅ Execute AI-generated code in DCC")
    logger.info("✅ Monitor AI responses")
    logger.info("")
    logger.info("To use with real AI chat:")
    logger.info("1. Uncomment the webview.load_url() line")
    logger.info("2. Adjust selectors in injection script if needed")
    logger.info("3. Test with DevTools (F12) open")
    logger.info("=" * 60)

    try:
        webview.show()
    except Exception as e:
        logger.error(f"Error: {e}")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
