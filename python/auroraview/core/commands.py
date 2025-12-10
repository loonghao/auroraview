# -*- coding: utf-8 -*-
"""Command System for simplified Python <-> JavaScript API definition.

This module provides a decorator-based command system inspired by Tauri's
#[command] macro, allowing easy definition of Python functions callable
from JavaScript.

Example:
    >>> from auroraview import WebView
    >>>
    >>> webview = WebView(title="Command Demo")
    >>>
    >>> # Define commands with decorator
    >>> @webview.command
    >>> def greet(name: str) -> str:
    ...     return f"Hello, {name}!"
    >>>
    >>> @webview.command("custom_name")
    >>> def my_function(x: int, y: int) -> int:
    ...     return x + y
    >>>
    >>> # In JavaScript:
    >>> # const result = await window.auroraview.invoke("greet", {name: "Alice"});
    >>> # console.log(result);  // "Hello, Alice!"
"""

from __future__ import annotations

import asyncio
import inspect
import logging
import traceback
from enum import Enum
from typing import (
    TYPE_CHECKING,
    Any,
    Callable,
    Dict,
    List,
    Optional,
    TypeVar,
    Union,
    overload,
)

if TYPE_CHECKING:
    from .webview import WebView

logger = logging.getLogger(__name__)

F = TypeVar("F", bound=Callable[..., Any])


class CommandErrorCode(Enum):
    """Error codes for command invocation failures.

    These codes help JavaScript identify the type of error and handle it appropriately.
    """

    # General errors
    UNKNOWN = "UNKNOWN"
    INTERNAL = "INTERNAL"

    # Invocation errors
    INVALID_DATA = "INVALID_DATA"
    MISSING_COMMAND = "MISSING_COMMAND"
    COMMAND_NOT_FOUND = "COMMAND_NOT_FOUND"

    # Argument errors
    INVALID_ARGUMENTS = "INVALID_ARGUMENTS"
    MISSING_ARGUMENT = "MISSING_ARGUMENT"
    TYPE_ERROR = "TYPE_ERROR"

    # Execution errors
    EXECUTION_ERROR = "EXECUTION_ERROR"
    TIMEOUT = "TIMEOUT"
    CANCELLED = "CANCELLED"

    # Permission errors
    PERMISSION_DENIED = "PERMISSION_DENIED"


