#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""AuroraView Inspector Demo.

This example demonstrates the Inspector API for AI-friendly testing
and automation of AuroraView WebView applications.

Prerequisites:
    1. Start an AuroraView app with CDP enabled:
       auroraview-gallery --devtools --devtools-port 9222

    2. Or run the multi-tab browser demo:
       python examples/multi_tab_browser_demo.py

Usage:
    python examples/inspector_demo.py

Environment variables:
    AURORAVIEW_CDP_ENDPOINT - CDP endpoint URL (default: http://localhost:9222)
"""

from __future__ import annotations

import os
import sys


def main():
    """Run Inspector demo."""
    from auroraview.testing import Inspector, _RUST_BACKEND

    # Show backend info
    print(f"Inspector backend: {'Rust' if _RUST_BACKEND else 'Python (Playwright)'}")
    print()

    # Get CDP endpoint
    endpoint = os.environ.get("AURORAVIEW_CDP_ENDPOINT", "http://localhost:9222")
    print(f"Connecting to: {endpoint}")

    try:
        # Connect to running instance
        with Inspector.connect(endpoint) as page:
            print("Connected!")
            print()

            # === 1. Get Page Snapshot ===
            print("=" * 60)
            print("1. PAGE SNAPSHOT")
            print("=" * 60)

            snap = page.snapshot()
            print(f"Title: {snap.title}")
            print(f"URL: {snap.url}")
            print(f"Viewport: {snap.viewport[0]}x{snap.viewport[1]}")
            print(f"Interactive elements: {snap.ref_count()} refs")
            print()

            # Show refs
            if snap.ref_count() > 0:
                print("Interactive Elements:")
                for ref_id, ref_info in list(snap.refs.items())[:10]:  # Show first 10
                    print(f"  {ref_id}  [{ref_info.role}] \"{ref_info.name}\"")
                if snap.ref_count() > 10:
                    print(f"  ... and {snap.ref_count() - 10} more")
            print()

            # === 2. Find Elements ===
            print("=" * 60)
            print("2. FIND ELEMENTS")
            print("=" * 60)

            # Find buttons
            buttons = snap.find("button")
            print(f"Found {len(buttons)} elements containing 'button'")
            for ref in buttons[:3]:
                print(f"  {ref.ref_id}: {ref.name}")
            print()

            # === 3. Take Screenshot ===
            print("=" * 60)
            print("3. SCREENSHOT")
            print("=" * 60)

            png_bytes = page.screenshot()
            print(f"Screenshot: {len(png_bytes)} bytes")

            # Optionally save
            # with open("screenshot.png", "wb") as f:
            #     f.write(png_bytes)
            print()

            # === 4. Execute JavaScript ===
            print("=" * 60)
            print("4. JAVASCRIPT EVALUATION")
            print("=" * 60)

            result = page.eval("document.title")
            print(f"document.title = \"{result}\"")

            result = page.eval("window.innerWidth + 'x' + window.innerHeight")
            print(f"viewport = {result}")

            result = page.eval("navigator.userAgent")
            print(f"userAgent = {result[:50]}...")
            print()

            # === 5. Wait Conditions ===
            print("=" * 60)
            print("5. WAIT CONDITIONS")
            print("=" * 60)

            # Wait for network idle
            success = page.wait("idle", timeout=5.0)
            print(f"Network idle: {success}")

            # Wait for DOM loaded
            success = page.wait("loaded", timeout=5.0)
            print(f"DOM loaded: {success}")
            print()

            # === 6. Interaction Demo (Optional) ===
            print("=" * 60)
            print("6. INTERACTION (DEMO)")
            print("=" * 60)

            # Press Escape (safe, no side effects)
            result = page.press("Escape")
            print(f"Press Escape: {result}")

            # If there are interactive elements, show how to interact
            if snap.ref_count() > 0:
                first_ref = next(iter(snap.refs.keys()))
                print(f"\nTo click element {first_ref}:")
                print(f"  page.click(\"{first_ref}\")")
                print(f"\nTo fill a textbox:")
                print(f"  page.fill(\"{first_ref}\", \"your text\")")
            print()

            # === 7. Snapshot Formats ===
            print("=" * 60)
            print("7. SNAPSHOT FORMATS")
            print("=" * 60)

            # Text format (AI-friendly)
            text = snap.to_text()
            print("Text format (first 500 chars):")
            print(text[:500])
            if len(text) > 500:
                print("...")
            print()

            # JSON format
            json_str = snap.to_json()
            print(f"JSON format: {len(json_str)} characters")
            print("(Use snap.to_json() for structured data)")
            print()

            print("=" * 60)
            print("Demo complete!")
            print("=" * 60)

    except Exception as e:
        print(f"Error: {e}")
        print()
        print("Make sure:")
        print("  1. An AuroraView app is running with CDP enabled")
        print("  2. The CDP endpoint is accessible")
        print()
        print("Example commands to start an app:")
        print("  auroraview-gallery --devtools --devtools-port 9222")
        print("  python examples/multi_tab_browser_demo.py")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
