# -*- coding: utf-8 -*-
"""High-level browser API with multi-tab support.

This module provides a complete tabbed browser experience with
tab management, navigation controls, and customizable UI.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional

if TYPE_CHECKING:
    from auroraview.core.webview import WebView


logger = logging.getLogger(__name__)


class Browser:
    """High-level browser API with multi-tab support.

    Provides a complete tabbed browser experience with:
    - Tab management (create, close, switch)
    - Navigation controls (back, forward, reload)
    - URL bar and search
    - Customizable UI

    Example:
        >>> browser = Browser(title="My Browser")
        >>> browser.new_tab("https://google.com")
        >>> browser.new_tab("https://github.com")
        >>> browser.show(wait=True)  # Blocking

    For DCC integration:
        >>> browser = Browser(parent=maya_widget)
        >>> browser.new_tab("https://docs.autodesk.com")
        >>> browser.show()  # Non-blocking
    """

    def __init__(
        self,
        title: str = "AuroraView Browser",
        width: int = 1200,
        height: int = 800,
        debug: bool = False,
        parent: Any = None,
        default_url: str = "about:blank",
    ):
        """Initialize Browser.

        Args:
            title: Window title
            width: Window width
            height: Window height
            debug: Enable developer tools
            parent: Parent widget for DCC integration
            default_url: Default URL for new tabs
        """
        self.title = title
        self.width = width
        self.height = height
        self.debug = debug
        self.parent = parent
        self.default_url = default_url

        # Internal state
        self._tabs: List[Dict[str, Any]] = []
        self._active_tab_id: Optional[str] = None
        self._tab_counter = 0
        self._webview: Optional["WebView"] = None
        self._running = False
        self._on_ready_callbacks: List[Callable[["Browser"], None]] = []

    def new_tab(self, url: str = "", title: str = "New Tab") -> Dict[str, Any]:
        """Create a new tab.

        Args:
            url: Initial URL
            title: Tab title

        Returns:
            The created tab info dict
        """
        self._tab_counter += 1
        tab_id = f"tab_{self._tab_counter}"
        tab = {
            "id": tab_id,
            "url": url or self.default_url,
            "title": title,
            "canGoBack": False,
            "canGoForward": False,
            "loading": False,
        }
        self._tabs.append(tab)

        # If this is the first tab, activate it
        if len(self._tabs) == 1:
            self._active_tab_id = tab_id

        # Update UI
        self._sync_tabs_to_ui()
        return tab

    def close_tab(self, tab_id: Optional[str] = None) -> None:
        """Close a tab.

        Args:
            tab_id: Tab ID (closes active tab if None)
        """
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return

        # Find and remove tab
        for i, tab in enumerate(self._tabs):
            if tab["id"] == tab_id:
                self._tabs.pop(i)
                break

        # If closed active tab, activate another
        if tab_id == self._active_tab_id:
            if self._tabs:
                self._active_tab_id = self._tabs[0]["id"]
            else:
                self._active_tab_id = None

        self._sync_tabs_to_ui()

    def activate_tab(self, tab_id: str) -> None:
        """Activate a tab by ID."""
        for tab in self._tabs:
            if tab["id"] == tab_id:
                self._active_tab_id = tab_id
                self._sync_tabs_to_ui()
                # Navigate to tab's URL
                if self._webview and tab["url"]:
                    self._webview.load_url(tab["url"])
                break

    def navigate(self, url: str, tab_id: Optional[str] = None) -> None:
        """Navigate a tab to a URL.

        Args:
            url: URL to navigate to
            tab_id: Tab ID (uses active tab if None)
        """
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return

        # Update tab URL
        for tab in self._tabs:
            if tab["id"] == tab_id:
                tab["url"] = url
                tab["loading"] = True
                break

        # If active tab, navigate WebView
        if tab_id == self._active_tab_id and self._webview:
            self._webview.load_url(url)

        self._sync_tabs_to_ui()

    def go_back(self) -> None:
        """Go back in the active tab."""
        if self._webview:
            self._webview.go_back()

    def go_forward(self) -> None:
        """Go forward in the active tab."""
        if self._webview:
            self._webview.go_forward()

    def reload(self) -> None:
        """Reload the active tab."""
        if self._webview:
            self._webview.reload()

    def get_tabs(self) -> List[Dict[str, Any]]:
        """Get all tabs."""
        return self._tabs.copy()

    def get_active_tab(self) -> Optional[Dict[str, Any]]:
        """Get the active tab."""
        for tab in self._tabs:
            if tab["id"] == self._active_tab_id:
                return tab
        return None

    def on_ready(self, callback: Callable[["Browser"], None]) -> None:
        """Register a callback for when browser is ready.

        Args:
            callback: Function to call when browser is ready
        """
        if self._running:
            callback(self)
        else:
            self._on_ready_callbacks.append(callback)

    def show(self, wait: bool = True) -> None:
        """Show the browser.

        Args:
            wait: If True, block until window closes
        """
        from auroraview.core.packed import is_packed_mode, run_api_server, send_set_html

        if is_packed_mode():
            # In packed mode, Rust controls the WebView.
            # We send the dynamic HTML via IPC and run as API server.
            logger.info("Browser running in packed mode")

            # Create a mock webview for API registration
            self._create_packed_webview()
            self._running = True

            # Send the browser HTML to Rust WebView
            html = self._get_browser_html()
            send_set_html(html, title=self.title)

            # Run ready callbacks
            for callback in self._on_ready_callbacks:
                try:
                    callback(self)
                except Exception as e:
                    logger.warning(f"Ready callback error: {e}")

            # Run as API server (this blocks)
            run_api_server(self._webview)
        else:
            # Normal mode: create WebView directly
            self._create_webview()
            self._running = True

            for callback in self._on_ready_callbacks:
                try:
                    callback(self)
                except Exception as e:
                    logger.warning(f"Ready callback error: {e}")

            self._webview.show(wait=wait)

    def run(self) -> None:
        """Run the browser (blocking). Alias for show(wait=True)."""
        self.show(wait=True)

    def close(self) -> None:
        """Close the browser."""
        if self._webview:
            self._webview.close()
        self._running = False

    def _create_webview(self) -> None:
        """Create the browser WebView (normal mode)."""
        from auroraview import create_webview

        self._webview = create_webview(
            title=self.title,
            html=self._get_browser_html(),
            width=self.width,
            height=self.height,
            debug=self.debug,
            parent=self.parent,
            auto_show=False,
        )

        self._setup_api()

    def _create_packed_webview(self) -> None:
        """Create a mock WebView for packed mode API registration.

        In packed mode, Rust controls the actual WebView. This method creates
        a lightweight Python object that can hold bound functions for the
        API server.
        """
        from auroraview import create_webview

        # Create a minimal webview object for API registration
        # In packed mode, this won't actually show a window
        self._webview = create_webview(
            title=self.title,
            html="",  # Empty, we'll set HTML via IPC
            width=self.width,
            height=self.height,
            debug=self.debug,
            parent=self.parent,
            auto_show=False,
        )

        self._setup_api()

    def _setup_api(self) -> None:
        """Set up API bindings."""
        if not self._webview:
            return

        @self._webview.bind_call("browser.new_tab")
        def api_new_tab(url: str = "", title: str = "New Tab") -> Dict[str, Any]:
            tab = self.new_tab(url, title)
            return {"tabId": tab["id"], "success": True}

        @self._webview.bind_call("browser.close_tab")
        def api_close_tab(tabId: str = "") -> Dict[str, Any]:
            self.close_tab(tabId or None)
            return {"success": True}

        @self._webview.bind_call("browser.activate_tab")
        def api_activate_tab(tabId: str) -> Dict[str, Any]:
            self.activate_tab(tabId)
            return {"success": True}

        @self._webview.bind_call("browser.navigate")
        def api_navigate(url: str, tabId: str = "") -> Dict[str, Any]:
            self.navigate(url, tabId or None)
            return {"success": True}

        @self._webview.bind_call("browser.go_back")
        def api_go_back() -> Dict[str, Any]:
            self.go_back()
            return {"success": True}

        @self._webview.bind_call("browser.go_forward")
        def api_go_forward() -> Dict[str, Any]:
            self.go_forward()
            return {"success": True}

        @self._webview.bind_call("browser.reload")
        def api_reload() -> Dict[str, Any]:
            self.reload()
            return {"success": True}

        @self._webview.bind_call("browser.get_tabs")
        def api_get_tabs() -> Dict[str, Any]:
            return {
                "tabs": self._tabs,
                "activeTabId": self._active_tab_id,
            }

        @self._webview.bind_call("browser.get_state")
        def api_get_state() -> Dict[str, Any]:
            return {
                "tabs": self._tabs,
                "activeTabId": self._active_tab_id,
            }

    def _sync_tabs_to_ui(self) -> None:
        """Sync tab state to the UI.

        Emits events to update the frontend when tabs change.
        """
        if not self._webview:
            return

        try:
            emitter = self._webview.create_emitter()
            emitter.emit(
                "browser:tabs_changed",
                {
                    "tabs": self._tabs,
                    "activeTabId": self._active_tab_id,
                },
            )
        except Exception as e:
            logger.debug(f"Failed to sync tabs to UI: {e}")

    def _get_browser_html(self) -> str:
        """Get the browser controller HTML.

        Returns:
            HTML string for the browser UI
        """
        import auroraview._core as core

        return core.get_browser_controller_html()
