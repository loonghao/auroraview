#!/usr/bin/env python
"""
Maya DCC Integration Example

This example shows how to integrate DCC WebView with Maya.
It demonstrates embedding a WebView in Maya's UI.

Usage:
    In Maya Python console:
    >>> exec(open('examples/maya_dcc_integration.py').read())
"""

import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from maya import cmds, mel
    from maya.app.general.mayaMixin import MayaQWidgetBaseMixin
    from PySide2 import QtWidgets, QtCore
    MAYA_AVAILABLE = True
except ImportError:
    MAYA_AVAILABLE = False
    print("Maya not available. This example requires Maya.")


def get_maya_main_window():
    """Get Maya's main window pointer"""
    if not MAYA_AVAILABLE:
        return None
    
    # Get Maya's main window
    main_window = mel.eval('$temp1=$gMainWindow')
    return main_window


def create_webview_panel():
    """Create a WebView panel in Maya"""
    if not MAYA_AVAILABLE:
        print("Maya is not available")
        return
    
    print("=" * 70)
    print("AuroraView - Maya Integration Example")
    print("=" * 70)
    print()
    
    # Import WebView
    from auroraview import WebView
    
    # Create WebView instance
    print("Creating WebView...")
    webview = WebView(
        title="AuroraView - Maya Tool",
        width=600,
        height=400
    )
    print("✓ WebView created")
    
    # Load HTML content
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Maya AuroraView Tool</title>
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
                padding: 20px;
            }
            
            .container {
                background: white;
                border-radius: 12px;
                box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
                padding: 40px;
                max-width: 500px;
                width: 100%;
            }
            
            h1 {
                color: #333;
                margin-bottom: 10px;
                font-size: 24px;
            }
            
            .subtitle {
                color: #666;
                margin-bottom: 30px;
                font-size: 14px;
            }
            
            .info-box {
                background: #f0f4ff;
                border-left: 4px solid #667eea;
                padding: 15px;
                margin-bottom: 20px;
                border-radius: 4px;
                font-size: 13px;
                color: #333;
                line-height: 1.6;
            }
            
            .button-group {
                display: flex;
                gap: 10px;
                margin-top: 20px;
            }
            
            button {
                flex: 1;
                padding: 12px 20px;
                border: none;
                border-radius: 6px;
                font-size: 14px;
                font-weight: 600;
                cursor: pointer;
                transition: all 0.3s ease;
            }
            
            .btn-primary {
                background: #667eea;
                color: white;
            }
            
            .btn-primary:hover {
                background: #5568d3;
                transform: translateY(-2px);
                box-shadow: 0 10px 20px rgba(102, 126, 234, 0.3);
            }
            
            .btn-secondary {
                background: #f0f4ff;
                color: #667eea;
            }
            
            .btn-secondary:hover {
                background: #e0e8ff;
            }
            
            .status {
                background: #f9f9f9;
                border: 1px solid #e0e0e0;
                border-radius: 6px;
                padding: 15px;
                margin-top: 20px;
                font-size: 12px;
                color: #666;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🎨 Maya AuroraView</h1>
            <p class="subtitle">Embedded WebView in Maya</p>
            
            <div class="info-box">
                <strong>This is a WebView embedded in Maya!</strong><br>
                This demonstrates how AuroraView can be integrated into DCC software
                like Maya, Houdini, and Blender.
            </div>
            
            <div class="button-group">
                <button class="btn-primary" onclick="handleClick()">Create Cube</button>
                <button class="btn-secondary" onclick="showInfo()">Info</button>
            </div>
            
            <div class="status">
                <strong>Status:</strong> WebView is running in Maya<br>
                <strong>Mode:</strong> Embedded (DCC Integration)<br>
                <strong>Framework:</strong> Rust + Python + WebView
            </div>
        </div>
        
        <script>
            function handleClick() {
                // Send event to Python
                window.dispatchEvent(new CustomEvent('create_cube', {
                    detail: { size: 1.0 }
                }));
                alert('✓ Cube creation event sent to Maya!');
            }
            
            function showInfo() {
                alert('AuroraView v0.1.0\\n\\nEmbedded WebView for DCC Software\\n\\nBuilt with Rust + Python');
            }
        </script>
    </body>
    </html>
    """
    
    print("Loading HTML content...")
    webview.load_html(html)
    print("✓ HTML loaded")
    
    # Register event handler
    print("Registering event handlers...")
    
    @webview.on("create_cube")
    def handle_create_cube(data):
        print(f"✓ Event received: {data}")
        if MAYA_AVAILABLE:
            try:
                size = data.get("size", 1.0)
                cmds.polyCube(w=size, h=size, d=size)
                print(f"✓ Created cube in Maya with size: {size}")
            except Exception as e:
                print(f"✗ Error creating cube: {e}")
    
    print("✓ Event handlers registered")
    print()
    print("=" * 70)
    print("WebView is ready!")
    print("=" * 70)
    print()
    print("Next steps:")
    print("1. The WebView is embedded in Maya")
    print("2. Click buttons to interact with Maya")
    print("3. Events are sent from WebView to Python")
    print("4. Python can execute Maya commands")
    print()
    
    # For now, just show the WebView info
    # In a real implementation, this would be embedded in Maya's UI
    print("Note: In a real implementation, this WebView would be embedded")
    print("in Maya's UI using the create_embedded() method with Maya's window handle.")
    print()
    print("Example:")
    print("  maya_hwnd = get_maya_main_window_handle()")
    print("  webview.create_embedded(maya_hwnd, 600, 400)")
    print()


def main():
    """Main entry point"""
    print()
    print("╔" + "=" * 68 + "╗")
    print("║" + " " * 68 + "║")
    print("║" + "  AuroraView - Maya Integration Example".center(68) + "║")
    print("║" + " " * 68 + "║")
    print("╚" + "=" * 68 + "╝")
    print()
    
    if not MAYA_AVAILABLE:
        print("This example requires Maya to be running.")
        print("Please run this script from Maya's Python console.")
        print()
        print("In Maya Python console:")
        print("  >>> exec(open('examples/maya_dcc_integration.py').read())")
        print()
        return 1
    
    try:
        create_webview_panel()
        return 0
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())

