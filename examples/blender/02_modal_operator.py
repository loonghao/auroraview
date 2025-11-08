"""
Blender WebView Integration - Final Solution

This version uses the new window utilities API to get Blender's window handle,
then creates an embedded WebView with modal operator for event processing.

Key Features:
- Uses auroraview.find_windows_by_title() to get Blender's HWND
- Creates WebView in embedded mode (non-blocking)
- Uses modal operator to drive event loop at 120Hz
- Blender UI remains fully responsive
"""

import sys
from pathlib import Path

# ============================================================================
# PATH SETUP
# ============================================================================
# IMPORTANT: Modify this path to match your project location
PROJECT_ROOT = r"C:\Users\hallo\Documents\augment-projects\dcc_webview"

# Add python directory to sys.path
_python_dir = Path(PROJECT_ROOT) / "python"
if str(_python_dir) not in sys.path:
    sys.path.insert(0, str(_python_dir))
    print(f"[PATH] Added to sys.path: {_python_dir}")

# ============================================================================
# IMPORTS
# ============================================================================
import logging

import bpy

from auroraview import (
    WebView,
    find_windows_by_title,
    get_foreground_window,
)

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# ============================================================================
# GET BLENDER WINDOW HANDLE
# ============================================================================


def get_blender_hwnd():
    """Get Blender's window handle using auroraview window utilities

    Returns:
        int: Window handle (HWND) or None if not found
    """
    logger.info("Searching for Blender window...")

    # Method 1: Try foreground window first
    try:
        foreground = get_foreground_window()
        if foreground:
            logger.info(f"Foreground window: {foreground.title}")
            if "Blender" in foreground.title:
                logger.info("✅ Found Blender as foreground window")
                logger.info(f"   HWND: {foreground.hwnd}, PID: {foreground.pid}")
                return foreground.hwnd
        else:
            logger.info("No foreground window found")
    except Exception as e:
        logger.error(f"Error getting foreground window: {e}")

    # Method 2: Search all windows for Blender
    try:
        logger.info("Searching all windows for 'Blender'...")
        blender_windows = find_windows_by_title("Blender")
        logger.info(f"Found {len(blender_windows)} window(s) matching 'Blender'")

        if blender_windows:
            for i, window in enumerate(blender_windows, 1):
                logger.info(f"  {i}. {window.title} (HWND: {window.hwnd}, PID: {window.pid})")

            window = blender_windows[0]  # Use first match
            logger.info(f"✅ Using first match: {window.title}")
            return window.hwnd
    except Exception as e:
        logger.error(f"Error searching for Blender windows: {e}")

    logger.warning("⚠️ Could not find Blender window")
    return None


# ============================================================================
# MODAL OPERATOR
# ============================================================================


class AuroraViewModalOperator(bpy.types.Operator):
    """Modal operator that processes WebView events without blocking Blender"""

    bl_idname = "auroraview.modal_operator_final"
    bl_label = "AuroraView Modal Operator (Final)"

    _timer = None
    _webview = None

    def modal(self, context, event):
        """Called by Blender's window manager at high frequency"""

        if event.type == "TIMER":
            # Process WebView events
            if self._webview:
                try:
                    # Check if WebView is initialized (non-blocking mode)
                    if hasattr(self._webview, "_async_core"):
                        with self._webview._async_core_lock:
                            if self._webview._async_core is None:
                                # Not initialized yet, skip this tick
                                return {"PASS_THROUGH"}

                    # Process events
                    should_close = self._webview.process_events()

                    if should_close:
                        logger.info("WebView closed, stopping modal operator")
                        self.cancel(context)
                        return {"FINISHED"}

                except Exception as e:
                    logger.error(f"Error processing WebView events: {e}")

        # Check if WebView is still running
        if not self._webview or not self._webview.is_running():
            logger.info("WebView no longer running, stopping modal operator")
            self.cancel(context)
            return {"FINISHED"}

        # Pass through to allow Blender to process other events
        return {"PASS_THROUGH"}

    def execute(self, context):
        """Called when operator starts"""
        logger.info("Starting AuroraView modal operator")

        # Add timer at 120Hz (same as BQT)
        wm = context.window_manager
        self._timer = wm.event_timer_add(1 / 120, window=context.window)
        wm.modal_handler_add(self)

        return {"RUNNING_MODAL"}

    def cancel(self, context):
        """Called when operator stops"""
        logger.info("Stopping AuroraView modal operator")

        if self._timer:
            wm = context.window_manager
            wm.event_timer_remove(self._timer)
            self._timer = None

        # Clean up WebView reference
        self._webview = None