class CommandError(Exception):
    """Exception raised when a command fails.

    This exception provides structured error information that can be
    serialized and sent to JavaScript.

    Attributes:
        code: Error code from CommandErrorCode enum
        message: Human-readable error message
        details: Optional additional error details
    """

    def __init__(
        self,
        code: CommandErrorCode,
        message: str,
        details: Optional[Dict[str, Any]] = None,
    ):
        """Initialize CommandError.

        Args:
            code: Error code
            message: Error message
            details: Optional additional details
        """
        super().__init__(message)
        self.code = code
        self.message = message
        self.details = details or {}

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization.

        Returns:
            Dictionary with error information
        """
        result = {
            "code": self.code.value,
            "message": self.message,
        }
        if self.details:
            result["details"] = self.details
        return result

    def __repr__(self) -> str:
        return f"CommandError({self.code.value}: {self.message})"


class CommandRegistry:
    """Registry for managing commands callable from JavaScript.

    This class manages the registration and invocation of Python commands
    that can be called from JavaScript via the IPC bridge.

    Attributes:
        _commands: Dictionary mapping command names to handlers
        _webview: Associated WebView instance
    """

    def __init__(self, webview: Optional[WebView] = None):
        """Initialize the CommandRegistry.

        Args:
            webview: Associated WebView instance
        """
        self._commands: Dict[str, Callable[..., Any]] = {}
        self._webview: Optional[WebView] = webview

    def _attach_webview(self, webview: WebView) -> None:
        """Attach a WebView instance and register IPC handler.

        Args:
            webview: WebView instance to attach
        """
        self._webview = webview
        # Register the invoke handler
        webview.register_callback("__invoke__", self._handle_invoke)

    def _handle_invoke(self, data: Dict[str, Any]) -> Any:
        """Handle command invocation from JavaScript.

        Args:
            data: Invocation data with 'id', 'command' and 'args' fields

        Returns:
            Response dict with 'id' and either 'result' or 'error'
        """
        invoke_id = data.get("id", "") if isinstance(data, dict) else ""

        def make_error(error: CommandError) -> Dict[str, Any]:
            """Create error response with invoke ID."""
            return {"id": invoke_id, "error": error.to_dict()}

        def make_result(result: Any) -> Dict[str, Any]:
            """Create success response with invoke ID."""
            return {"id": invoke_id, "result": result}

        if not isinstance(data, dict):
            return make_error(
                CommandError(
                    CommandErrorCode.INVALID_DATA,
                    "Invalid invoke data: expected object",
                )
            )

        command_name = data.get("command")
        args = data.get("args", {})

        if not command_name:
            return make_error(
                CommandError(
                    CommandErrorCode.MISSING_COMMAND,
                    "Missing command name in invoke request",
                )
            )

        if command_name not in self._commands:
            return make_error(
                CommandError(
                    CommandErrorCode.COMMAND_NOT_FOUND,
                    f"Command not found: {command_name}",
                    {"command": command_name, "available": list(self._commands.keys())},
                )
            )

        try:
            handler = self._commands[command_name]

            # Handle async functions
            if asyncio.iscoroutinefunction(handler):
                try:
                    asyncio.get_running_loop()
                    asyncio.ensure_future(handler(**args))
                    return {"id": invoke_id, "pending": True}
                except RuntimeError:
                    result = asyncio.run(handler(**args))
                    return make_result(result)
            else:
                result = handler(**args)
                return make_result(result)

        except CommandError as e:
            # Re-raise CommandError as-is
            return make_error(e)

        except TypeError as e:
            # Argument type/count mismatch
            sig = inspect.signature(handler)
            return make_error(
                CommandError(
                    CommandErrorCode.INVALID_ARGUMENTS,
                    f"Invalid arguments for '{command_name}': {e}",
                    {
                        "command": command_name,
                        "expected": list(sig.parameters.keys()),
                        "received": list(args.keys()) if isinstance(args, dict) else [],
                    },
                )
            )

        except Exception as e:
            # Unexpected error during execution
            logger.error(f"Command '{command_name}' error: {e}")
            logger.debug(traceback.format_exc())
            return make_error(
                CommandError(
                    CommandErrorCode.EXECUTION_ERROR,
                    f"Command execution failed: {e}",
                    {"command": command_name, "exception": type(e).__name__},
                )
            )

    @overload
    def register(self, func: F) -> F: ...

    @overload
    def register(self, name: str) -> Callable[[F], F]: ...

    def register(self, func_or_name: Union[F, str, None] = None) -> Union[F, Callable[[F], F]]:
        """Register a command (decorator).

        Can be used with or without arguments:

            @commands.register
            def my_command(): ...

            @commands.register("custom_name")
            def my_command(): ...

        Args:
            func_or_name: Function to register or custom command name

        Returns:
            Decorated function or decorator
        """

        def decorator(func: F, name: Optional[str] = None) -> F:
            cmd_name = name or func.__name__
            self._commands[cmd_name] = func

            # Emit registration to JS if webview is attached
            if self._webview:
                self._webview.emit(
                    "__command_registered__",
                    {"name": cmd_name, "params": list(inspect.signature(func).parameters.keys())},
                )

            logger.debug(f"Registered command: {cmd_name}")
            return func

        # Handle different call patterns
        if func_or_name is None:
            # @commands.register()
            return lambda f: decorator(f)
        elif callable(func_or_name):
            # @commands.register
            return decorator(func_or_name)
        else:
            # @commands.register("name")
            return lambda f: decorator(f, func_or_name)

    def unregister(self, name: str) -> bool:
        """Unregister a command.

        Args:
            name: Command name to unregister

        Returns:
            True if command was removed, False if not found
        """
        if name in self._commands:
            del self._commands[name]
            logger.debug(f"Unregistered command: {name}")
            return True
        return False

    def list_commands(self) -> List[str]:
        """List all registered command names.

        Returns:
            List of command names
        """
        return list(self._commands.keys())

    def has_command(self, name: str) -> bool:
        """Check if a command is registered.

        Args:
            name: Command name to check

        Returns:
            True if command exists
        """
        return name in self._commands

    def invoke(self, name: str, **kwargs: Any) -> Any:
        """Invoke a command directly from Python.

        Args:
            name: Command name
            **kwargs: Command arguments

        Returns:
            Command result

        Raises:
            KeyError: If command not found
        """
        if name not in self._commands:
            raise KeyError(f"Unknown command: {name}")
        return self._commands[name](**kwargs)

    def __len__(self) -> int:
        """Return number of registered commands."""
        return len(self._commands)

    def __contains__(self, name: str) -> bool:
        """Check if command is registered."""
        return name in self._commands

    def __repr__(self) -> str:
        """String representation."""
        return f"CommandRegistry({list(self._commands.keys())})"
