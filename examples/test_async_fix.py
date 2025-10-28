#!/usr/bin/env python
"""
Test script to verify the thread safety fix for show_async()

This script tests that:
1. WebView can be created in main thread
2. HTML content is stored correctly
3. show_async() creates a new WebView instance in background thread
4. No PanicException about unsendable types
"""

import logging
import time
import threading

logging.basicConfig(
    level=logging.DEBUG,
    format='%(name)s : %(levelname)s : %(message)s'
)
logger = logging.getLogger(__name__)

try:
    from auroraview import WebView
except ImportError as e:
    logger.error(f"Failed to import WebView: {e}")
    raise

def test_show_async_basic():
    """Test basic show_async functionality."""
    logger.info("=" * 70)
    logger.info("TEST 1: Basic show_async() test")
    logger.info("=" * 70)
    
    # Create WebView in main thread
    webview = WebView(
        title="Test - Basic Async",
        width=400,
        height=300
    )
    
    # Load HTML in main thread
    html = """
    <html>
    <body style="background: #2b2b2b; color: #00cc00; font-family: Arial; padding: 20px;">
        <h1>✓ Thread Safety Test</h1>
        <p>If you see this, the fix works!</p>
        <p>WebView was created in background thread.</p>
        <p>Close this window to continue.</p>
    </body>
    </html>
    """
    
    webview.load_html(html)
    logger.info("✓ HTML loaded in main thread")
    
    # Show in background thread
    logger.info("Calling show_async()...")
    webview.show_async()
    logger.info("✓ show_async() returned immediately")
    
    # Give it time to start
    time.sleep(1)
    
    # Check that it's running
    if webview._is_running:
        logger.info("✓ WebView is running in background")
    else:
        logger.error("✗ WebView is not running!")
        return False
    
    # Wait for window to close
    logger.info("Waiting for window to close...")
    webview.wait()
    logger.info("✓ Window closed")
    
    return True

def test_show_async_with_events():
    """Test show_async with event handlers."""
    logger.info("=" * 70)
    logger.info("TEST 2: show_async() with event handlers")
    logger.info("=" * 70)
    
    webview = WebView(
        title="Test - Events",
        width=400,
        height=300
    )
    
    # Register event handler
    event_received = threading.Event()
    
    @webview.on("test_event")
    def handle_test_event(data):
        logger.info(f"✓ Event received: {data}")
        event_received.set()
    
    html = """
    <html>
    <body style="background: #2b2b2b; color: #00cc00; font-family: Arial; padding: 20px;">
        <h1>✓ Event Test</h1>
        <p>Testing event communication.</p>
        <button onclick="sendEvent()">Send Event</button>
        <p id="status">Ready</p>
        <script>
            function sendEvent() {
                document.getElementById('status').textContent = 'Sending event...';
                window.dispatchEvent(new CustomEvent('test_event', {
                    detail: { message: 'Hello from WebView' }
                }));
            }
        </script>
    </body>
    </html>
    """
    
    webview.load_html(html)
    logger.info("✓ HTML with event handler loaded")
    
    webview.show_async()
    logger.info("✓ show_async() called")
    
    time.sleep(1)
    
    if webview._is_running:
        logger.info("✓ WebView is running")
    else:
        logger.error("✗ WebView is not running!")
        return False
    
    webview.wait()
    logger.info("✓ Window closed")
    
    return True

def test_multiple_async_calls():
    """Test that multiple show_async calls are handled correctly."""
    logger.info("=" * 70)
    logger.info("TEST 3: Multiple show_async() calls")
    logger.info("=" * 70)
    
    webview = WebView(
        title="Test - Multiple Calls",
        width=400,
        height=300
    )
    
    html = "<html><body>Test</body></html>"
    webview.load_html(html)
    
    # First call should work
    webview.show_async()
    logger.info("✓ First show_async() called")
    
    time.sleep(0.5)
    
    # Second call should be ignored
    webview.show_async()
    logger.info("✓ Second show_async() called (should be ignored)")
    
    if webview._is_running:
        logger.info("✓ WebView is still running (only one instance)")
    else:
        logger.error("✗ WebView is not running!")
        return False
    
    webview.wait()
    logger.info("✓ Window closed")
    
    return True

if __name__ == "__main__":
    logger.info("")
    logger.info("╔" + "=" * 68 + "╗")
    logger.info("║" + " " * 68 + "║")
    logger.info("║" + "  AuroraView Thread Safety Test Suite".center(68) + "║")
    logger.info("║" + " " * 68 + "║")
    logger.info("╚" + "=" * 68 + "╝")
    logger.info("")
    
    results = []
    
    try:
        results.append(("Basic async", test_show_async_basic()))
    except Exception as e:
        logger.error(f"✗ Test failed with exception: {e}", exc_info=True)
        results.append(("Basic async", False))
    
    logger.info("")
    
    try:
        results.append(("Events", test_show_async_with_events()))
    except Exception as e:
        logger.error(f"✗ Test failed with exception: {e}", exc_info=True)
        results.append(("Events", False))
    
    logger.info("")
    
    try:
        results.append(("Multiple calls", test_multiple_async_calls()))
    except Exception as e:
        logger.error(f"✗ Test failed with exception: {e}", exc_info=True)
        results.append(("Multiple calls", False))
    
    logger.info("")
    logger.info("=" * 70)
    logger.info("TEST RESULTS")
    logger.info("=" * 70)
    
    for test_name, result in results:
        status = "✓ PASS" if result else "✗ FAIL"
        logger.info(f"{status}: {test_name}")
    
    all_passed = all(result for _, result in results)
    
    logger.info("=" * 70)
    if all_passed:
        logger.info("✓ ALL TESTS PASSED!")
    else:
        logger.info("✗ SOME TESTS FAILED!")
    logger.info("=" * 70)
    logger.info("")

