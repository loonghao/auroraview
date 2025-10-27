"""Simple WebView panel example for Maya.

This example demonstrates how to create a basic WebView panel in Maya
that can communicate with the Maya scene.

Usage:
    1. Copy this file to your Maya scripts directory
    2. Run in Maya's Script Editor:
        import simple_panel
        simple_panel.show()
"""

from auroraview import WebView
import maya.cmds as cmds


class MayaWebViewPanel:
    """A simple WebView panel for Maya."""

    def __init__(self):
        """Initialize the panel."""
        self.webview = None

    def create_panel(self):
        """Create and configure the WebView panel."""
        # Create WebView instance
        self.webview = WebView(
            title="Maya WebView Panel",
            width=800,
            height=600,
            html=self._get_html_content()
        )

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
        
        self.webview.show()


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

