"""
Example 01: Basic Maya Integration (Native Backend)

This example demonstrates how to create a basic WebView panel in Maya
using the Native backend with HWND parenting.

Features:
- Native backend integration
- HWND-based window parenting
- Bidirectional Python ↔ JavaScript communication
- Maya scene interaction

Usage:
    Run in Maya's Script Editor:
        import sys
        sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\examples')
        import maya.01_basic_integration as example
        example.show()
"""

from auroraview import NativeWebView
import maya.cmds as cmds
import maya.OpenMayaUI as omui
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QWidget


class MayaWebViewPanel:
    """A simple WebView panel for Maya using Native backend."""

    def __init__(self):
        """Initialize the panel."""
        self.webview = None

    def create_panel(self):
        """Create and configure the WebView panel."""
        # Get Maya main window handle
        main_window_ptr = omui.MQtUtil.mainWindow()
        maya_window = wrapInstance(int(main_window_ptr), QWidget)
        hwnd = maya_window.winId()

        # Create WebView instance with Native backend
        self.webview = NativeWebView(
            title="Maya WebView Panel",
            width=800,
            height=600,
            parent_hwnd=hwnd,
            parent_mode="owner"  # Recommended for cross-thread safety
        )

        # Load HTML content
        self.webview.load_html(self._get_html_content())

        # Register event handlers
        self._register_handlers()

        return self.webview

    def _get_html_content(self):
        """Get the HTML content for the panel."""
        return """
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <title>Maya WebView Panel</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    margin: 20px;
                    background: #2b2b2b;
                    color: #e0e0e0;
                }
                .container {
                    max-width: 800px;
                    margin: 0 auto;
                }
                h1 {
                    color: #4fc3f7;
                }
                button {
                    background: #4fc3f7;
                    color: white;
                    border: none;
                    padding: 10px 20px;
                    margin: 5px;
                    cursor: pointer;
                    border-radius: 4px;
                }
                button:hover {
                    background: #29b6f6;
                }
                #scene-info {
                    background: #1e1e1e;
                    padding: 15px;
                    border-radius: 4px;
                    margin-top: 20px;
                }
                .info-item {
                    margin: 10px 0;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Maya WebView Panel</h1>
                
                <div>
                    <button onclick="getSceneInfo()">Get Scene Info</button>
                    <button onclick="createCube()">Create Cube</button>
                    <button onclick="createSphere()">Create Sphere</button>
                </div>
                
                <div id="scene-info">
                    <h3>Scene Information</h3>
                    <div id="info-content">
                        Click "Get Scene Info" to load scene data
                    </div>
                </div>
            </div>
            
            <script>
                // Request scene information from Maya
                function getSceneInfo() {
                    // This will trigger the Python callback
                    window.emit('get_scene_info', {});
                }
                
                // Create a cube in Maya
                function createCube() {
                    window.emit('create_object', { type: 'cube' });
                }
                
                // Create a sphere in Maya
                function createSphere() {
                    window.emit('create_object', { type: 'sphere' });
                }
                
                // Listen for scene info updates from Python
                window.on('scene_info_updated', function(data) {
                    const content = document.getElementById('info-content');
                    content.innerHTML = `
                        <div class="info-item"><strong>Objects:</strong> ${data.object_count}</div>
                        <div class="info-item"><strong>Current Frame:</strong> ${data.current_frame}</div>
                        <div class="info-item"><strong>Selection:</strong> ${data.selection.join(', ') || 'None'}</div>
                    `;
                });
                
                // Listen for object creation confirmation
                window.on('object_created', function(data) {
                    console.log('Created:', data.name);
                    getSceneInfo(); // Refresh scene info
                });
            </script>
        </body>
        </html>
        """

    def _register_handlers(self):
        """Register Python event handlers."""
        
        @self.webview.on('get_scene_info')
        def handle_get_scene_info(data):
            """Get current Maya scene information."""
            # Get all objects in the scene
            all_objects = cmds.ls(dag=True, long=False)
            
            # Get current selection
            selection = cmds.ls(selection=True) or []
            
            # Get current frame
            current_frame = cmds.currentTime(query=True)
            
            # Send data back to JavaScript
            self.webview.emit('scene_info_updated', {
                'object_count': len(all_objects),
                'current_frame': int(current_frame),
                'selection': selection
            })

        @self.webview.on('create_object')
        def handle_create_object(data):
            """Create an object in Maya."""
            obj_type = data.get('type', 'cube')
            
            if obj_type == 'cube':
                obj = cmds.polyCube()[0]
            elif obj_type == 'sphere':
                obj = cmds.polySphere()[0]
            else:
                obj = cmds.polyCube()[0]
            
            # Notify JavaScript
            self.webview.emit('object_created', {
                'name': obj,
                'type': obj_type
            })

    def show(self):
        """Show the panel."""
        if self.webview is None:
            self.create_panel()

        # Use show_async() for non-blocking display
        self.webview.show_async()


# Global instance
_panel = None


def show():
    """Show the Maya WebView panel."""
    global _panel
    
    if _panel is None:
        _panel = MayaWebViewPanel()
    
    _panel.show()


if __name__ == '__main__':
    show()

