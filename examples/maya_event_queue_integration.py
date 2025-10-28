#!/usr/bin/env python
"""
Maya WebView Integration with Thread-Safe Event Queue

This example demonstrates how to use the DCCEventQueue for safe communication
between WebView (running in background thread) and Maya main thread.

Key features:
- WebView runs in background thread (Maya stays responsive)
- Events are posted to a thread-safe queue
- Maya main thread processes events periodically
- All Maya API calls happen in the main thread

Usage:
1. Open Maya 2022
2. Open Script Editor (Ctrl + Shift + E)
3. Switch to Python tab
4. Copy this entire script
5. Paste into the Python tab
6. Execute (Ctrl + Enter)
7. Click buttons in WebView - Maya should stay responsive!
"""

import sys
import os
import logging

# Add project paths
project_root = r"C:\Users\hallo\Documents\augment-projects\dcc_webview"
python_path = os.path.join(project_root, "python")
if python_path not in sys.path:
    sys.path.insert(0, python_path)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='# %(name)s : %(message)s #'
)
logger = logging.getLogger(__name__)

# Import after path setup
from auroraview import WebView
from auroraview.dcc_event_queue import DCCEventQueue
import maya.cmds as cmds


def main():
    """Main function to demonstrate event queue integration."""
    logger.info("=" * 70)
    logger.info("Maya WebView Integration with Event Queue")
    logger.info("=" * 70)
    logger.info("")
    
    # Create event queue
    event_queue = DCCEventQueue()
    logger.info("✓ Created event queue")
    
    # Define Maya command callbacks
    def on_select_object(obj_name):
        """Select an object in Maya."""
        try:
            if cmds.objExists(obj_name):
                cmds.select(obj_name)
                logger.info(f"✓ Selected: {obj_name}")
            else:
                logger.warning(f"Object not found: {obj_name}")
        except Exception as e:
            logger.error(f"Error selecting object: {e}")
    
    def on_create_cube():
        """Create a cube in Maya."""
        try:
            cube = cmds.polyCube(name="WebViewCube")[0]
            logger.info(f"✓ Created cube: {cube}")
        except Exception as e:
            logger.error(f"Error creating cube: {e}")
    
    def on_delete_selected():
        """Delete selected objects."""
        try:
            selected = cmds.ls(selection=True)
            if selected:
                cmds.delete(selected)
                logger.info(f"✓ Deleted: {selected}")
            else:
                logger.warning("No objects selected")
        except Exception as e:
            logger.error(f"Error deleting objects: {e}")
    
    # Register callbacks
    event_queue.register_callback("select_object", on_select_object)
    event_queue.register_callback("create_cube", on_create_cube)
    event_queue.register_callback("delete_selected", on_delete_selected)
    logger.info("✓ Registered callbacks")
    logger.info("")
    
    # Create HTML UI
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <style>
            body {
                font-family: Arial, sans-serif;
                padding: 20px;
                background: #f5f5f5;
            }
            .container {
                max-width: 400px;
                margin: 0 auto;
                background: white;
                padding: 20px;
                border-radius: 8px;
                box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            }
            h1 {
                color: #333;
                margin-top: 0;
            }
            .button-group {
                display: flex;
                flex-direction: column;
                gap: 10px;
            }
            button {
                padding: 10px 15px;
                font-size: 14px;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                background: #007bff;
                color: white;
                transition: background 0.3s;
            }
            button:hover {
                background: #0056b3;
            }
            button:active {
                background: #004085;
            }
            .info {
                margin-top: 20px;
                padding: 10px;
                background: #e7f3ff;
                border-left: 4px solid #007bff;
                border-radius: 4px;
                font-size: 12px;
                color: #333;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🎨 Maya WebView Control</h1>
            <div class="button-group">
                <button onclick="createCube()">Create Cube</button>
                <button onclick="deleteSelected()">Delete Selected</button>
                <button onclick="selectCube()">Select Cube</button>
            </div>
            <div class="info">
                <strong>ℹ️ Info:</strong><br>
                Click buttons to control Maya.<br>
                Maya should stay responsive!
            </div>
        </div>
        
        <script>
            function createCube() {
                console.log("Creating cube...");
                window.pywebview.api.post_event("create_cube");
            }
            
            function deleteSelected() {
                console.log("Deleting selected...");
                window.pywebview.api.post_event("delete_selected");
            }
            
            function selectCube() {
                console.log("Selecting cube...");
                window.pywebview.api.post_event("select_object", "WebViewCube");
            }
        </script>
    </body>
    </html>
    """
    
    # Create WebView
    webview = WebView(
        title="Maya WebView Control",
        width=400,
        height=350
    )
    
    # Create API bridge
    class WebViewAPI:
        def post_event(self, event_name, *args):
            """Post event from JavaScript to event queue."""
            logger.info(f"WebView event: {event_name} {args}")
            event_queue.post_event(event_name, *args)
    
    webview.set_api(WebViewAPI())
    webview.load_html(html)
    logger.info("✓ Created WebView")
    logger.info("")
    
    # Show WebView in background thread
    logger.info("Starting WebView in background thread...")
    webview.show_async()
    logger.info("✓ WebView started")
    logger.info("")
    
    # Setup event processing in Maya
    logger.info("Setting up event processing in Maya...")
    
    def process_events_callback():
        """Process events from queue."""
        count = event_queue.process_events()
        if count > 0:
            logger.info(f"Processed {count} events from queue")
    
    # Use Maya's scriptJob to process events periodically
    try:
        cmds.scriptJob(
            event=["idle", process_events_callback],
            permanent=True
        )
        logger.info("✓ Event processing setup complete")
    except Exception as e:
        logger.warning(f"Could not setup scriptJob: {e}")
        logger.info("Events will be processed manually")
    
    logger.info("")
    logger.info("=" * 70)
    logger.info("✓ Setup complete!")
    logger.info("✓ Click buttons in WebView to control Maya")
    logger.info("✓ Maya should stay responsive")
    logger.info("=" * 70)


if __name__ == "__main__":
    main()

