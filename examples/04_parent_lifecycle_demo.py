#!/usr/bin/env python
"""
Example 04: Parent Window Lifecycle Management

This example demonstrates automatic WebView cleanup when the parent window closes.
This is essential for DCC integration (Maya, Houdini, etc.) to prevent orphaned
WebView windows when the DCC application closes.

Features:
- Automatic WebView closure when parent window is destroyed
- Background monitoring of parent window lifecycle
- Clean resource cleanup

Usage:
    python examples/04_parent_lifecycle_demo.py
"""

import logging
import sys

# Setup path to import auroraview
import _setup_path  # noqa: F401

from auroraview import WebView

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


def create_parent_window():
    """Create a simple parent window using tkinter."""
    try:
        import tkinter as tk
        from tkinter import ttk
    except ImportError:
        logger.error("tkinter is required for this example")
        logger.error("Install it with: pip install tk")
        return None, None

    # Create parent window
    root = tk.Tk()
    root.title("Parent Window (Close me to test lifecycle)")
    root.geometry("400x300")

    # Add instructions
    frame = ttk.Frame(root, padding="20")
    frame.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))

    ttk.Label(frame, text="Parent Window Lifecycle Demo", font=("Arial", 16, "bold")).grid(
        row=0, column=0, pady=10
    )

    ttk.Label(
        frame, text="This window simulates a DCC application (Maya, Houdini, etc.)", wraplength=350
    ).grid(row=1, column=0, pady=5)

    ttk.Label(
        frame, text="The WebView window is monitoring this parent window.", wraplength=350
    ).grid(row=2, column=0, pady=5)

    ttk.Label(
        frame,
        text="When you close this window, the WebView will automatically close too!",
        wraplength=350,
        foreground="red",
        font=("Arial", 10, "bold"),
    ).grid(row=3, column=0, pady=10)

    ttk.Label(
        frame,
        text="Try it: Close this window and watch the WebView close automatically.",
        wraplength=350,
        foreground="blue",
    ).grid(row=4, column=0, pady=5)

    # Get window handle (HWND on Windows)
    root.update()  # Ensure window is created
    hwnd = None

    try:
        # On Windows, get HWND
        if sys.platform == "win32":
            hwnd = root.winfo_id()
            logger.info(f"Parent window HWND: 0x{hwnd:x}")
        else:
            logger.warning("Parent window monitoring is only supported on Windows")
    except Exception as e:
        logger.error(f"Failed to get window handle: {e}")

    return root, hwnd


def main():
    """Main function demonstrating parent window lifecycle management."""
    logger.info("=" * 60)
    logger.info("AuroraView - Example 04: Parent Window Lifecycle")
    logger.info("=" * 60)
    logger.info("")

    # Create parent window
    logger.info("Creating parent window...")
    parent_window, parent = create_parent_window()

    if parent_window is None or parent_hwnd is None:
        logger.error("Failed to create parent window")
        return 1

    logger.info(f"[OK] Parent window created with HWND: 0x{parent_hwnd:x}")
    logger.info("")

    # Create WebView with parent
    logger.info("Creating WebView with parent window monitoring...")
    webview = WebView.create(
        title="AuroraView - Child Window",
        width=800,
        height=600,
        parent=parent_hwnd,
        mode="owner",  # Use owner mode for cross-thread safety
        debug=True,
    )
    logger.info("[OK] WebView created")
    logger.info("")

    # Load HTML content
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Parent Lifecycle Demo</title>
        <meta charset="UTF-8">
        <style>
            * { margin: 0; padding: 0; box-sizing: border-box; }
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
                padding: 20px;
            }
            .container {
                background: white;
                border-radius: 12px;
                padding: 40px;
                box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
                max-width: 600px;
                text-align: center;
            }
            h1 {
                color: #333;
                margin-bottom: 20px;
                font-size: 28px;
            }
            .status {
                background: #f0f9ff;
                border: 2px solid #0ea5e9;
                border-radius: 8px;
                padding: 20px;
                margin: 20px 0;
            }
            .status-icon {
                font-size: 48px;
                margin-bottom: 10px;
            }
            .status-text {
                color: #0369a1;
                font-size: 18px;
                font-weight: 600;
            }
            .info {
                background: #fef3c7;
                border-left: 4px solid #f59e0b;
                padding: 15px;
                margin: 20px 0;
                text-align: left;
            }
            .info-title {
                color: #92400e;
                font-weight: 600;
                margin-bottom: 10px;
            }
            .info-text {
                color: #78350f;
                line-height: 1.6;
            }
            .feature-list {
                text-align: left;
                margin: 20px 0;
            }
            .feature-item {
                padding: 10px;
                margin: 5px 0;
                background: #f9fafb;
                border-radius: 6px;
                display: flex;
                align-items: center;
            }
            .feature-icon {
                color: #10b981;
                margin-right: 10px;
                font-size: 20px;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🔗 Parent Window Lifecycle Demo</h1>
            
            <div class="status">
                <div class="status-icon">👀</div>
                <div class="status-text">Monitoring Parent Window</div>
            </div>
            
            <div class="info">
                <div class="info-title">⚠️ How to Test:</div>
                <div class="info-text">
                    Close the parent window (the tkinter window) and watch this WebView 
                    automatically close too! This demonstrates automatic lifecycle management 
                    for DCC integration.
                </div>
            </div>
            
            <div class="feature-list">
                <div class="feature-item">
                    <span class="feature-icon">✅</span>
                    <span>Automatic cleanup when parent closes</span>
                </div>
                <div class="feature-item">
                    <span class="feature-icon">✅</span>
                    <span>Background monitoring (500ms interval)</span>
                </div>
                <div class="feature-item">
                    <span class="feature-icon">✅</span>
                    <span>No orphaned windows</span>
                </div>
                <div class="feature-item">
                    <span class="feature-icon">✅</span>
                    <span>Clean resource release</span>
                </div>
            </div>
            
            <p style="color: #666; margin-top: 20px; font-size: 14px;">
                This feature is essential for DCC applications like Maya, Houdini, 
                Blender, etc. to prevent WebView windows from staying open after 
                the DCC application exits.
            </p>
        </div>
    </body>
    </html>
    """

    logger.info("Loading HTML content...")
    webview.load_html(html_content)
    logger.info("[OK] HTML loaded")
    logger.info("")

    # Show WebView
    logger.info("Showing WebView window...")
    logger.info("")
    logger.info("=" * 60)
    logger.info("INSTRUCTIONS:")
    logger.info("1. You should see TWO windows:")
    logger.info("   - Parent Window (tkinter)")
    logger.info("   - WebView Window (child)")
    logger.info("")
    logger.info("2. Close the PARENT window")
    logger.info("3. Watch the WebView automatically close!")
    logger.info("")
    logger.info("This demonstrates automatic lifecycle management")
    logger.info("for DCC integration (Maya, Houdini, etc.)")
    logger.info("=" * 60)
    logger.info("")

    try:
        # Show WebView (non-blocking)
        webview.show()
        logger.info("WebView window opened")

        # Run parent window event loop
        logger.info("Starting parent window event loop...")
        logger.info("(Close the parent window to test lifecycle management)")
        parent_window.mainloop()

        logger.info("")
        logger.info("=" * 60)
        logger.info("Parent window closed!")
        logger.info("WebView should have closed automatically.")
        logger.info("=" * 60)

    except Exception as e:
        logger.error(f"Error: {e}")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
