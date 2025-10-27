#!/usr/bin/env python
"""
Debug Window - Diagnose WebView Issues

This script helps diagnose why the WebView window might not be displaying.
"""

import sys
import logging
from pathlib import Path
import time

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

# Configure detailed logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def test_import():
    """Test 1: Import the module"""
    logger.info("=" * 70)
    logger.info("Test 1: Import Module")
    logger.info("=" * 70)
    
    try:
        from auroraview import WebView
        logger.info("✓ Successfully imported WebView")
        return True
    except Exception as e:
        logger.error(f"✗ Failed to import: {e}", exc_info=True)
        return False


def test_create():
    """Test 2: Create WebView instance"""
    logger.info("")
    logger.info("=" * 70)
    logger.info("Test 2: Create WebView Instance")
    logger.info("=" * 70)
    
    try:
        from auroraview import WebView
        
        logger.info("Creating WebView with default parameters...")
        webview = WebView()
        logger.info(f"✓ Created: {webview}")
        
        logger.info("Creating WebView with custom parameters...")
        webview2 = WebView(
            title="Debug Window",
            width=600,
            height=400
        )
        logger.info(f"✓ Created: {webview2}")
        
        return webview2
    except Exception as e:
        logger.error(f"✗ Failed to create: {e}", exc_info=True)
        return None


def test_load_html(webview):
    """Test 3: Load HTML"""
    logger.info("")
    logger.info("=" * 70)
    logger.info("Test 3: Load HTML Content")
    logger.info("=" * 70)
    
    try:
        html = "<h1>Debug Test</h1><p>If you see this, HTML loaded successfully!</p>"
        logger.info(f"Loading HTML ({len(html)} bytes)...")
        webview.load_html(html)
        logger.info("✓ HTML loaded successfully")
        return True
    except Exception as e:
        logger.error(f"✗ Failed to load HTML: {e}", exc_info=True)
        return False


def test_event_handler(webview):
    """Test 4: Register event handler"""
    logger.info("")
    logger.info("=" * 70)
    logger.info("Test 4: Register Event Handler")
    logger.info("=" * 70)
    
    try:
        logger.info("Registering event handler...")
        
        @webview.on("test_event")
        def handle_test(data):
            logger.info(f"✓ Event handler called with data: {data}")
        
        logger.info("✓ Event handler registered successfully")
        return True
    except Exception as e:
        logger.error(f"✗ Failed to register handler: {e}", exc_info=True)
        return False


def test_show(webview):
    """Test 5: Show window"""
    logger.info("")
    logger.info("=" * 70)
    logger.info("Test 5: Show WebView Window")
    logger.info("=" * 70)
    logger.info("")
    logger.info("IMPORTANT: A window should appear now!")
    logger.info("If you see a window:")
    logger.info("  - Close it to continue the test")
    logger.info("  - The test will complete successfully")
    logger.info("")
    logger.info("If you DON'T see a window:")
    logger.info("  - Wait 5 seconds")
    logger.info("  - The test will timeout and report the issue")
    logger.info("")
    logger.info("=" * 70)
    logger.info("")
    
    try:
        logger.info("Calling webview.show()...")
        logger.info("(This will block until the window is closed)")
        logger.info("")
        
        webview.show()
        
        logger.info("")
        logger.info("✓ Window closed successfully")
        return True
    except Exception as e:
        logger.error(f"✗ Error showing window: {e}", exc_info=True)
        return False


def main():
    """Run all tests"""
    logger.info("")
    logger.info("╔" + "=" * 68 + "╗")
    logger.info("║" + " " * 68 + "║")
    logger.info("║" + "  AuroraView - Debug Window Test".center(68) + "║")
    logger.info("║" + " " * 68 + "║")
    logger.info("╚" + "=" * 68 + "╝")
    logger.info("")
    
    results = {}
    
    # Test 1: Import
    results['import'] = test_import()
    if not results['import']:
        logger.error("Cannot continue without successful import")
        return 1
    
    # Test 2: Create
    webview = test_create()
    results['create'] = webview is not None
    if not results['create']:
        logger.error("Cannot continue without successful creation")
        return 1
    
    # Test 3: Load HTML
    results['load_html'] = test_load_html(webview)
    
    # Test 4: Event Handler
    results['event_handler'] = test_event_handler(webview)
    
    # Test 5: Show Window
    results['show'] = test_show(webview)
    
    # Summary
    logger.info("")
    logger.info("=" * 70)
    logger.info("Test Summary")
    logger.info("=" * 70)
    logger.info("")
    
    for test_name, result in results.items():
        status = "✓ PASS" if result else "✗ FAIL"
        logger.info(f"{status}: {test_name}")
    
    logger.info("")
    
    all_passed = all(results.values())
    if all_passed:
        logger.info("✓ All tests passed!")
        logger.info("")
        logger.info("If you saw the window, everything is working correctly!")
        logger.info("If you didn't see the window, there may be a display issue.")
        return 0
    else:
        logger.error("✗ Some tests failed")
        logger.error("")
        logger.error("Failed tests:")
        for test_name, result in results.items():
            if not result:
                logger.error(f"  - {test_name}")
        return 1


if __name__ == "__main__":
    sys.exit(main())

