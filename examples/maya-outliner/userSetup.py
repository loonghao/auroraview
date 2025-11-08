"""
Maya userSetup.py for AuroraView Outliner

Copy this file to one of these locations to auto-load the outliner on Maya startup:
- Windows: C:/Users/<username>/Documents/maya/<version>/scripts/userSetup.py
- macOS: ~/Library/Preferences/Autodesk/maya/<version>/scripts/userSetup.py
- Linux: ~/maya/<version>/scripts/userSetup.py

Or set MAYA_SCRIPT_PATH environment variable to include this directory.
"""

import os
import sys

import maya.utils as mutils
from maya import cmds


def setup_auroraview_outliner():
    """Setup AuroraView Outliner on Maya startup"""

    # Get the project root (2 levels up from this script)
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(os.path.dirname(script_dir))

    # Add AuroraView to Python path
    python_dir = os.path.join(project_root, "python")
    if python_dir not in sys.path:
        sys.path.insert(0, python_dir)
        print(f"[AuroraView] Added to PYTHONPATH: {python_dir}")

    # Add maya-outliner example to Python path
    outliner_dir = os.path.join(project_root, "examples", "maya-outliner")
    if outliner_dir not in sys.path:
        sys.path.insert(0, outliner_dir)
        print(f"[AuroraView] Added to PYTHONPATH: {outliner_dir}")

    # Create a shelf button for easy access
    def create_shelf_button():
        """Create a shelf button to launch the outliner"""

        # Get or create AuroraView shelf
        shelf_name = "AuroraView"
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
            command=f"""
import sys
import os

# Ensure paths are set
project_root = r"{project_root}"
python_dir = os.path.join(project_root, "python")
outliner_dir = os.path.join(project_root, "examples", "maya-outliner")

if python_dir not in sys.path:
    sys.path.insert(0, python_dir)
if outliner_dir not in sys.path:
    sys.path.insert(0, outliner_dir)

# Launch outliner
from maya_integration import maya_outliner
maya_outliner.main()
""",
            sourceType="python",
        )
        print("[AuroraView] Created shelf button: Outliner")

    # Create shelf button after Maya UI is ready
    mutils.executeDeferred(create_shelf_button)

    print("[AuroraView] Setup complete!")
    print("[AuroraView] Click the 'Outliner' button on the AuroraView shelf to launch")


# Run setup when Maya starts
try:
    setup_auroraview_outliner()
except Exception as e:
    print(f"[AuroraView] Error during setup: {e}")
    import traceback

    traceback.print_exc()
