"""Example: Creating a custom timer backend for Maya.

This example demonstrates how to create a custom timer backend
for Maya's scriptJob system and register it globally.
"""

from typing import Any, Callable

from auroraview import WebView, register_timer_backend
from auroraview.timer_backends import TimerBackend


class MayaTimerBackend(TimerBackend):
    """Maya scriptJob-based timer backend.

    This backend uses Maya's scriptJob idle event for timing.
    It's optimized for Maya's event loop and provides better
    integration than the generic thread-based backend.
    """

    def is_available(self) -> bool:
        """Check if Maya is available."""
        try:
            import maya.cmds  # noqa: F401

            return True
        except ImportError:
            return False

    def start(self, interval_ms: int, callback: Callable[[], None]) -> Any:
        """Start Maya scriptJob.

        Note: Maya's idle event fires very frequently, so we ignore
        the interval_ms parameter. If you need throttling, implement
        it in your callback.

        Args:
            interval_ms: Ignored (Maya idle events fire at their own rate)
            callback: Function to call on each idle event

        Returns:
            scriptJob ID
        """
        import maya.cmds as cmds

        # Create scriptJob that runs on idle events
        job_id = cmds.scriptJob(event=["idle", callback])
        return job_id

    def stop(self, handle: Any) -> None:
        """Stop Maya scriptJob.

        Args:
            handle: scriptJob ID returned by start()
        """
        import maya.cmds as cmds

        if cmds.scriptJob(exists=handle):
            cmds.scriptJob(kill=handle, force=True)


# Register the Maya backend with high priority
# This ensures it's used instead of Qt or Thread backends in Maya
register_timer_backend(MayaTimerBackend, priority=200)


def create_maya_webview():
    """Create a WebView in Maya with automatic Maya timer backend.

    The WebView will automatically use the MayaTimerBackend because
    it was registered with high priority (200).
    """
    import maya.OpenMayaUI as omui

    # Get Maya main window handle
    maya_window = int(omui.MQtUtil.mainWindow())

    # Create WebView - it will automatically use MayaTimerBackend
    webview = WebView.create(
        title="Maya Tool",
        url="http://localhost:3000",
        parent=maya_window,
        mode="owner",
    )

    # The EventTimer will automatically select MayaTimerBackend
    # because it has the highest priority and is available
    webview.show()

    return webview


def create_maya_webview_explicit():
    """Create a WebView with explicit backend selection.

    This shows how to explicitly provide a backend instance
    instead of relying on automatic selection.
    """
    import maya.OpenMayaUI as omui

    from auroraview.event_timer import EventTimer

    maya_window = int(omui.MQtUtil.mainWindow())

    webview = WebView.create(
        title="Maya Tool",
        url="http://localhost:3000",
        parent=maya_window,
        mode="owner",
    )

    # Explicitly create and use Maya backend
    backend = MayaTimerBackend()
    timer = EventTimer(webview, interval_ms=16, backend=backend)

    # Register callbacks
    @timer.on_close
    def on_close():
        print("WebView closed")
        timer.stop()

    timer.start()
    webview.show()

    return webview, timer


if __name__ == "__main__":
    # In Maya, just call:
    # webview = create_maya_webview()

    # Or with explicit backend:
    # webview, timer = create_maya_webview_explicit()

    print("This example is meant to be run inside Maya")
    print("Copy the functions to Maya's script editor and run them")

