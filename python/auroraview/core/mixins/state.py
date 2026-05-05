# -*- coding: utf-8 -*-
# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView State Mixin - lazy-initialized shared state."""

from __future__ import annotations

from typing import TYPE_CHECKING, Optional

if TYPE_CHECKING:
    from ..state import State


class WebViewStateMixin:
    """Mixin for lazy-initialized shared state (Python ↔ JavaScript sync)."""

    # Instance variables to be set by WebView.__init__
    _state: Optional["State"]

    @property
    def state(self) -> "State":
        """Get the shared state container for Python ↔ JavaScript sync.

        Returns:
            State container with dict-like interface

        Example:
            >>> webview.state["user"] = {"name": "Alice"}
            >>> webview.state["theme"] = "dark"
            >>>
            >>> @webview.state.on_change
            >>> def handle_change(key, value, source):
            ...     print(f"{key} = {value} from {source}")
        """
        if self._state is None:
            from ..state import State

            self._state = State(self)
        return self._state
