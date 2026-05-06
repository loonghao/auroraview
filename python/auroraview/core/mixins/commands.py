# -*- coding: utf-8 -*-
# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Commands Mixin - command registry and decorator."""

from __future__ import annotations

from typing import TYPE_CHECKING, Optional

if TYPE_CHECKING:
    from ..commands import CommandRegistry


class WebViewCommandsMixin:
    """Mixin for command registry (Python ↔ JavaScript RPC)."""

    # Instance variables to be set by WebView.__init__
    _commands: Optional["CommandRegistry"]

    @property
    def commands(self) -> "CommandRegistry":
        """Get the command registry for Python ↔ JavaScript RPC.

        Returns:
            CommandRegistry instance

        Example:
            >>> @webview.commands.register
            >>> def greet(name: str) -> str:
            ...     return f"Hello, {name}!"
        """
        if self._commands is None:
            from ..commands import CommandRegistry

            self._commands = CommandRegistry(self)
        return self._commands

    def command(self, func_or_name=None):
        """Decorator to register a command callable from JavaScript.

        This is a convenience shortcut for `webview.commands.register`.

        Args:
            func_or_name: Function to register or custom command name

        Returns:
            Decorated function

        Example:
            >>> @webview.command
            >>> def greet(name: str) -> str:
            ...     return f"Hello, {name}!"

            >>> @webview.command("add_numbers")
            >>> def add(x: int, y: int) -> int:
            ...     return x + y

            >>> # In JavaScript:
            >>> # const msg = await auroraview.invoke("greet", {name: "World"});
            >>> # const sum = await auroraview.invoke("add_numbers", {x: 1, y: 2});
        """
        return self.commands.register(func_or_name)
