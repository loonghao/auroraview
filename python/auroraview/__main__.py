"""AuroraView CLI entry point.

This module provides a pure Python CLI implementation using argparse.
It creates a WebView window using the auroraview Python bindings.
"""

import argparse
import sys
from pathlib import Path


def main():
    """Main entry point for the CLI.

    This function provides a pure Python implementation of the CLI
    that works with uvx and other Python package managers.
    """
    parser = argparse.ArgumentParser(
        prog="auroraview",
        description="Launch a WebView window with a URL or local HTML file",
    )

    # URL or HTML file (mutually exclusive)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("-u", "--url", type=str, help="URL to load in the WebView")
    group.add_argument("-f", "--html", type=Path, help="Local HTML file to load in the WebView")

    # Optional arguments
    parser.add_argument(
        "--assets-root",
        type=Path,
        help="Assets root directory for local HTML files (defaults to HTML file's directory)",
    )
    parser.add_argument(
        "-t", "--title", type=str, default="AuroraView", help="Window title (default: AuroraView)"
    )
    parser.add_argument(
        "-w", "--width", type=int, default=1024, help="Window width in pixels (default: 1024)"
    )
    parser.add_argument(
        "-H", "--height", type=int, default=768, help="Window height in pixels (default: 768)"
    )
    parser.add_argument("-d", "--debug", action="store_true", help="Enable debug logging")

    args = parser.parse_args()

    try:
        from auroraview import WebView, normalize_url, rewrite_html_for_custom_protocol

        # Prepare the URL or HTML content
        if args.url:
            # Normalize URL (add https:// if missing)
            url = normalize_url(args.url)
            html_content = None
        else:
            # Read HTML file
            html_file = args.html
            if not html_file.exists():
                print(f"Error: HTML file not found: {html_file}", file=sys.stderr)
                sys.exit(1)

            # Read and rewrite HTML for custom protocol support
            raw_html = html_file.read_text(encoding="utf-8")
            html_content = rewrite_html_for_custom_protocol(raw_html)
            url = None

        # Create WebView with content
        webview = WebView(
            title=args.title,
            width=args.width,
            height=args.height,
            url=url,
            html=html_content,
            debug=args.debug,
        )

        # Show the WebView (blocking mode for CLI)
        webview.show_blocking()

    except ImportError as e:
        print(
            "Error: Failed to import auroraview module.",
            file=sys.stderr,
        )
        print(
            f"Details: {e}",
            file=sys.stderr,
        )
        print(
            "Please ensure the package is properly installed with: pip install auroraview",
            file=sys.stderr,
        )
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        if args.debug:
            import traceback

            traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