# ============================================================================
# MAIN EXAMPLE
# ============================================================================


def main():
    """Main example function"""

    logger.info("=" * 60)
    logger.info("AuroraView Blender Integration - Final Solution")
    logger.info("=" * 60)

    # Get Blender window handle using new API
    try:
        blender_hwnd = get_blender_hwnd()

        if blender_hwnd is None:
            logger.warning("⚠️ Could not get Blender window handle")
            logger.warning("⚠️ Creating standalone window (may freeze UI)")
            parent = None
        else:
            logger.info(f"✅ Using Blender window as parent: {blender_hwnd}")
            parent = blender_hwnd
    except Exception as e:
        logger.error(f"❌ Error getting Blender window handle: {e}")
        logger.warning("⚠️ Creating standalone window (may freeze UI)")
        parent = None

    # Create WebView
    logger.info("Creating WebView...")
    webview = WebView.create(
        title="Blender WebView (Final Solution)",
        width=800,
        height=600,
        parent=parent_hwnd,  # Auto-selects "owner" mode for cross-thread safety
    )

    # Load HTML content
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <style>
            body {
                margin: 0;
                padding: 20px;
                font-family: Arial, sans-serif;
                background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
                color: white;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                min-height: 100vh;
            }
            h1 {
                font-size: 48px;
                margin-bottom: 20px;
                text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
            }
            .status {
                font-size: 24px;
                margin: 10px 0;
                padding: 10px 20px;
                background: rgba(255,255,255,0.2);
                border-radius: 10px;
            }
            .success {
                color: #4ade80;
                font-weight: bold;
            }
            .feature {
                color: #fbbf24;
                font-weight: bold;
            }
        </style>
    </head>
    <body>
        <h1>🎨 Blender WebView</h1>
        <div class="status feature">✨ Final Solution</div>
        <div class="status">Using Window Utilities API</div>
        <div class="status success">✅ Blender UI Should Be Responsive!</div>
        <div class="status">Try switching workspaces, editing objects, etc.</div>
    </body>
    </html>
    """

    webview.load_html(html_content)
    logger.info("HTML content loaded")

    # Register operator class if not already registered
    if not hasattr(bpy.types, "AURORAVIEW_OT_modal_operator_final"):
        bpy.utils.register_class(AuroraViewModalOperator)

    # Store WebView reference in operator
    AuroraViewModalOperator._webview = webview

    # Start modal operator
    logger.info("Starting modal operator...")
    bpy.ops.auroraview.modal_operator_final()

    # Show WebView in non-blocking mode
    logger.info("Showing WebView...")
    webview.show(wait=False)

    logger.info("=" * 60)
    logger.info("✅ Setup complete!")
    logger.info("=" * 60)
    logger.info("")
    logger.info("IMPORTANT:")
    logger.info("- Blender UI should remain responsive")
    logger.info("- You can switch workspaces, edit objects, etc.")
    logger.info("- Close the WebView window to stop the modal operator")
    logger.info("")
    logger.info("Technical Details:")
    if parent_hwnd:
        logger.info(f"- WebView mode: Embedded (parent HWND: {parent_hwnd})")
    else:
        logger.info("- WebView mode: Standalone (no parent)")
    logger.info("- Event processing: Modal operator at 120Hz")
    logger.info("- Window utilities: auroraview.find_windows_by_title()")
    logger.info("")


# ============================================================================
# RUN
# ============================================================================

if __name__ == "__main__":
    main()
