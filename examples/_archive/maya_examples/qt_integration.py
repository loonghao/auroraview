"""
Example 03: Maya Integration (Qt Backend)

This example demonstrates how to create a WebView panel in Maya
using the Qt backend for seamless Qt widget integration.

Features:
- Qt backend integration
- Seamless Qt widget parenting
- No HWND handling required
- Better integration with Maya's Qt UI

Requirements:
    pip install auroraview[qt]

Usage:
    Run in Maya's Script Editor:
        import sys
        sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\examples')
        import maya.03_qt_integration as example
        example.show()
"""

try:
    from auroraview import QtWebView
except ImportError:
    print("[ERROR] Qt backend not available!")
    print("Install with: pip install auroraview[qt]")
    raise

import maya.cmds as cmds
import maya.OpenMayaUI as omui
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QWidget


def get_maya_main_window():
    """Get Maya's main window as a QWidget."""
    main_window_ptr = omui.MQtUtil.mainWindow()
    return wrapInstance(int(main_window_ptr), QWidget)


class MayaQtWebViewPanel:
    """A WebView panel for Maya using Qt backend."""

    def __init__(self):
        """Initialize the panel."""
        self.webview = None

    def create_panel(self):
        """Create and configure the WebView panel."""
        # Get Maya main window as Qt widget
        maya_window = get_maya_main_window()
        
        # Create WebView instance with Qt backend
        self.webview = QtWebView(
            parent=maya_window,
            title="Maya Qt WebView Panel",
            width=800,
            height=600
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
            <title>Maya Qt WebView Panel</title>
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
                .badge {
                    display: inline-block;
                    background: #667eea;
                    color: white;
                    padding: 4px 8px;
                    border-radius: 4px;
                    font-size: 12px;
                    margin-left: 10px;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Maya Qt WebView Panel <span class="badge">Qt Backend</span></h1>
                
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
        
        # Qt widgets use show() directly
        self.webview.show()


# Global instance
_panel = None


def show():
    """Show the Maya Qt WebView panel."""
    global _panel
    
    if _panel is None:
        _panel = MayaQtWebViewPanel()
    
    _panel.show()


if __name__ == '__main__':
    show()
