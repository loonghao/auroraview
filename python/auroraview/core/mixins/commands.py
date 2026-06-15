# -*- coding: utf-8 -*-
# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView Commands Mixin - command registry and decorator."""

from __future__ import annotations

from typing import TYPE_CHECKING, Dict, List, Optional, Union

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

    def command(
        self,
        func_or_name=None,
        *,
        name: Optional[str] = None,
        cli: Union[bool, str, List[str]] = False,
        help: Optional[str] = None,
        args_help: Optional[Dict[str, str]] = None,
    ):
        """Decorator to register a command callable from JavaScript.

        This is a convenience shortcut for `webview.commands.register`.

        Args:
            func_or_name: Function to register or custom command name
            cli: CLI exposure switch + aliases (RFC 0018 §6.2). ``False``
                (default) keeps the command off the command line; ``True`` /
                ``"alias"`` / ``["a", "b"]`` opts in.
            help: Help text for ``-h``; falls back to the docstring first line.
            args_help: Per-parameter descriptions keyed by parameter name.

        Returns:
            Decorated function

        Example:
            >>> @webview.command
            >>> def greet(name: str) -> str:
            ...     return f"Hello, {name}!"

            >>> @webview.command("add_numbers")
            >>> def add(x: int, y: int) -> int:
            ...     return x + y

            >>> @webview.command(name="export", cli="exp", help="Export data")
            >>> def export(path: str) -> dict:
            ...     return {"written": path}

            >>> # In JavaScript:
            >>> # const msg = await auroraview.invoke("greet", {name: "World"});
            >>> # const sum = await auroraview.invoke("add_numbers", {x: 1, y: 2});
        """
        # When no CLI metadata is supplied, defer entirely to register()'s
        # positional handling so all legacy call styles keep working.
        if cli is False and help is None and args_help is None and name is None:
            return self.commands.register(func_or_name)
        return self.commands.register(
            func_or_name, name=name, cli=cli, help=help, args_help=args_help
        )
