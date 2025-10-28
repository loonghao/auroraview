#!/usr/bin/env python
"""
Diagnostic script to check which version of auroraview is being loaded.

Run this in Maya to verify that the latest compiled version is being used.
"""

import sys
import os
import logging

logging.basicConfig(level=logging.INFO, format='# %(name)s : %(message)s #')
logger = logging.getLogger(__name__)

def diagnose_module_loading():
    """Diagnose which version of auroraview is being loaded."""
    logger.info("=" * 70)
    logger.info("Diagnosing Module Loading")
    logger.info("=" * 70)
    logger.info("")
    
    # Show current sys.path
    logger.info("Current sys.path:")
    for i, path in enumerate(sys.path):
        logger.info(f"  [{i}] {path}")
    logger.info("")
    
    # Add project paths
    project_root = r"C:\Users\hallo\Documents\augment-projects\dcc_webview"
    python_path = os.path.join(project_root, "python")
    release_path = os.path.join(project_root, "target", "release")
    
    logger.info("Adding project paths to sys.path:")
    if python_path not in sys.path:
        sys.path.insert(0, python_path)
        logger.info(f"  ✓ Added: {python_path}")
    else:
        logger.info(f"  ✓ Already in path: {python_path}")
    
    if release_path not in sys.path:
        sys.path.insert(0, release_path)
        logger.info(f"  ✓ Added: {release_path}")
    else:
        logger.info(f"  ✓ Already in path: {release_path}")
    logger.info("")
    
    # Check for .pyd files
    logger.info("Checking for compiled .pyd files:")
    pyd_files = []
    for root, dirs, files in os.walk(release_path):
        for file in files:
            if file.endswith('.pyd') or file.endswith('.so'):
                full_path = os.path.join(root, file)
                pyd_files.append(full_path)
                logger.info(f"  ✓ Found: {file}")
    
    if not pyd_files:
        logger.warning("  ✗ No .pyd files found in target/release!")
    logger.info("")
    
    # Try to import auroraview
    logger.info("Attempting to import auroraview...")
    try:
        import auroraview
        logger.info(f"  ✓ Successfully imported auroraview")
        logger.info(f"    Location: {auroraview.__file__}")
        logger.info("")
        
        # Try to import _core
        logger.info("Attempting to import auroraview._core...")
        import auroraview._core
        logger.info(f"  ✓ Successfully imported auroraview._core")
        logger.info(f"    Location: {auroraview._core.__file__}")
        logger.info("")
        
        # Check WebView class
        logger.info("Checking WebView class...")
        from auroraview import WebView
        logger.info(f"  ✓ WebView class: {WebView}")
        logger.info(f"    Module: {WebView.__module__}")
        logger.info("")
        
        # Try to create a WebView instance
        logger.info("Attempting to create WebView instance...")
        webview = WebView(title="Test", width=400, height=300)
        logger.info(f"  ✓ Successfully created WebView instance")
        logger.info(f"    Instance: {webview}")
        logger.info("")
        
        logger.info("=" * 70)
        logger.info("✓ All imports successful!")
        logger.info("✓ Module loading is working correctly!")
        logger.info("=" * 70)
        
        return True
        
    except Exception as e:
        logger.error(f"  ✗ Import failed: {e}", exc_info=True)
        logger.info("")
        logger.info("=" * 70)
        logger.info("✗ Module loading failed!")
        logger.info("=" * 70)
        return False

if __name__ == "__main__":
    success = diagnose_module_loading()
    sys.exit(0 if success else 1)

