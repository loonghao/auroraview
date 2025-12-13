#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""DCC Integration Example - Shows how to integrate AuroraView with DCC applications.

This example demonstrates best practices for integrating AuroraView with
Digital Content Creation (DCC) applications like Maya, Houdini, and Blender.

Key features demonstrated:
- Non-blocking event loop integration
- Qt timer-based event processing
- Window lifecycle management
- Proper cleanup on DCC shutdown

Recommended APIs:
- QtWebView: For Qt-based DCC apps (Maya, Houdini, Nuke, 3ds Max)
- AuroraView: For HWND-based apps (Unreal Engine)
- run_desktop: For standalone desktop applications
"""

from typing import Optional

from auroraview import WebView
from auroraview.core.events import WindowEventData
from auroraview.utils.event_timer import EventTimer


class DCCWebViewPanel:
    """A WebView panel designed for DCC application integration.

    This class wraps WebView with DCC-specific functionality:
    - Uses Qt timer for event processing (if available)
    - Handles DCC shutdown gracefully
    - Provides window state tracking
    """

    def __init__(
        self,
        title: str = "AuroraView Panel",
        width: int = 800,
        height: int = 600,
        timer_interval: int = 16,  # ~60 FPS
    ):
        """Initialize the DCC WebView panel.

        Args:
            title: Window title
            width: Initial window width
            height: Initial window height
            timer_interval: Event processing interval in milliseconds
        """
        self.title = title
        self.width = width
        self.height = height
        self.timer_interval = timer_interval

        self._webview: Optional[WebView] = None
        self._timer: Optional[EventTimer] = None
        self._is_visible = False
        self._is_focused = False

    def create(self, html_content: Optional[str] = None, url: Optional[str] = None):
        """Create and show the WebView panel.

        Args:
            html_content: HTML content to load (optional)
            url: URL to load (optional, used if html_content is None)
        """
        # Create WebView
        self._webview = WebView(
            title=self.title,
            width=self.width,
            height=self.height,
            resizable=True,
        )

        # Register window event handlers
        self._setup_event_handlers()

        # Load content
        if html_content:
            self._webview.load_html(html_content)
        elif url:
            self._webview.load_url(url)
        else:
            self._webview.load_html(self._default_html())

        # Create event timer for non-blocking operation
        self._timer = EventTimer(
            webview=self._webview,
            interval=self.timer_interval,
            check_window_validity=True,
        )

        # Start the timer (uses Qt timer if available, falls back to threading)
        self._timer.start()

        print(f"[DCCWebViewPanel] Created panel: {self.title}")

    def _setup_event_handlers(self):
        """Set up window event handlers."""
        if not self._webview:
            return

        @self._webview.on_shown
        def on_shown(data: WindowEventData):
            self._is_visible = True
            print("[DCCWebViewPanel] Window shown")

        @self._webview.on_hidden
        def on_hidden(data: WindowEventData):
            self._is_visible = False
            print("[DCCWebViewPanel] Window hidden")

        @self._webview.on_focused
        def on_focused(data: WindowEventData):
            self._is_focused = True
            print("[DCCWebViewPanel] Window focused")

        @self._webview.on_blurred
        def on_blurred(data: WindowEventData):
            self._is_focused = False
            print("[DCCWebViewPanel] Window blurred")

        @self._webview.on_resized
        def on_resized(data: WindowEventData):
            self.width = data.width or self.width
            self.height = data.height or self.height
            print(f"[DCCWebViewPanel] Resized to {self.width}x{self.height}")

        @self._webview.on_closing
        def on_closing(data: WindowEventData):
            print("[DCCWebViewPanel] Window closing...")
            self.destroy()
            return True

    def destroy(self):
        """Clean up and destroy the panel."""
        if self._timer:
            self._timer.stop()
            self._timer = None

        if self._webview:
            self._webview.close()
            self._webview = None

        print(f"[DCCWebViewPanel] Panel destroyed: {self.title}")

    def _default_html(self) -> str:
        """Return default HTML content."""
        return """
        <!DOCTYPE html>
        <html>
        <head>
            <title>DCC Panel</title>
            <style>
                body { font-family: Arial; padding: 20px; background: #2d2d2d; color: #fff; }
                h1 { color: #00d4ff; }
            </style>
        </head>
        <body>
            <h1>AuroraView DCC Panel</h1>
            <p>This panel is integrated with your DCC application.</p>
        </body>
        </html>
        """

    @property
    def is_visible(self) -> bool:
        """Check if the panel is visible."""
        return self._is_visible

    @property
    def is_focused(self) -> bool:
        """Check if the panel is focused."""
        return self._is_focused

    @property
    def webview(self) -> Optional[WebView]:
        """Get the underlying WebView instance."""
        return self._webview

    def show(self):
        """Show the WebView panel."""
        if self._webview:
            self._webview.show()


def main():
    """Run the DCC integration example.

    This demonstrates how to create a WebView panel that integrates
    with DCC applications using non-blocking event processing.
    """
    print("DCC Integration Example")
    print("=" * 50)
    print("This example shows how to integrate AuroraView with DCC apps.")
    print()
    print("For real DCC integration, use:")
    print("  - QtWebView: For Qt-based DCC apps (Maya, Houdini, Nuke)")
    print("  - AuroraView: For HWND-based apps (Unreal Engine)")
    print()

    # Create and show the panel
    panel = DCCWebViewPanel(
        title="DCC Integration Demo",
        width=800,
        height=600,
    )

    # Create with default HTML
    panel.create()

    # Show the panel
    panel.show()


if __name__ == "__main__":
    main()

