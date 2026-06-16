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
from dataclasses import dataclass, field
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

# RFC 0018 §12.4: reserved CLI verbs/flags a command alias must never collide
# with. `run`/`list` are the trigger verbs; `help`/`version` back the -h/-V
# flags. Kept lowercase for case-sensitive comparison against alias strings.
_RESERVED_CLI_VERBS = frozenset({"run", "list", "help", "version"})


def _is_headless_cli_mode() -> bool:
    """Whether we're in a packed headless CLI mode (RFC 0018 §7/§13.3).

    In CLI invoke and pack-time dump modes there is no front-end and stdout is
    reserved for the command result / metadata table, so internal events like
    ``__command_registered__`` must not be emitted (it would go straight to
    stdout via the packed-mode path and corrupt the output). Imported lazily to
    avoid a circular import with :mod:`auroraview.core.packed`.
    """
    try:
        from .packed import is_cli_dump_mode, is_cli_invoke_mode
    except Exception:  # noqa: BLE001 - never let a probe break registration
        return False
    return is_cli_invoke_mode() or is_cli_dump_mode()


def _normalize_cli(cli: "Union[bool, str, List[str]]") -> "tuple[bool, List[str]]":
    """Normalize a decorator ``cli`` value into ``(enabled, aliases)``.

    RFC 0018 §6.2 collapses the CLI on/off switch and the alias list into one
    parameter:

    =================  =========================================
    ``cli`` value      meaning
    =================  =========================================
    ``False``          not exposed to the CLI (default)
    ``True``           exposed, no alias
    ``"exi"``          exposed, alias ``exi``
    ``["exi","edi"]``  exposed, multiple aliases
    =================  =========================================

    Returns:
        ``(enabled, aliases)`` where ``aliases`` is always a list (possibly
        empty) of stripped, non-empty alias strings.

    Raises:
        ValueError: If an alias is empty/blank or not a string.
    """
    if cli is False:
        return False, []
    if cli is True:
        return True, []
    if isinstance(cli, str):
        alias = cli.strip()
        if not alias:
            raise ValueError("cli alias string must not be empty")
        return True, [alias]
    if isinstance(cli, (list, tuple)):
        aliases: List[str] = []
        for item in cli:
            if not isinstance(item, str):
                raise ValueError(f"cli alias must be a string, got {type(item).__name__}")
            alias = item.strip()
            if not alias:
                raise ValueError("cli alias string must not be empty")
            aliases.append(alias)
        return True, aliases
    raise ValueError(f"cli must be bool, str, or list[str], got {type(cli).__name__}")


def _resolve_help(help_text: Optional[str], func: Callable[..., Any]) -> str:
    """Resolve a command's help text, falling back to the docstring first line.

    RFC 0018 §6.2: when ``help`` is omitted, use the first non-empty line of
    the function's docstring. Returns an empty string if neither is available.
    """
    if help_text:
        return help_text
    doc = inspect.getdoc(func)
    if doc:
        for line in doc.splitlines():
            stripped = line.strip()
            if stripped:
                return stripped
    return ""


