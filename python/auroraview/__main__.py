"""AuroraView CLI entry point.

This module provides a pure Python CLI implementation using argparse.
It creates a WebView window using the auroraview Python bindings.
"""

import argparse
import sys
import traceback
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
        "-w",
        "--width",
        type=int,
        default=1024,
        help="Window width in pixels (default: 1024, set to 0 to maximize)",
    )
    parser.add_argument(
        "-H",
        "--height",
        type=int,
        default=768,
        help="Window height in pixels (default: 768, set to 0 to maximize)",
    )
    parser.add_argument("-d", "--debug", action="store_true", help="Enable debug logging")
    parser.add_argument(
        "--allow-new-window",
        action="store_true",
        help="Allow opening new windows (e.g., via window.open)",
    )
    parser.add_argument(
        "--allow-file-protocol",
        action="store_true",
        help="Enable file:// protocol support (allows loading local files from HTML)",
    )
    parser.add_argument(
        "--always-on-top",
        action="store_true",
        help="Keep window always on top",
    )

    args = parser.parse_args()

    try:
        from auroraview import normalize_url
        from auroraview._core import run_standalone

        # Prepare the URL or HTML content
        if args.url:
            # Normalize URL (add https:// if missing)
            url = normalize_url(args.url)
            html_content = None
            html_path = None
            asset_root = None
        else:
            # Read HTML file
            html_file = args.html.resolve()  # Get absolute path
            if not html_file.exists():
                print(f"Error: HTML file not found: {html_file}", file=sys.stderr)
                sys.exit(1)

            # Read HTML content (rewriting is now handled in Rust)
            html_content = html_file.read_text(encoding="utf-8")
            html_path = str(html_file)
            url = None

            # Determine asset_root: explicit or derive from HTML file location
            if args.assets_root:
                asset_root = str(args.assets_root.resolve())
            else:
                # Auto-derive from HTML file location
                asset_root = str(html_file.parent)

        # Run standalone WebView (blocking until window closes, then exits process)
        # This uses the same event_loop.run() approach as the Rust CLI
        run_standalone(
            title=args.title,
            width=args.width,
            height=args.height,
            url=url,
            html=html_content,
            dev_tools=args.debug,
            allow_new_window=args.allow_new_window,
            allow_file_protocol=args.allow_file_protocol,
            always_on_top=args.always_on_top,
            asset_root=asset_root,
            html_path=html_path,
            rewrite_relative_paths=True,  # Automatically rewrite relative paths
        )

        # This line will never be reached because run_standalone() exits the process
        # when the window closes

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
            traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
