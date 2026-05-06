# -*- coding: utf-8 -*-
# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Bridge Mixin - Bridge integration methods."""

from __future__ import annotations

import logging
from typing import Any, Dict, Optional, TYPE_CHECKING

if TYPE_CHECKING:
    from ..bridge import Bridge


logger = logging.getLogger(__name__)


class WebViewBridgeMixin:
    """Mixin for Bridge integration (bidirectional communication)."""

    # Instance variables to be set by WebView.__init__
    _bridge: Optional["Bridge"]

    def _setup_bridge_integration(self) -> None:
        """Setup bidirectional communication between Bridge and WebView.

        This method is called automatically when a Bridge is associated with the WebView.
        It sets up:
        1. Bridge → WebView: Forward bridge events to WebView UI
        2. WebView → Bridge: Register handler to send commands to bridge clients
        """
        if not self._bridge:
            return

        logger.info("Setting up Bridge ↔ WebView integration")

        # Bridge → WebView: Forward events
        def bridge_callback(action: str, data: Dict, result: Any):
            """Forward bridge events to WebView UI."""
            logger.debug(f"Bridge event: {action}")
            # Emit event to JavaScript with 'bridge:' prefix
            self.emit(f"bridge:{action}", {"action": action, "data": data, "result": result})

        self._bridge.set_webview_callback(bridge_callback)

        # WebView → Bridge: Register command sender
        @self.on("send_to_bridge")
        def handle_send_to_bridge(data):
            """Send command from WebView to Bridge clients."""
            command = data.get("command")
            params = data.get("params", {})
            logger.debug(f"WebView → Bridge: {command}")
            if self._bridge:
                self._bridge.execute_command(command, params)
            return {"status": "sent"}

        logger.info("Bridge <-> WebView integration complete")

    @property
    def bridge(self) -> Optional["Bridge"]:  # type: ignore
        """Get the associated Bridge instance.

        Returns:
            Bridge instance or None if no bridge is associated

        Example:
            >>> webview = WebView.create("Tool", bridge=True)
            >>> print(webview.bridge)  # Bridge(ws://localhost:9001, ...)
            >>>
            >>> # Register handlers on the bridge
            >>> @webview.bridge.on('custom_event')
            >>> async def handle_custom(data, client):
            ...     return {"status": "ok"}
        """
        return self._bridge

    def send_to_bridge(self, command: str, params: Dict[str, Any] = None) -> None:
        """Send command to Bridge clients (convenience method).

        Args:
            command: Command name
            params: Command parameters

        Example:
            >>> webview.send_to_bridge('create_layer', {'name': 'New Layer'})
        """
        if not self._bridge:
            logger.warning("No bridge associated with this WebView")
            return

        self._bridge.execute_command(command, params)
