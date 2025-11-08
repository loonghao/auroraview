#!/usr/bin/env python
"""
Test loading remote URLs in Maya with robust error handling.

This script is designed to work reliably in Maya environment.
"""

import logging
import time

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


def test_remote_url_maya(url="https://www.baidu.com", max_retries=3):
    """
    Test loading a remote URL in Maya with robust error handling.

    Args:
        url: URL to load
        max_retries: Maximum number of retry attempts
    """
    logger.info("=" * 60)
    logger.info("Testing Remote URL in Maya")
    logger.info("=" * 60)
    logger.info(f"URL: {url}")
    logger.info("")

    # Import Maya commands if available
    try:
        import maya.cmds as cmds
        import maya.utils as utils

        in_maya = True
        logger.info("Running in Maya environment")
    except ImportError:
        in_maya = False
        logger.info("Running in standalone mode")

    # Import AuroraView
    from auroraview import WebView

    # Attempt to create WebView with retry logic
    webview = None
    for attempt in range(max_retries):
        try:
            logger.info(f"Attempt {attempt + 1}/{max_retries}: Creating WebView...")

            # Create WebView WITHOUT url parameter
            webview = WebView(title=f"Test: {url}", width=1200, height=800, debug=True)

            logger.info("✓ WebView created successfully")
            break

        except RuntimeError as e:
            error_msg = str(e)
            logger.error(f"✗ Failed to create WebView: {error_msg}")

            if "0x80070057" in error_msg:
                if attempt < max_retries - 1:
                    logger.info("Retrying in 1 second...")
                    time.sleep(1)
                else:
                    logger.error("")
                    logger.error("=" * 60)
                    logger.error("TROUBLESHOOTING STEPS:")
                    logger.error("=" * 60)
                    logger.error("1. Update WebView2 Runtime:")
                    logger.error(
                        "   https://developer.microsoft.com/en-us/microsoft-edge/webview2/"
                    )
                    logger.error("")
                    logger.error("2. Check system resources (close other applications)")
                    logger.error("")
                    logger.error("3. Try restarting Maya")
                    logger.error("")
                    logger.error("4. Run diagnostic tool:")
                    logger.error(
                        '   python -c "from examples.test_maya_remote_url import diagnose; diagnose()"'
                    )
                    logger.error("=" * 60)
                    return None
            else:
                raise

    if not webview:
        logger.error("Failed to create WebView after all retries")
        return None

    # Load initial HTML
    logger.info("Loading initial HTML...")
    webview.load_html("""
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body {
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    height: 100vh;
                    margin: 0;
                    font-family: Arial, sans-serif;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                }
                .loader {
                    text-align: center;
                }
                .spinner {
                    border: 4px solid rgba(255,255,255,0.3);
                    border-top: 4px solid white;
                    border-radius: 50%;
                    width: 40px;
                    height: 40px;
                    animation: spin 1s linear infinite;
                    margin: 0 auto 20px;
                }
                @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }
            </style>
        </head>
        <body>
            <div class="loader">
                <div class="spinner"></div>
                <h2>Loading...</h2>
                <p>Preparing to load remote site</p>
            </div>
        </body>
        </html>
    """)
    logger.info("✓ Initial HTML loaded")

    # Show window (show() now uses show_async() internally - works everywhere)
    logger.info("Showing WebView window...")
    webview.show()
    logger.info("✓ Window shown (non-blocking)")

    # Wait a bit for window to appear
    time.sleep(0.5)

    # Load remote URL
    logger.info(f"Loading remote URL: {url}")
    try:
        webview.load_url(url)
        logger.info("✓ Remote URL loaded successfully")
    except Exception as e:
        logger.error(f"✗ Failed to load remote URL: {e}")
        return webview

    logger.info("")
    logger.info("=" * 60)
    logger.info("SUCCESS!")
    logger.info("=" * 60)
    logger.info("The WebView window should now be visible.")
    logger.info("If you see a white screen, open DevTools (F12) to check for errors.")
    logger.info("=" * 60)

    return webview


def diagnose():
    """Run diagnostic checks for WebView2."""
    import ctypes
    import os
    import platform
    import winreg

    print("=" * 60)
    print("WebView2 Diagnostic Tool")
    print("=" * 60)
    print()

    # Check registry
    print("1. Checking WebView2 Runtime...")
    try:
        key = winreg.OpenKey(
            winreg.HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
            0,
            winreg.KEY_READ,
        )
        version, _ = winreg.QueryValueEx(key, "pv")
        location, _ = winreg.QueryValueEx(key, "location")
        winreg.CloseKey(key)

        print("   ✓ WebView2 Runtime installed")
        print(f"   Version: {version}")
        print(f"   Location: {location}")

        if os.path.exists(location):
            print("   ✓ Runtime files exist")
        else:
            print("   ✗ Runtime files NOT found")

    except FileNotFoundError:
        print("   ✗ WebView2 Runtime NOT installed")
        print("   Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/")

    print()

    # System info
    print("2. System Information...")
    print(f"   OS: {platform.system()} {platform.release()}")
    print(f"   Version: {platform.version()}")
    print(f"   Architecture: {platform.machine()}")

    print()

    # Screen info
    print("3. Screen Information...")
    user32 = ctypes.windll.user32
    width = user32.GetSystemMetrics(0)
    height = user32.GetSystemMetrics(1)
    print(f"   Screen size: {width}x{height}")
    print(f"   Recommended size: {int(width * 0.8)}x{int(height * 0.8)}")

    print()

    # Maya check
    print("4. Maya Environment...")
    try:
        import maya.cmds as cmds

        maya_version = cmds.about(version=True)
        print(f"   ✓ Running in Maya {maya_version}")
    except ImportError:
        print("   ✗ Not running in Maya")

    print()
    print("=" * 60)


def main():
    """Main function for standalone testing."""
    import argparse

    parser = argparse.ArgumentParser(description="Test remote URL loading in Maya")
    parser.add_argument("--url", default="https://www.baidu.com", help="URL to load")
    parser.add_argument("--diagnose", action="store_true", help="Run diagnostic checks")
    parser.add_argument("--retries", type=int, default=3, help="Max retry attempts")

    args = parser.parse_args()

    if args.diagnose:
        diagnose()
    else:
        webview = test_remote_url_maya(args.url, args.retries)

        # Keep the script running if in standalone mode
        try:
            import maya.cmds
        except ImportError:
            if webview:
                input("\nPress Enter to exit...")


if __name__ == "__main__":
    main()
