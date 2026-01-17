# -*- coding: utf-8 -*-
"""Tab container for managing multiple tabs with WebView windows.

This module provides a foundation for building tabbed browsers,
multi-panel DCC tools, and other multi-webview applications.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from threading import Lock
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional
from uuid import uuid4

if TYPE_CHECKING:
    from auroraview.core.webview import WebView

from auroraview.core.window_manager import get_window_manager

logger = logging.getLogger(__name__)


@dataclass
class TabState:
    """State for a single tab.

    Attributes:
        id: Unique tab identifier
        title: Tab title (displayed in tab bar)
        url: Current URL
        favicon: Favicon URL or data URI
        is_loading: Whether the page is loading
        can_go_back: Whether back navigation is available
        can_go_forward: Whether forward navigation is available
        webview_id: Reference to WindowManager (None if not loaded)
        metadata: Custom metadata storage
    """

    id: str
    title: str = "New Tab"
    url: str = ""
    favicon: str = ""
    is_loading: bool = False
    can_go_back: bool = False
    can_go_forward: bool = False
    webview_id: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            "id": self.id,
            "title": self.title,
            "url": self.url,
            "favicon": self.favicon,
            "isLoading": self.is_loading,
            "canGoBack": self.can_go_back,
            "canGoForward": self.can_go_forward,
            "metadata": self.metadata,
        }


class TabContainer:
    """Container for managing multiple tabs with WebView windows.

    This provides a foundation for building tabbed browsers, multi-panel
    DCC tools, and other multi-webview applications.

    Features:
    - Tab state management (create, close, activate)
    - Lazy WebView loading
    - Navigation controls
    - Event callbacks for UI updates

    Example:
        >>> container = TabContainer(
        ...     on_tabs_update=lambda tabs: print(f"Tabs: {len(tabs)}"),
        ...     default_url="https://example.com"
        ... )
        >>> tab = container.create_tab("https://github.com")
        >>> container.navigate("https://google.com")
        >>> container.close_tab(tab.id)
    """

    def __init__(
        self,
        on_tab_change: Optional[Callable[[TabState], None]] = None,
        on_tabs_update: Optional[Callable[[List[TabState]], None]] = None,
        default_url: str = "",
        webview_factory: Optional[Callable[..., "WebView"]] = None,
        webview_options: Optional[Dict[str, Any]] = None,
    ):
        """Initialize TabContainer.

        Args:
            on_tab_change: Callback when active tab changes
            on_tabs_update: Callback when tab list changes
            default_url: Default URL for new tabs
            webview_factory: Custom factory for creating WebViews
            webview_options: Options passed to WebView creation
        """
        self._tabs: Dict[str, TabState] = {}
        self._tab_order: List[str] = []  # Maintain insertion order
        self._active_tab_id: Optional[str] = None
        self._on_tab_change = on_tab_change
        self._on_tabs_update = on_tabs_update
        self._default_url = default_url
        self._webview_factory = webview_factory
        self._webview_options = webview_options or {}
        self._lock = Lock()
        self._wm = get_window_manager()

    def create_tab(
        self,
        url: str = "",
        title: str = "New Tab",
        activate: bool = True,
        load_immediately: bool = True,
    ) -> TabState:
        """Create a new tab.

        Args:
            url: Initial URL (uses default_url if empty)
            title: Initial tab title
            activate: Whether to activate the new tab
            load_immediately: Whether to create WebView immediately

        Returns:
            The created TabState
        """
        tab_id = f"tab_{uuid4().hex[:8]}"
        tab = TabState(
            id=tab_id,
            url=url or self._default_url,
            title=title,
        )

        with self._lock:
            self._tabs[tab_id] = tab
            self._tab_order.append(tab_id)

            if activate or self._active_tab_id is None:
                self._active_tab_id = tab_id

        logger.info(f"Tab created: {tab_id} (url={tab.url})")

        if load_immediately and tab.url:
            self._load_tab_webview(tab)

        self._notify_tabs_update()

        if activate and self._on_tab_change:
            self._on_tab_change(tab)

        return tab

    def close_tab(self, tab_id: str) -> Optional[str]:
        """Close a tab.

        Args:
            tab_id: ID of the tab to close

        Returns:
            ID of the new active tab, or None if no tabs remain
        """
        with self._lock:
            if tab_id not in self._tabs:
                return self._active_tab_id

            tab = self._tabs.pop(tab_id)
            self._tab_order.remove(tab_id)

            # Close WebView if exists
            if tab.webview_id:
                webview = self._wm.get(tab.webview_id)
                if webview:
                    try:
                        webview.close()
                    except Exception as e:
                        logger.warning(f"Error closing tab WebView: {e}")

            # Select new active tab
            if self._active_tab_id == tab_id:
                if self._tab_order:
                    # Try to select adjacent tab
                    self._active_tab_id = self._tab_order[-1]
                else:
                    self._active_tab_id = None

        logger.info(f"Tab closed: {tab_id}")
        self._notify_tabs_update()

        if self._active_tab_id and self._on_tab_change:
            new_tab = self._tabs.get(self._active_tab_id)
            if new_tab:
                self._on_tab_change(new_tab)

        return self._active_tab_id

    def activate_tab(self, tab_id: str) -> bool:
        """Activate a tab.

        Args:
            tab_id: ID of the tab to activate

        Returns:
            True if tab was found and activated
        """
        with self._lock:
            if tab_id not in self._tabs:
                return False

            if self._active_tab_id == tab_id:
                return True

            old_tab = self._tabs.get(self._active_tab_id)
            new_tab = self._tabs[tab_id]

            # Hide old tab's webview
            if old_tab and old_tab.webview_id:
                webview = self._wm.get(old_tab.webview_id)
                if webview:
                    try:
                        webview.hide()
                    except Exception as e:
                        logger.warning(f"Error hiding tab WebView: {e}")

            # Show/load new tab's webview
            if new_tab.webview_id:
                webview = self._wm.get(new_tab.webview_id)
                if webview:
                    try:
                        webview.show(wait=False)
                    except Exception as e:
                        logger.warning(f"Error showing tab WebView: {e}")
            elif new_tab.url:
                self._load_tab_webview(new_tab)

            self._active_tab_id = tab_id

        logger.info(f"Tab activated: {tab_id}")

        if self._on_tab_change:
            self._on_tab_change(new_tab)

        self._notify_tabs_update()
        return True

    def navigate(self, url: str, tab_id: Optional[str] = None) -> bool:
        """Navigate a tab to a URL.

        Args:
            url: URL to navigate to
            tab_id: Tab ID (uses active tab if None)

        Returns:
            True if navigation was initiated
        """
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return False

        with self._lock:
            tab = self._tabs.get(tab_id)
            if not tab:
                return False

            tab.url = url
            tab.is_loading = True

            if tab.webview_id:
                webview = self._wm.get(tab.webview_id)
                if webview:
                    webview.load_url(url)
            else:
                self._load_tab_webview(tab)

        logger.info(f"Tab navigating: {tab_id} -> {url}")
        self._notify_tabs_update()
        return True

    def go_back(self, tab_id: Optional[str] = None) -> bool:
        """Go back in the specified tab."""
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return False

        tab = self._tabs.get(tab_id)
        if not tab or not tab.webview_id:
            return False

        webview = self._wm.get(tab.webview_id)
        if webview and tab.can_go_back:
            webview.go_back()
            return True
        return False

    def go_forward(self, tab_id: Optional[str] = None) -> bool:
        """Go forward in the specified tab."""
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return False

        tab = self._tabs.get(tab_id)
        if not tab or not tab.webview_id:
            return False

        webview = self._wm.get(tab.webview_id)
        if webview and tab.can_go_forward:
            webview.go_forward()
            return True
        return False

    def reload(self, tab_id: Optional[str] = None) -> bool:
        """Reload the specified tab."""
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return False

        tab = self._tabs.get(tab_id)
        if not tab or not tab.webview_id:
            return False

        webview = self._wm.get(tab.webview_id)
        if webview:
            webview.reload()
            tab.is_loading = True
            self._notify_tabs_update()
            return True
        return False

    def update_tab(self, tab_id: str, **kwargs: Any) -> bool:
        """Update tab properties.

        Args:
            tab_id: Tab ID
            **kwargs: Properties to update (title, favicon, metadata, etc.)

        Returns:
            True if tab was found and updated
        """
        with self._lock:
            tab = self._tabs.get(tab_id)
            if not tab:
                return False

            for key, value in kwargs.items():
                if hasattr(tab, key):
                    setattr(tab, key, value)

        self._notify_tabs_update()
        return True

    def get_tab(self, tab_id: str) -> Optional[TabState]:
        """Get a tab by ID."""
        return self._tabs.get(tab_id)

    def get_active_tab(self) -> Optional[TabState]:
        """Get the active tab."""
        if self._active_tab_id:
            return self._tabs.get(self._active_tab_id)
        return None

    def get_active_tab_id(self) -> Optional[str]:
        """Get the active tab ID."""
        return self._active_tab_id

    def get_all_tabs(self) -> List[TabState]:
        """Get all tabs in order."""
        return [self._tabs[tid] for tid in self._tab_order if tid in self._tabs]

    def get_tab_count(self) -> int:
        """Get the number of tabs."""
        return len(self._tabs)

    def get_webview(self, tab_id: Optional[str] = None) -> Optional["WebView"]:
        """Get the WebView for a tab.

        Args:
            tab_id: Tab ID (uses active tab if None)

        Returns:
            The WebView instance, or None
        """
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return None

        tab = self._tabs.get(tab_id)
        if not tab or not tab.webview_id:
            return None

        return self._wm.get(tab.webview_id)

    def _load_tab_webview(self, tab: TabState) -> None:
        """Create and load a WebView for a tab."""
        from auroraview import create_webview

        factory = self._webview_factory or create_webview
        webview = factory(url=tab.url, auto_show=False, **self._webview_options)

        tab.webview_id = webview.window_id
        self._setup_webview_events(tab, webview)

        # Show if this is the active tab
        if self._active_tab_id == tab.id:
            webview.show(wait=False)

        logger.info(f"Tab WebView loaded: {tab.id} -> {tab.webview_id}")

    def _setup_webview_events(self, tab: TabState, webview: "WebView") -> None:
        """Set up event handlers for a tab's WebView."""

        @webview.on("page:load_start")
        def on_load_start(data: Any) -> None:
            tab.is_loading = True
            self._notify_tabs_update()

        @webview.on("page:load_finish")
        def on_load_finish(data: Any) -> None:
            tab.is_loading = False
            if isinstance(data, dict):
                if data.get("url"):
                    tab.url = data["url"]
                if data.get("title"):
                    tab.title = data["title"]
                tab.can_go_back = data.get("canGoBack", False)
                tab.can_go_forward = data.get("canGoForward", False)
            self._notify_tabs_update()

        @webview.on("page:title_changed")
        def on_title_changed(data: Any) -> None:
            if isinstance(data, dict) and data.get("title"):
                tab.title = data["title"]
                self._notify_tabs_update()

        @webview.on("page:favicon_changed")
        def on_favicon_changed(data: Any) -> None:
            if isinstance(data, dict) and data.get("favicon"):
                tab.favicon = data["favicon"]
                self._notify_tabs_update()

        @webview.on("navigation:state_changed")
        def on_nav_state(data: Any) -> None:
            if isinstance(data, dict):
                tab.can_go_back = data.get("canGoBack", tab.can_go_back)
                tab.can_go_forward = data.get("canGoForward", tab.can_go_forward)
                self._notify_tabs_update()

        @webview.on("closing")
        def on_closing(data: Any) -> None:
            self.close_tab(tab.id)

    def _notify_tabs_update(self) -> None:
        """Notify tabs update callback."""
        if self._on_tabs_update:
            try:
                self._on_tabs_update(self.get_all_tabs())
            except Exception as e:
                logger.warning(f"Tabs update callback error: {e}")

    def close_all(self) -> None:
        """Close all tabs."""
        for tab_id in list(self._tab_order):
            self.close_tab(tab_id)
