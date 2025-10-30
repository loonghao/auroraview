#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
Test script for window decorations (title bar) control.

This script demonstrates:
1. Creating a WebView without decorations (no title bar)
2. Custom HTML-based window controls
3. Proper window closing via JavaScript events
"""

import sys
sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\python')

from auroraview import NativeWebView

# HTML with custom window controls
html = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            overflow: hidden;
        }
        
        /* Custom title bar */
        .title-bar {
            background: rgba(0, 0, 0, 0.3);
            padding: 12px 16px;
            display: flex;
            justify-content: space-between;
            align-items: center;
            -webkit-app-region: drag; /* Make it draggable */
            backdrop-filter: blur(10px);
        }
        
        .title-bar h1 {
            font-size: 14px;
            font-weight: 600;
            letter-spacing: 0.5px;
        }
        
        .window-controls {
            display: flex;
            gap: 8px;
            -webkit-app-region: no-drag; /* Buttons should be clickable */
        }
        
        .window-controls button {
            width: 32px;
            height: 32px;
            border: none;
            border-radius: 6px;
            background: rgba(255, 255, 255, 0.1);
            color: white;
            cursor: pointer;
            font-size: 16px;
            transition: all 0.2s;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        
        .window-controls button:hover {
            background: rgba(255, 255, 255, 0.2);
            transform: scale(1.05);
        }
        
        .window-controls button.close:hover {
            background: #e74c3c;
        }
        
        /* Content area */
        .content {
            padding: 40px;
            text-align: center;
        }
        
        .content h2 {
            font-size: 32px;
            margin-bottom: 16px;
            font-weight: 700;
        }
        
        .content p {
            font-size: 16px;
            opacity: 0.9;
            line-height: 1.6;
            max-width: 500px;
            margin: 0 auto 32px;
        }
        
        .demo-button {
            background: white;
            color: #667eea;
            border: none;
            padding: 12px 32px;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
        }
        
        .demo-button:hover {
            transform: translateY(-2px);
            box-shadow: 0 6px 16px rgba(0, 0, 0, 0.3);
        }
        
        .demo-button:active {
            transform: translateY(0);
        }
        
        .status {
            margin-top: 24px;
            padding: 16px;
            background: rgba(0, 0, 0, 0.2);
            border-radius: 8px;
            font-family: 'Courier New', monospace;
            font-size: 14px;
        }
    </style>
</head>
<body>
    <!-- Custom title bar -->
    <div class="title-bar">
        <h1>✨ AuroraView - No Decorations Demo</h1>
        <div class="window-controls">
            <button onclick="testEvent()" title="Test Event">🔔</button>
            <button class="close" onclick="closeWindow()" title="Close">✕</button>
        </div>
    </div>
    
    <!-- Content -->
    <div class="content">
        <h2>🎨 Custom Window Controls</h2>
        <p>
            This window has no native title bar (decorations=False).
            All controls are custom HTML/CSS/JavaScript.
        </p>
        <button class="demo-button" onclick="testEvent()">
            🚀 Test Event
        </button>
        <div class="status" id="status">
            Ready. Click the test button or close button.
        </div>
    </div>
    
    <script>
        console.log('🟢 [init] Script loaded');
        
        function testEvent() {
            console.log('📤 [testEvent] Sending test event to Python...');
            const event = new CustomEvent('test_event', {
                detail: {
                    message: 'Hello from JavaScript!',
                    timestamp: Date.now()
                }
            });
            window.dispatchEvent(event);
            
            document.getElementById('status').textContent = 
                'Event sent at ' + new Date().toLocaleTimeString();
        }
        
        function closeWindow() {
            console.log('📤 [closeWindow] Sending close event to Python...');
            const event = new CustomEvent('close_window', {
                detail: {
                    source: 'close_button',
                    timestamp: Date.now()
                }
            });
            window.dispatchEvent(event);
            
            document.getElementById('status').textContent = 
                'Close requested...';
        }
        
        // Notify Python that we're ready
        window.addEventListener('DOMContentLoaded', () => {
            console.log('🟢 [DOMContentLoaded] Page ready');
            const event = new CustomEvent('webview_ready', {
                detail: { timestamp: Date.now() }
            });
            window.dispatchEvent(event);
        });
    </script>
</body>
</html>
"""

# Create WebView without decorations
print("Creating WebView without decorations...")
print("This is a standalone window (no parent)")
webview = NativeWebView(
    title="No Decorations Demo",
    width=600,
    height=400,
    decorations=False,  # No title bar!
    resizable=True,
    dev_tools=True,
    parent_hwnd=None,  # Standalone mode
    parent_mode=None,  # No parent mode
)

# Event handlers
@webview.on("webview_ready")
def handle_ready(data):
    print(f"✅ WebView ready: {data}")

@webview.on("test_event")
def handle_test(data):
    print(f"🔔 Test event received: {data}")

@webview.on("close_window")
def handle_close(data):
    print(f"🔴 Close requested: {data}")
    print("Closing window...")
    webview.close()
    print("✅ Window closed")

# Load HTML
webview.load_html(html)

# Show window
print("Showing window...")
print("NOTE: In standalone mode, show() is blocking until window closes")
webview.show()
print("✅ Window closed by user")

