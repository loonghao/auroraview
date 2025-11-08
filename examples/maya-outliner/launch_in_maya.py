"""
Quick launch script for Maya Outliner

Copy and paste this into Maya's Script Editor to launch the outliner.
"""

import os
import sys

# Add the example directory to Python path
example_dir = os.path.dirname(__file__)
if example_dir not in sys.path:
    sys.path.insert(0, example_dir)

# Import and run
from maya_integration import maya_outliner

# Launch the outliner
maya_outliner.main()
