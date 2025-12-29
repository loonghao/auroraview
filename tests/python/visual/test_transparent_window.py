#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Visual test for transparent window functionality.

This test uses screenshot comparison to verify that transparent windows
are truly transparent (not showing white/gray background).

Run with:
    uv run python tests/python/visual/test_transparent_window.py
"""

from __future__ import annotations

import sys
from pathlib import Path

# Add project root to path
PROJECT_ROOT = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(PROJECT_ROOT / "python"))


# Minimal HTML for transparent test - just a colored circle
TRANSPARENT_TEST_HTML = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        html, body {
            width: 100%;
            height: 100%;
            background: transparent !important;
            overflow: hidden;
        }

        .container {
            width: 100%;
            height: 100%;
            display: flex;
            align-items: center;
            justify-content: center;
            background: transparent !important;
        }

        .circle {
            width: 80px;
            height: 80px;
            border-radius: 50%;
            background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
            box-shadow: 0 4px 20px rgba(99, 102, 241, 0.5);
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            font-weight: bold;
            font-size: 14px;
            cursor: pointer;
        }

        .circle:hover {
            transform: scale(1.1);
            transition: transform 0.2s;
        }

        /* Debug info */
        .debug {
            position: fixed;
            bottom: 5px;
            left: 5px;
            font-size: 10px;
            color: rgba(255,255,255,0.5);
            font-family: monospace;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="circle" id="btn" onclick="onClick()">
            TEST
        </div>
    </div>
    <div class="debug" id="debug">Loading...</div>

    <script>
        let clickCount = 0;

        function onClick() {
            clickCount++;
            document.getElementById('debug').textContent = 'Clicks: ' + clickCount;

            if (window.auroraview && window.auroraview.call) {
                window.auroraview.call('on_click', { count: clickCount });
            }
        }

        // Report transparency status
        window.addEventListener('load', function() {
            const body = document.body;
            const computedStyle = window.getComputedStyle(body);
            const bgColor = computedStyle.backgroundColor;

            // Check if background is truly transparent
            const isTransparent = bgColor === 'rgba(0, 0, 0, 0)' ||
                                  bgColor === 'transparent' ||
                                  bgColor === '';

            document.getElementById('debug').textContent =
                'BG: ' + bgColor + ' | Transparent: ' + isTransparent;

            console.log('[TransparencyTest] Body background:', bgColor);
            console.log('[TransparencyTest] Is transparent:', isTransparent);

            if (window.auroraview && window.auroraview.call) {
                window.auroraview.call('report_status', {
                    background: bgColor,
                    isTransparent: isTransparent
                });
            }
        });
    </script>
</body>
</html>
"""


def run_transparency_test():
    """Run the transparency test."""
    from auroraview import AuroraView

    print("\n" + "=" * 60)
    print("Transparent Window Test")
    print("=" * 60)
    print("\nThis test creates a transparent window with a purple circle.")
    print("If the window is truly transparent, you should see your desktop")
    print("through the window (except for the purple circle).")
    print("\nExpected: Purple circle floating on desktop")
    print("Problem: White/gray background visible around the circle")
    print("=" * 60 + "\n")

    class TransparentTest(AuroraView):
        """Test transparent window."""

        def __init__(self):
            super().__init__(
                html=TRANSPARENT_TEST_HTML,
                width=120,
                height=120,
                frame=False,
                transparent=True,
                undecorated_shadow=False,  # Critical for transparency
                always_on_top=True,
                tool_window=True,
            )
            self.bind_call("on_click", self.on_click)
            self.bind_call("report_status", self.report_status)

        def on_click(self, count: int = 0):
            """Handle click."""
            print(f"[Test] Click count: {count}")
            return {"ok": True}

        def report_status(self, background: str = "", isTransparent: bool = False):
            """Report transparency status from JS."""
            print(f"[Test] JS reports - Background: {background}")
            print(f"[Test] JS reports - Is Transparent: {isTransparent}")

            if not isTransparent:
                print("\n[WARNING] JavaScript reports background is NOT transparent!")
                print("This may indicate a WebView2 configuration issue.")

            return {"ok": True}

    test = TransparentTest()
    test.show()


def run_comparison_test():
    """Run comparison test between different configurations."""
    from auroraview import AuroraView

    print("\n" + "=" * 60)
    print("Transparency Configuration Comparison Test")
    print("=" * 60)
    print("\nCreating 3 windows with different configurations:")
    print("  1. transparent=True, undecorated_shadow=False (should be transparent)")
    print("  2. transparent=True, undecorated_shadow=True (may have issues)")
    print("  3. transparent=False (should have solid background)")
    print("=" * 60 + "\n")

    windows = []

    # Config 1: Correct transparent config
    class Window1(AuroraView):
        def __init__(self):
            html = TRANSPARENT_TEST_HTML.replace("TEST", "W1")
            super().__init__(
                html=html,
                width=100,
                height=100,
                frame=False,
                transparent=True,
                undecorated_shadow=False,
                always_on_top=True,
                tool_window=True,
                x=100,
                y=100,
            )

    # Config 2: Transparent but with shadow
    class Window2(AuroraView):
        def __init__(self):
            html = TRANSPARENT_TEST_HTML.replace("TEST", "W2")
            super().__init__(
                html=html,
                width=100,
                height=100,
                frame=False,
                transparent=True,
                undecorated_shadow=True,  # This might cause issues
                always_on_top=True,
                tool_window=True,
                x=220,
                y=100,
            )

    # Config 3: Not transparent
    class Window3(AuroraView):
        def __init__(self):
            html = TRANSPARENT_TEST_HTML.replace("TEST", "W3").replace(
                "background: transparent !important;", "background: #1a1a2e !important;"
            )
            super().__init__(
                html=html,
                width=100,
                height=100,
                frame=False,
                transparent=False,
                undecorated_shadow=True,
                always_on_top=True,
                tool_window=True,
                x=340,
                y=100,
            )

    print("Creating Window 1 (transparent=True, shadow=False)...")
    w1 = Window1()
    windows.append(w1)

    print("Creating Window 2 (transparent=True, shadow=True)...")
    w2 = Window2()
    windows.append(w2)

    print("Creating Window 3 (transparent=False)...")
    w3 = Window3()
    windows.append(w3)

    print("\nShowing all windows...")
    for w in windows:
        w.show()


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Transparent window test")
    parser.add_argument(
        "--compare", action="store_true", help="Run comparison test with multiple configurations"
    )
    args = parser.parse_args()

    if args.compare:
        run_comparison_test()
    else:
        run_transparency_test()
