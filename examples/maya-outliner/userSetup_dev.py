"""
Maya userSetup.py for AuroraView Outliner (Development Mode)

This version uses Maya's module system (.mod file) for development.

Installation:
    Run: just maya-setup-dev

    This will:
    1. Copy auroraview.mod to ~/Documents/maya/modules/
    2. Copy this file to ~/Documents/maya/2024/scripts/userSetup.py

The .mod file configures PYTHONPATH to point to your project directory,
allowing you to:
- Edit code in the project directory
- Changes are immediately available in Maya (no copying needed)
- Run 'just maya-dev' to rebuild Rust and restart Maya
"""

import maya.utils as mutils
from maya import cmds


def setup_auroraview_outliner():
    """Setup AuroraView Outliner on Maya startup using module system"""

    # Verify PYTHONPATH is set by the module system
    print("[AuroraView] Checking module configuration...")

    # Try to import auroraview
    try:
        import auroraview

        print(f"[AuroraView] ✓ AuroraView {getattr(auroraview, '__version__', 'dev')} loaded")
        print(f"[AuroraView]   Location: {auroraview.__file__}")
    except ImportError as e:
        print(f"[AuroraView] ✗ Failed to import auroraview: {e}")
        print("[AuroraView]   Make sure auroraview.mod is in ~/Documents/maya/modules/")
        print("[AuroraView]   Run 'just maya-setup-dev' to install it")

    # Try to import maya_integration
    try:
        from maya_integration import maya_outliner

        print("[AuroraView] ✓ Maya integration loaded")
        print(f"[AuroraView]   Location: {maya_outliner.__file__}")
    except ImportError as e:
        print(f"[AuroraView] ✗ Failed to import maya_integration: {e}")
        print("[AuroraView]   Make sure auroraview.mod is in ~/Documents/maya/modules/")
        print("[AuroraView]   Run 'just maya-setup-dev' to install it")

    # Create shelf UI (deferred to avoid Maya startup issues)
    def create_shelf():
        """Create AuroraView shelf with Outliner button"""
        shelf_name = "AuroraView"

        # Check if shelf exists, create if not
        if not cmds.shelfLayout(shelf_name, exists=True):
            cmds.shelfLayout(shelf_name, parent="ShelfLayout")
            print(f"[AuroraView] Created shelf: {shelf_name}")

        # Check if button already exists
        existing_buttons = cmds.shelfLayout(shelf_name, query=True, childArray=True) or []
        for button in existing_buttons:
            if cmds.shelfButton(button, query=True, label=True) == "Outliner":
                print("[AuroraView] Shelf button already exists")
                return

        # Create shelf button
        cmds.shelfButton(
            parent=shelf_name,
            label="Outliner",
            annotation="Launch AuroraView Outliner",
            image="outliner.png",
            command="""
# Launch AuroraView Outliner
from maya_integration import maya_outliner
maya_outliner.main()
""",
            sourceType="python",
        )
        print("[AuroraView] Created shelf button: Outliner")

    # Use executeDeferred to create shelf after Maya UI is ready
    mutils.executeDeferred(create_shelf)


# Run setup on Maya startup
print("=" * 60)
print("AuroraView Outliner - Development Mode")
print("=" * 60)
setup_auroraview_outliner()
print("=" * 60)
