"""Standalone test for auroraview.

This script can be run in standalone Python to test the WebView functionality
without requiring a DCC application.

Usage:
    uv run python examples/standalone_test.py
"""

import sys
import logging

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)


def test_basic_import():
    """Test basic import."""
    logger.info("Testing basic import...")
    try:
        import auroraview
        logger.info(f"✓ Successfully imported auroraview v{auroraview.__version__}")
        return True
    except ImportError as e:
        logger.error(f"✗ Failed to import auroraview: {e}")
        return False


def test_webview_creation():
    """Test WebView creation."""
    logger.info("Testing WebView creation...")
    try:
        from auroraview import WebView
        
        webview = WebView(
            title="Standalone Test",
            width=800,
            height=600,
        )
        logger.info(f"✓ Successfully created WebView: {webview}")
        return True
    except Exception as e:
        logger.error(f"✗ Failed to create WebView: {e}")
        import traceback
        traceback.print_exc()
        return False


def test_html_loading():
    """Test HTML loading."""
    logger.info("Testing HTML loading...")
    try:
        from auroraview import WebView
        
        html_content = """
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    height: 100vh;
                    margin: 0;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                }
                .container {
                    text-align: center;
                    color: white;
                }
                h1 {
                    font-size: 3em;
                    margin-bottom: 0.5em;
                }
                p {
                    font-size: 1.2em;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>🚀 AuroraView</h1>
                <p>Standalone Test - Running Successfully!</p>
                <p id="time"></p>
            </div>
            <script>
                function updateTime() {
                    document.getElementById('time').textContent = 
                        new Date().toLocaleTimeString();
                }
                setInterval(updateTime, 1000);
                updateTime();
            </script>
        </body>
        </html>
        """
        
        webview = WebView(
            title="AuroraView - Standalone Test",
            width=800,
            height=600,
        )
        webview.load_html(html_content)
        logger.info("✓ Successfully loaded HTML content")
        
        # Note: show() will block, so we don't call it in automated tests
        # webview.show()
        
        return True
    except Exception as e:
        logger.error(f"✗ Failed to load HTML: {e}")
        import traceback
        traceback.print_exc()
        return False


def test_event_system():
    """Test event system."""
    logger.info("Testing event system...")
    try:
        from auroraview import WebView
        
        webview = WebView(title="Event Test")
        
        # Test event registration
        @webview.on("test_event")
        def handle_test(data):
            logger.info(f"Received event data: {data}")
        
        logger.info("✓ Successfully registered event handler")
        
        # Test event emission
        webview.emit("test_event", {"message": "Hello from Python!"})
        logger.info("✓ Successfully emitted event")
        
        return True
    except Exception as e:
        logger.error(f"✗ Failed event system test: {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    """Run all tests."""
    logger.info("=" * 60)
    logger.info("AuroraView - Standalone Test Suite")
    logger.info("=" * 60)
    
    tests = [
        ("Import Test", test_basic_import),
        ("WebView Creation", test_webview_creation),
        ("HTML Loading", test_html_loading),
        ("Event System", test_event_system),
    ]
    
    results = []
    for name, test_func in tests:
        logger.info("")
        logger.info(f"Running: {name}")
        logger.info("-" * 60)
        result = test_func()
        results.append((name, result))
    
    # Summary
    logger.info("")
    logger.info("=" * 60)
    logger.info("Test Summary")
    logger.info("=" * 60)
    
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    for name, result in results:
        status = "✓ PASS" if result else "✗ FAIL"
        logger.info(f"{status}: {name}")
    
    logger.info("")
    logger.info(f"Results: {passed}/{total} tests passed")
    
    if passed == total:
        logger.info("🎉 All tests passed!")
        return 0
    else:
        logger.error(f"❌ {total - passed} test(s) failed")
        return 1


if __name__ == "__main__":
    sys.exit(main())

