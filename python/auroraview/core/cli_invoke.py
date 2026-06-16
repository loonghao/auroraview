# -*- coding: utf-8 -*-
"""Headless CLI command invocation for packed apps (RFC 0018 §7 / §15.2).

When the Rust launcher sets ``AURORAVIEW_CLI_INVOKE=<name>`` and
``AURORAVIEW_CLI_ARGS=<json-list>``, the entry point calls :func:`run_cli_invoke`
instead of opening a window. It parses the raw argument tokens against the
target callable's signature (§6.3), invokes the command in-process — resolving
only CLI-exposed commands (§15.2) — serializes the JSON result to stdout, and
returns the §4.4 exit code.

Exit codes (§4.4):
    0  success
    1  the command raised an exception
    2  command not found / argument error
"""

from __future__ import annotations

import inspect
import json
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional, Tuple

if TYPE_CHECKING:
    from .webview import WebView

EXIT_OK = 0
EXIT_COMMAND_ERROR = 1
EXIT_USAGE = 2


class _UsageError(Exception):
    """Raised for argument/lookup problems that map to exit code 2."""


def run_cli_invoke(webview: "WebView", command: str, raw_args: List[str]) -> int:
    """Invoke ``command`` with ``raw_args`` and return the exit code.

    Writes the JSON result to stdout on success, or a structured error to
    stderr on failure. Never raises — every outcome is mapped to an exit code.
    """
    try:
        func = _lookup_command(webview, command)
        if func is None:
            _print_error("CommandNotFound", f"command not found: {command}")
            return EXIT_USAGE

        kwargs = _parse_args(func, raw_args)
    except _UsageError as exc:
        _print_error("UsageError", str(exc))
        return EXIT_USAGE

    try:
        result = func(**kwargs)
    except Exception as exc:  # noqa: BLE001 - surface as structured stderr
        _print_command_error(exc)
        return EXIT_COMMAND_ERROR

    print(json.dumps(result, ensure_ascii=False, default=str), flush=True)
    return EXIT_OK


def _lookup_command(webview: "WebView", command: str) -> Optional[Callable[..., Any]]:
    """Resolve a CLI-exposed command by canonical name or alias (§15.2).

    Only commands that opted into the CLI via ``cli != False`` are resolvable,
    mirroring the Rust ``resolve_command`` gate (which matches against the
    embedded ``cli_commands`` table — itself sourced solely from ``_cli_meta``).
    This is defense in depth: the Rust launcher already normalizes alias →
    canonical name before setting ``AURORAVIEW_CLI_INVOKE``, but gating here too
    means a hand-set ``AURORAVIEW_CLI_INVOKE`` can never reach a command that was
    never CLI-exposed (e.g. a ``bind_call``/``bind_api`` handler, which carries
    no CLI metadata).

    Accepts either the canonical name or any registered alias, matching Rust.
    """
    registry = getattr(webview, "_commands", None)
    if registry is None:
        return None

    cli_meta = getattr(registry, "_cli_meta", None)
    if not isinstance(cli_meta, dict):
        return None

    # Exact canonical-name match.
    if command in cli_meta:
        return registry._commands.get(command)

    # Alias match → resolve to the owning canonical name.
    for name, meta in cli_meta.items():
        if command in getattr(meta, "aliases", ()):
            return registry._commands.get(name)

    return None


def _parse_args(func: Callable[..., Any], raw_args: List[str]) -> Dict[str, Any]:
    """Map raw CLI tokens onto ``func``'s parameters (§6.3).

    Supports positional and ``--key value`` forms, mixed (positional first,
    keywords after). Boolean flags accept ``--flag`` / ``--no-flag``. Types are
    coerced from the signature annotation; complex types parse as JSON. A
    parameter supplied both positionally and by keyword, an unknown ``--key``,
    or a missing required argument all raise :class:`_UsageError`.
    """
    sig = _signature(func)
    params = [
        p
        for p in sig.parameters.values()
        if p.name != "self"
        and p.kind
        not in (inspect.Parameter.VAR_POSITIONAL, inspect.Parameter.VAR_KEYWORD)
    ]
    by_name = {p.name: p for p in params}

    positionals, keywords = _split_tokens(raw_args, by_name)

    if len(positionals) > len(params):
        raise _UsageError(
            f"too many positional arguments: expected at most {len(params)}, "
            f"got {len(positionals)}"
        )

    bound: Dict[str, Any] = {}

    # Positional tokens fill parameters in signature order.
    for param, token in zip(params, positionals):
        bound[param.name] = _coerce(param, token)

    # Keyword tokens override/补 by name; reject double-assignment.
    for key, raw_value in keywords.items():
        if key not in by_name:
            raise _UsageError(f"unknown option: --{key}")
        if key in bound:
            raise _UsageError(f"argument '{key}' given both positionally and as --{key}")
        bound[key] = _coerce(by_name[key], raw_value)

    # Validate required parameters are present.
    for param in params:
        if param.name in bound:
            continue
        if param.default is inspect.Parameter.empty:
            raise _UsageError(f"missing required argument: {param.name}")

    return bound


