# -*- coding: utf-8 -*-
"""AuroraView Browser - Full-featured multi-tab browser.

A Chrome-like browser built with AuroraView's TabManager, featuring:

Architecture (based on Microsoft WebView2Browser):
    - Single UI thread with shared CoreWebView2Environment
    - Controller WebView for browser UI (tab bar, toolbar)
    - Content WebViews managed by show/hide (not create/destroy)
    - Frameless window with custom window controls

Features:
    - Multi-tab browsing (Ctrl+T new tab, Ctrl+W close tab)
    - Navigation controls (back, forward, reload, home)
    - URL bar with smart search/URL detection
    - Bookmarks support
    - DevTools (F12)
    - CDP (Chrome DevTools Protocol) support for automation
    - Frameless window with native-like window controls

Usage:
    # Basic browser
    python examples/agent_browser.py

    # With DevTools enabled
    python examples/agent_browser.py --debug

    # Open specific URLs as tabs
    python examples/agent_browser.py --urls https://google.com https://github.com

    # With CDP debugging (connect via chrome://inspect)
    python examples/agent_browser.py --cdp-port 9222

Reference:
    - Microsoft WebView2Browser: https://github.com/MicrosoftEdge/WebView2Browser

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import argparse
import logging
import sys
from pathlib import Path

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


def parse_args() -> argparse.Namespace:
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(
        description="AuroraView Browser - Full-featured multi-tab browser",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Keyboard Shortcuts:
  Ctrl+T      New tab
  Ctrl+W      Close current tab
  Ctrl+L      Focus URL bar
  F5          Reload page
  Ctrl+R      Reload page
  Alt+Left    Go back
  Alt+Right   Go forward
  Ctrl+D      Toggle bookmark
  F12         Open DevTools

Examples:
  # Basic browser
  python agent_browser.py

  # With multiple initial tabs
  python agent_browser.py --urls https://google.com https://github.com

  # Enable CDP debugging on port 9222
  python agent_browser.py --cdp-port 9222
""",
    )
    parser.add_argument(
        "--title",
        default="AuroraView Browser",
        help="Window title (default: AuroraView Browser)",
    )
    parser.add_argument(
        "--width",
        type=int,
        default=1280,
        help="Window width in pixels (default: 1280)",
    )
    parser.add_argument(
        "--height",
        type=int,
        default=900,
        help="Window height in pixels (default: 900)",
    )
    parser.add_argument(
        "--home",
        default="https://www.google.com",
        help="Home page URL (default: https://www.google.com)",
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Enable DevTools for debugging",
    )
    parser.add_argument(
        "--cdp-port",
        type=int,
        default=0,
        help="CDP remote debugging port (0 = disabled, typical: 9222)",
    )
    parser.add_argument(
        "--urls",
        nargs="*",
        help="URLs to open as initial tabs",
    )
    return parser.parse_args()


def main():
    """Run the AuroraView Browser."""
    args = parse_args()

    print("=" * 60)
    print("AuroraView Browser")
    print("=" * 60)
    print()
    print("Features:")
    print("  - Frameless window with custom title bar")
    print("  - Multi-tab browsing")
    print("  - Navigation controls (back, forward, reload)")
    print("  - URL bar with smart search/URL detection")
    print("  - Bookmarks support")
    print("  - Keyboard shortcuts (Ctrl+T, Ctrl+W, F12, etc.)")
    print()
    print(f"Title: {args.title}")
    print(f"Size: {args.width}x{args.height}")
    print(f"Home URL: {args.home}")
    print(f"Debug: {args.debug}")
    if args.cdp_port > 0:
        print(f"CDP Port: {args.cdp_port}")
        print(f"  Connect via: chrome://inspect")
    if args.urls:
        print(f"Initial URLs: {args.urls}")
    print()
    print("Starting browser...")
    print()

    try:
        from auroraview._core import run_browser

        run_browser(
            title=args.title,
            width=args.width,
            height=args.height,
            home_url=args.home,
            debug=args.debug,
            initial_urls=args.urls,
        )

        logger.info("AuroraView Browser finished")

    except ImportError as e:
        logger.error(f"Failed to import run_browser: {e}")
        logger.error("Make sure auroraview is built: just rebuild-pylib")
        sys.exit(1)
    except Exception as e:
        logger.error(f"Error running browser: {e}", exc_info=True)
        sys.exit(1)


if __name__ == "__main__":
    main()
