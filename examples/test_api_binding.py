"""Test API binding in Maya Outliner.

This script helps debug API binding issues.

Usage:
    1. Open Maya
    2. Run this script in Maya's Script Editor
    3. Check the console output for API binding details

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import sys
from pathlib import Path

# Add parent directory to path for imports
parent_dir = Path(__file__).parent.parent
if str(parent_dir) not in sys.path:
    sys.path.insert(0, str(parent_dir))

from maya_integration import maya_outliner


def test_api_binding():
    """Test API binding and list all bound methods."""
    print("=" * 60)
    print("Maya Outliner - API Binding Test")
    print("=" * 60)
    print()

    # Create outliner
    print("Creating Maya Outliner...")
    outliner = maya_outliner.main()
    
    print()
    print("=" * 60)
    print("API Object Information")
    print("=" * 60)
    
    # Check API object
    if hasattr(outliner, 'api'):
        api = outliner.api
        print(f"✓ API object found: {type(api)}")
        print()
        
        # List all public methods
        print("Public methods on API object:")
        methods = [name for name in dir(api) if not name.startswith('_') and callable(getattr(api, name))]
        for method in methods:
            print(f"  - {method}")
        print()
        print(f"Total: {len(methods)} public methods")
    else:
        print("✗ No API object found on outliner")
    
    print()
    print("=" * 60)
    print("AuroraView Wrapper Information")
    print("=" * 60)
    
    # Check AuroraView wrapper
    if hasattr(outliner, 'auroraview'):
        auroraview = outliner.auroraview
        print(f"✓ AuroraView wrapper found: {type(auroraview)}")
        
        # Check if bind_api was called
        if hasattr(auroraview, '_view'):
            view = auroraview._view
            print(f"✓ Underlying view: {type(view)}")
            
            # Try to access IPC handler
            if hasattr(view, '_webview'):
                webview = view._webview
                print(f"✓ WebView: {type(webview)}")
                
                if hasattr(webview, 'ipc_handler'):
                    ipc_handler = webview.ipc_handler
                    print(f"✓ IPC Handler: {type(ipc_handler)}")
    else:
        print("✗ No AuroraView wrapper found on outliner")
    
    print()
    print("=" * 60)
    print("JavaScript Test")
    print("=" * 60)
    print()
    print("Open the browser DevTools and run:")
    print()
    print("  console.log('window.auroraview:', window.auroraview)")
    print("  console.log('window.auroraview.api:', window.auroraview.api)")
    print("  console.log('typeof window.auroraview.api.get_scene_hierarchy:', typeof window.auroraview.api.get_scene_hierarchy)")
    print()
    print("  // Try calling the API")
    print("  window.auroraview.api.get_scene_hierarchy().then(result => {")
    print("    console.log('Result:', result)")
    print("  }).catch(error => {")
    print("    console.error('Error:', error)")
    print("  })")
    print()
    print("=" * 60)
    
    return outliner


if __name__ == "__main__":
    outliner = test_api_binding()