def _split_tokens(
    raw_args: List[str],
    by_name: Dict[str, inspect.Parameter],
) -> Tuple[List[str], Dict[str, Any]]:
    """Partition tokens into positionals and a ``{key: value}`` keyword map.

    ``--key value`` consumes the following token as the value. ``--flag`` with
    no following value (or followed by another option) is a boolean ``True``;
    ``--no-flag`` is boolean ``False``. Positionals may only appear before the
    first keyword (§6.3: "positional in front, keyword overrides").
    """
    positionals: List[str] = []
    keywords: Dict[str, Any] = {}
    seen_keyword = False

    i = 0
    n = len(raw_args)
    while i < n:
        token = raw_args[i]
        if token.startswith("--"):
            seen_keyword = True
            name = token[2:]
            if not name:
                raise _UsageError("empty option name '--'")

            if name.startswith("no-") and _is_bool_param(by_name.get(name[3:])):
                keywords[name[3:]] = False
                i += 1
                continue

            # A bool flag without an explicit value defaults to True.
            if _is_bool_param(by_name.get(name)):
                nxt = raw_args[i + 1] if i + 1 < n else None
                if nxt is None or nxt.startswith("--"):
                    keywords[name] = True
                    i += 1
                    continue
                keywords[name] = nxt
                i += 2
                continue

            if i + 1 >= n:
                raise _UsageError(f"option --{name} requires a value")
            keywords[name] = raw_args[i + 1]
            i += 2
        else:
            if seen_keyword:
                raise _UsageError(
                    f"positional argument '{token}' must come before keyword options"
                )
            positionals.append(token)
            i += 1

    return positionals, keywords


def _is_bool_param(param: Optional[inspect.Parameter]) -> bool:
    """Whether ``param`` is annotated (or defaulted) as ``bool``."""
    if param is None:
        return False
    if _annotation_name(param.annotation) == "bool":
        return True
    return isinstance(param.default, bool)


def _coerce(param: inspect.Parameter, value: Any) -> Any:
    """Coerce a raw token to the parameter's annotated type (§6.3)."""
    if isinstance(value, bool):
        return value

    kind = _annotation_name(param.annotation)
    if kind in ("", "str"):
        return value

    if kind == "bool":
        return _parse_bool(value)
    if kind == "int":
        try:
            return int(value)
        except (TypeError, ValueError):
            raise _UsageError(f"argument '{param.name}': expected int, got '{value}'")
    if kind == "float":
        try:
            return float(value)
        except (TypeError, ValueError):
            raise _UsageError(f"argument '{param.name}': expected float, got '{value}'")

    # Complex / unknown annotations: parse as JSON, fall back to the raw string.
    try:
        return json.loads(value)
    except (TypeError, ValueError, json.JSONDecodeError):
        return value


def _annotation_name(annotation: Any) -> str:
    """Normalize an annotation to a lowercase type name.

    Handles both real type objects and *string* annotations (which is what
    ``inspect.signature`` yields when the defining module uses
    ``from __future__ import annotations``). Returns ``""`` when there is no
    usable annotation.
    """
    if annotation is inspect.Parameter.empty or annotation is None:
        return ""
    if isinstance(annotation, type):
        return annotation.__name__
    # String annotation (PEP 563) or a typing construct: use its text form,
    # taking the leading bare word (e.g. "int", "str", "dict").
    text = str(annotation).strip()
    return text.split("[", 1)[0].split(".")[-1]


def _parse_bool(value: Any) -> bool:
    """Parse a positional boolean token (``true``/``false``/``1``/``0``)."""
    if isinstance(value, bool):
        return value
    lowered = str(value).strip().lower()
    if lowered in ("true", "1", "yes", "on"):
        return True
    if lowered in ("false", "0", "no", "off"):
        return False
    raise _UsageError(f"expected a boolean, got '{value}'")


def _signature(func: Callable[..., Any]) -> inspect.Signature:
    try:
        return inspect.signature(func)
    except (ValueError, TypeError) as exc:
        raise _UsageError(f"cannot introspect command signature: {exc}")


def _print_error(name: str, message: str) -> None:
    """Print a structured error object to stderr."""
    import sys

    print(
        json.dumps({"error": {"name": name, "message": message}}, ensure_ascii=False),
        file=sys.stderr,
        flush=True,
    )


def _print_command_error(exc: Exception) -> None:
    """Print a command exception, reusing CommandError code/message (§4.4)."""
    import sys

    payload: Dict[str, Any]
    try:
        from .commands import CommandError

        if isinstance(exc, CommandError):
            payload = {"error": {"name": exc.code.value, "message": exc.message}}
            if exc.details:
                payload["error"]["details"] = exc.details
        else:
            payload = {"error": {"name": type(exc).__name__, "message": str(exc)}}
    except Exception:  # noqa: BLE001 - defensive: never fail while reporting
        payload = {"error": {"name": type(exc).__name__, "message": str(exc)}}

    print(json.dumps(payload, ensure_ascii=False), file=sys.stderr, flush=True)