@dataclass
class CliCommandMeta:
    """CLI metadata for a single command (RFC 0018 §13.2).

    Captured at registration time and serialized at pack time into the overlay
    so the runtime ``-h``/``list`` path can render without starting Python.

    Attributes:
        name: Canonical command name (the registered name).
        aliases: CLI aliases from ``cli="x"`` / ``cli=["x","y"]`` (empty for
            ``cli=True``).
        help: Help text; falls back to the docstring first line.
        args_help: Per-parameter descriptions keyed by parameter name.
    """

    name: str
    aliases: List[str] = field(default_factory=list)
    help: str = ""
    args_help: Dict[str, str] = field(default_factory=dict)

    def params(self, func: Callable[..., Any]) -> List[Dict[str, Any]]:
        """Introspect ``func`` to build the parameter metadata list (§13.2).

        Each entry has ``name``, ``type`` (annotation name, ``"any"`` if
        unannotated), ``required`` (no default), ``default``, and ``help``
        (from ``args_help``). ``*args``/``**kwargs`` and ``self`` are skipped.
        """
        result: List[Dict[str, Any]] = []
        try:
            sig = inspect.signature(func)
        except (ValueError, TypeError):
            return result

        for pname, param in sig.parameters.items():
            if pname == "self":
                continue
            if param.kind in (
                inspect.Parameter.VAR_POSITIONAL,
                inspect.Parameter.VAR_KEYWORD,
            ):
                continue

            annotation = param.annotation
            if annotation is inspect.Parameter.empty:
                type_name = "any"
            elif isinstance(annotation, type):
                type_name = annotation.__name__
            else:
                type_name = str(annotation)

            has_default = param.default is not inspect.Parameter.empty
            result.append(
                {
                    "name": pname,
                    "type": type_name,
                    "required": not has_default,
                    "default": None if not has_default else param.default,
                    "help": self.args_help.get(pname, ""),
                }
            )
        return result

    def to_dict(self, func: Callable[..., Any]) -> Dict[str, Any]:
        """Serialize to the §13.2 overlay structure, introspecting ``func``."""
        return {
            "name": self.name,
            "aliases": list(self.aliases),
            "help": self.help,
            "params": self.params(func),
        }


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
        # RFC 0018: CLI metadata keyed by command name. Only populated for
        # commands that opt in via `cli != False`. Separate from `_commands`
        # so non-CLI commands stay untouched and the JS path is unaffected.
        self._cli_meta: Dict[str, "CliCommandMeta"] = {}

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

    def register(
        self,
        func_or_name: Union[F, str, None] = None,
        *,
        name: Optional[str] = None,
        cli: Union[bool, str, List[str]] = False,
        help: Optional[str] = None,
        args_help: Optional[Dict[str, str]] = None,
    ) -> Union[F, Callable[[F], F]]:
        """Register a command (decorator).

        Can be used with or without arguments:

            @commands.register
            def my_command(): ...

            @commands.register("custom_name")
            def my_command(): ...

        CLI exposure (RFC 0018 §6.2) is opt-in via ``cli``:

            @commands.register("export", cli=True)            # exposed, no alias
            @commands.register("export", cli="exp")           # exposed + alias
            @commands.register("export", cli=["exp", "e"])    # multiple aliases

        Args:
            func_or_name: Function to register or custom command name
            cli: CLI exposure switch + aliases. ``False`` (default) keeps the
                command off the command line; ``True``/str/list opts in.
            help: Help text for ``-h``; falls back to the docstring first line.
            args_help: Per-parameter descriptions keyed by parameter name.

        Returns:
            Decorated function or decorator
        """
        enabled, aliases = _normalize_cli(cli)

        def decorator(func: F, explicit_name: Optional[str] = None) -> F:
            cmd_name = explicit_name or func.__name__
            self._commands[cmd_name] = func

            # RFC 0018: record CLI metadata only when opted in.
            if enabled:
                self._register_cli_meta(cmd_name, func, aliases, help, args_help)

            # Emit registration to JS if webview is attached. Skipped in the
            # headless CLI modes (RFC 0018 §7/§13.3): there is no front-end to
            # consume the event, and stdout is reserved for the command result
            # (invoke) or the metadata table (dump) — emitting would corrupt it.
            if self._webview and not _is_headless_cli_mode():
                self._webview.emit(
                    "__command_registered__",
                    {"name": cmd_name, "params": list(inspect.signature(func).parameters.keys())},
                )

            logger.debug(f"Registered command: {cmd_name}")
            return func

        # An explicit `name=` keyword wins over a positional name.
        if name is not None:
            if callable(func_or_name):
                return decorator(func_or_name, name)
            return lambda f: decorator(f, name)

        # Handle different call patterns
        if func_or_name is None:
            # @commands.register()  /  @commands.register(cli=...)
            return lambda f: decorator(f)
        elif callable(func_or_name):
            # @commands.register
            return decorator(func_or_name)
        else:
            # @commands.register("name")
            return lambda f: decorator(f, func_or_name)

    def _register_cli_meta(
        self,
        cmd_name: str,
        func: Callable[..., Any],
        aliases: List[str],
        help_text: Optional[str],
        args_help: Optional[Dict[str, str]],
    ) -> None:
        """Record CLI metadata for a command after conflict checking (§12.4).

        Raises:
            ValueError: On any alias conflict (reserved verb, canonical name,
                or another command's alias).
        """
        self._check_alias_conflicts(cmd_name, aliases)
        self._cli_meta[cmd_name] = CliCommandMeta(
            name=cmd_name,
            aliases=list(aliases),
            help=_resolve_help(help_text, func),
            args_help=dict(args_help or {}),
        )

    def _check_alias_conflicts(self, cmd_name: str, aliases: List[str]) -> None:
        """Fail-fast alias conflict detection (RFC 0018 §12.4).

        An alias must not collide with a reserved verb, any canonical command
        name, or any alias already claimed by another command.

        Args:
            cmd_name: The command the aliases belong to (its own canonical
                name is exempt so re-registration is allowed).
            aliases: Aliases being registered.

        Raises:
            ValueError: On the first conflict found.
        """
        seen = set()
        for alias in aliases:
            if alias in seen:
                raise ValueError(f"duplicate alias '{alias}' for command '{cmd_name}'")
            seen.add(alias)

            if alias in _RESERVED_CLI_VERBS:
                raise ValueError(
                    f"alias '{alias}' for command '{cmd_name}' collides with a "
                    f"reserved CLI verb ({', '.join(sorted(_RESERVED_CLI_VERBS))})"
                )
            # Collision with another command's canonical name.
            if alias != cmd_name and alias in self._commands:
                raise ValueError(
                    f"alias '{alias}' for command '{cmd_name}' collides with command name '{alias}'"
                )
            # Collision with an alias owned by a different command.
            for other_name, other_meta in self._cli_meta.items():
                if other_name == cmd_name:
                    continue
                if alias in other_meta.aliases:
                    raise ValueError(
                        f"alias '{alias}' for command '{cmd_name}' already used "
                        f"by command '{other_name}'"
                    )

    def enable_cli(self, *names: Any, **_unused: Any) -> None:
        """Batch-enable CLI exposure for already-registered commands (§14.2).

        Two calling styles:

            webview.commands.enable_cli("export", "validate")             # no aliases
            webview.commands.enable_cli({"export": "exp", "validate": "v"})# with aliases

        A mapping value may be a single alias string or a list of aliases.
        Help text falls back to each command's docstring first line.

        Raises:
            KeyError: If a named command is not registered.
            ValueError: On alias conflicts (§12.4).
        """
        # Normalize the heterogeneous arguments into {name: cli_value}.
        plan: Dict[str, Union[bool, str, List[str]]] = {}
        for arg in names:
            if isinstance(arg, dict):
                for name, alias in arg.items():
                    plan[name] = alias
            elif isinstance(arg, str):
                plan[arg] = True
            else:
                raise ValueError(
                    f"enable_cli arguments must be str or dict, got {type(arg).__name__}"
                )

        for name, cli_value in plan.items():
            if name not in self._commands:
                raise KeyError(f"Unknown command: {name}")
            enabled, aliases = _normalize_cli(cli_value)
            if not enabled:
                continue
            self._register_cli_meta(name, self._commands[name], aliases, None, None)

    def cli_meta(self, name: str) -> Optional["CliCommandMeta"]:
        """Return the CLI metadata for a command, or ``None`` if not exposed."""
        return self._cli_meta.get(name)

    def list_cli_commands(self) -> List[str]:
        """List the canonical names of all CLI-exposed commands."""
        return list(self._cli_meta.keys())

    def unregister(self, name: str) -> bool:
        """Unregister a command.

        Args:
            name: Command name to unregister

        Returns:
            True if command was removed, False if not found
        """
        if name in self._commands:
            del self._commands[name]
            self._cli_meta.pop(name, None)
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

    def invoke(self, command_name: str, **kwargs: Any) -> Any:
        """Invoke a command directly from Python.

        Args:
            command_name: Command name to invoke
            **kwargs: Command arguments

        Returns:
            Command result

        Raises:
            KeyError: If command not found
        """
        if command_name not in self._commands:
            raise KeyError(f"Unknown command: {command_name}")
        return self._commands[command_name](**kwargs)

    def __len__(self) -> int:
        """Return number of registered commands."""
        return len(self._commands)

    def __contains__(self, name: str) -> bool:
        """Check if command is registered."""
        return name in self._commands

    def __repr__(self) -> str:
        """String representation."""
        return f"CommandRegistry({list(self._commands.keys())})"
