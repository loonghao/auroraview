"""High-level AuroraView base class.

This module provides the :class:`AuroraView` Python base class that wraps a
``WebView``/``QtWebView`` instance and offers:

* A stable facade for ``bind_call`` / ``bind_api`` / ``emit``.
* An internal keep-alive registry so DCC hosts (Maya, Nuke, etc.) do not need
  to store global references like ``__main__.my_dialog = dlg``.

The API shape follows the design in ``.augment/rules/architecture.md`` but is
implemented in a conservative way on top of the existing WebView classes.
"""

from __future__ import annotations

import logging
from typing import Any, Callable, ClassVar, Optional, Set

from .webview import WebView

logger = logging.getLogger(__name__)


class AuroraView:
    """Base class for building tools on top of AuroraView.

    The main goals are:

    * Provide a stable Python facade with ``bind_call`` / ``bind_api`` /
      ``emit`` that forwards to the underlying WebView/QtWebView.
    * Keep the underlying UI objects alive via a class-level registry so that
      embedders (Maya, Nuke, etc.) do not have to manage globals manually.

    In simple cases you can let :class:`AuroraView` create the underlying
    :class:`WebView` for you::

        tool = AuroraView(url="http://localhost:3000", api=my_api)
        tool.show()

    In Qt/DCC scenarios you can reuse an existing ``QtWebView`` instance by
    passing it via ``_view`` and specifying a ``_keep_alive_root`` (typically
    the top-level Qt dialog)::

        self.webview = QtWebView(self)
        self.tool = AuroraView(parent=self, api=self, _view=self.webview,
                               _keep_alive_root=self)
    """

    _instances: ClassVar[Set["AuroraView"]] = set()

    def __init__(
        self,
        *,
        parent: Any | None = None,
        url: str | None = None,
        html: str | None = None,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        fullscreen: bool = False,
        debug: bool = False,
        api: Any | None = None,
        _view: Any | None = None,
        _keep_alive_root: Any | None = None,
        _auto_show: bool = False,
        **kwargs: Any,
    ) -> None:
        self._parent = parent
        self._title = title
        self._width = width
        self._height = height
        self._fullscreen = fullscreen
        self._debug = debug
        self._url = url
        self._html = html
        self._api = api if api is not None else self

        if _view is not None:
            # Reuse an existing backend view (typically a QtWebView widget).
            self._view = _view
        else:
            # Fallback to native WebView backend.
            self._view = WebView(
                title=title,
                width=width,
                height=height,
                url=url,
                html=html,
                debug=debug,
                parent=parent,
                **kwargs,
            )

        # Bind API object if provided and the backend supports bind_api.
        bind_api = getattr(self._view, "bind_api", None)
        if self._api is not None and callable(bind_api):
            bind_api(self._api)

        # The root that should be kept alive (typically the top-level window).
        self._keep_alive_root = _keep_alive_root or self._view

        # Register for automatic keep-alive to prevent premature GC.
        AuroraView._instances.add(self)
        logger.debug("AuroraView instance registered for keep-alive: %r", self)

        if _auto_show:
            self.show()

    # ------------------------------------------------------------------
    # Lifecycle hooks (subclasses may override)
    # ------------------------------------------------------------------
    def on_show(self) -> None:  # pragma: no cover - default hook
        """Called after :meth:`show` is invoked."""

    def on_hide(self) -> None:  # pragma: no cover - default hook
        """Placeholder for future hide support."""

    def on_close(self) -> None:  # pragma: no cover - default hook
        """Called when :meth:`close` is executed."""

    def on_ready(self) -> None:  # pragma: no cover - default hook
        """Placeholder for JS bridge ready hook."""

    # ------------------------------------------------------------------
    # Delegation helpers
    # ------------------------------------------------------------------
    @property
    def view(self) -> Any:
        """Return the underlying WebView/QtWebView instance."""

        return self._view

    def emit(self, event_name: str, payload: Any) -> None:
        """Emit an event to JavaScript via the underlying view."""

        emit = getattr(self._view, "emit", None)
        if callable(emit):
            emit(event_name, payload)

    def bind_call(
        self,
        method: str,
        func: Optional[Callable[..., Any]] = None,
    ):
        """Bind a Python callable as an ``auroraview.call`` target."""

        bind_call = getattr(self._view, "bind_call", None)
        if not callable(bind_call):
            raise RuntimeError("Underlying view does not support bind_call")
        return bind_call(method, func)

    def bind_api(self, api: Any, namespace: str = "api") -> None:
        """Bind all public methods of an object under a namespace."""

        bind_api = getattr(self._view, "bind_api", None)
        if not callable(bind_api):
            raise RuntimeError("Underlying view does not support bind_api")
        bind_api(api, namespace=namespace)

    def show(self, *args: Any, **kwargs: Any) -> None:
        """Show the underlying view/window."""

        show = getattr(self._view, "show", None)
        if callable(show):
            show(*args, **kwargs)
        self.on_show()

    def close(self) -> None:
        """Close the tool and unregister it from the keep-alive registry."""

        if self in AuroraView._instances:
            AuroraView._instances.remove(self)
            logger.debug("AuroraView instance unregistered: %r", self)

        # Close keep-alive root if it has a close() method
        root = self._keep_alive_root
        try:
            close_root = getattr(root, "close", None)
            if callable(close_root):
                close_root()
        except Exception:  # pragma: no cover - defensive
            logger.exception("Error while closing keep_alive_root")

        # If root differs from the underlying view, attempt to close view as well
        if root is not self._view:
            try:
                close_view = getattr(self._view, "close", None)
                if callable(close_view):
                    close_view()
            except Exception:  # pragma: no cover - defensive
                logger.exception("Error while closing underlying view")

        self.on_close()
