# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""Standardized API Response Utilities.

This module provides utilities for creating standardized API responses
following the AuroraView API design guidelines.

Standard Envelope Format:
    {
        "ok": boolean,
        "data": any,       // Present if ok is true
        "error": string    // Present if ok is false
    }

Example:
    >>> from auroraview.core.response import ok, err, wrap_response
    >>>
    >>> # Simple success response
    >>> ok({"name": "test", "version": "1.0"})
    {'ok': True, 'data': {'name': 'test', 'version': '1.0'}}
    >>>
    >>> # Error response
    >>> err("File not found")
    {'ok': False, 'error': 'File not found'}
    >>>
    >>> # Auto-wrap decorator
    >>> @wrap_response
    ... def get_info():
    ...     return {"name": "test"}  # Automatically wrapped
"""

from __future__ import annotations

import functools
import logging
from typing import Any, Callable, Dict, Optional, TypeVar, Union, overload

logger = logging.getLogger(__name__)

# Type alias for standard response
Response = Dict[str, Any]

# Generic type for decorated functions
F = TypeVar("F", bound=Callable[..., Any])


def ok(data: Any = None) -> Response:
    """Create a successful response.

    Args:
        data: The data payload to include in the response.
              Can be any JSON-serializable value.

    Returns:
        Standard response dict with ok=True and data field.

    Example:
        >>> ok({"pid": 123, "mode": "pipe"})
        {'ok': True, 'data': {'pid': 123, 'mode': 'pipe'}}

        >>> ok([1, 2, 3])
        {'ok': True, 'data': [1, 2, 3]}

        >>> ok("simple value")
        {'ok': True, 'data': 'simple value'}

        >>> ok()  # No data
        {'ok': True, 'data': None}
    """
    return {"ok": True, "data": data}


def err(error: str, code: Optional[str] = None) -> Response:
    """Create an error response.

    Args:
        error: Human-readable error message.
        code: Optional error code for programmatic handling.

    Returns:
        Standard response dict with ok=False and error field.

    Example:
        >>> err("File not found")
        {'ok': False, 'error': 'File not found'}

        >>> err("Invalid parameter", code="INVALID_PARAM")
        {'ok': False, 'error': 'Invalid parameter', 'code': 'INVALID_PARAM'}
    """
    response: Response = {"ok": False, "error": error}
    if code is not None:
        response["code"] = code
    return response


def is_response(value: Any) -> bool:
    """Check if a value is already a standard response.

    A standard response must be a dict with an 'ok' boolean field.

    Args:
        value: Value to check.

    Returns:
        True if value is a standard response dict.
    """
    return isinstance(value, dict) and "ok" in value and isinstance(value.get("ok"), bool)


def normalize(value: Any) -> Response:
    """Normalize a value to standard response format.

    If the value is already a standard response (has 'ok' field), return as-is.
    Otherwise, wrap it in a success response.

    Args:
        value: Value to normalize.

    Returns:
        Standard response dict.

    Example:
        >>> normalize({"ok": True, "data": "test"})  # Already standard
        {'ok': True, 'data': 'test'}

        >>> normalize({"name": "test"})  # Wrap in data
        {'ok': True, 'data': {'name': 'test'}}

        >>> normalize([1, 2, 3])  # Wrap list
        {'ok': True, 'data': [1, 2, 3]}
    """
    if is_response(value):
        return value
    return ok(value)


@overload
def wrap_response(func: F) -> F:
    ...


@overload
def wrap_response(*, catch_exceptions: bool = True) -> Callable[[F], F]:
    ...


def wrap_response(
    func: Optional[F] = None,
    *,
    catch_exceptions: bool = True,
) -> Union[F, Callable[[F], F]]:
    """Decorator to automatically wrap function returns in standard response format.

    This decorator:
    1. Wraps non-standard returns in {"ok": True, "data": <return_value>}
    2. Passes through already-standard responses unchanged
    3. Optionally catches exceptions and converts to error responses

    Args:
        func: Function to decorate (for @wrap_response usage).
        catch_exceptions: If True (default), catch exceptions and return
                         {"ok": False, "error": <message>}.
                         If False, let exceptions propagate.

    Returns:
        Decorated function that always returns standard response.

    Example:
        >>> @wrap_response
        ... def get_info():
        ...     return {"name": "test", "version": "1.0"}
        ...
        >>> get_info()
        {'ok': True, 'data': {'name': 'test', 'version': '1.0'}}

        >>> @wrap_response
        ... def get_list():
        ...     return [1, 2, 3]
        ...
        >>> get_list()
        {'ok': True, 'data': [1, 2, 3]}

        >>> @wrap_response
        ... def already_standard():
        ...     return {"ok": True, "data": "already wrapped"}
        ...
        >>> already_standard()  # Passed through unchanged
        {'ok': True, 'data': 'already wrapped'}

        >>> @wrap_response
        ... def failing_func():
        ...     raise ValueError("Something went wrong")
        ...
        >>> failing_func()
        {'ok': False, 'error': 'Something went wrong'}

        >>> @wrap_response(catch_exceptions=False)
        ... def strict_func():
        ...     raise ValueError("Propagated")
        ...
        >>> strict_func()  # Raises ValueError
    """

    def decorator(fn: F) -> F:
        @functools.wraps(fn)
        def wrapper(*args: Any, **kwargs: Any) -> Response:
            try:
                result = fn(*args, **kwargs)
                return normalize(result)
            except Exception as e:
                if catch_exceptions:
                    logger.exception("Exception in wrapped API call '%s'", fn.__name__)
                    return err(str(e))
                raise

        return wrapper  # type: ignore

    # Support both @wrap_response and @wrap_response() syntax
    if func is not None:
        return decorator(func)
    return decorator


class ApiResponse:
    """Builder class for creating API responses with fluent interface.

    Example:
        >>> ApiResponse.success({"name": "test"}).build()
        {'ok': True, 'data': {'name': 'test'}}

        >>> ApiResponse.failure("Not found").with_code("NOT_FOUND").build()
        {'ok': False, 'error': 'Not found', 'code': 'NOT_FOUND'}
    """

    def __init__(self, ok_status: bool, data: Any = None, error: Optional[str] = None):
        self._ok = ok_status
        self._data = data
        self._error = error
        self._code: Optional[str] = None
        self._extra: Dict[str, Any] = {}

    @classmethod
    def success(cls, data: Any = None) -> "ApiResponse":
        """Create a success response builder."""
        return cls(ok_status=True, data=data)

    @classmethod
    def failure(cls, error: str) -> "ApiResponse":
        """Create a failure response builder."""
        return cls(ok_status=False, error=error)

    def with_code(self, code: str) -> "ApiResponse":
        """Add an error code."""
        self._code = code
        return self

    def with_extra(self, key: str, value: Any) -> "ApiResponse":
        """Add extra field to response (use sparingly)."""
        self._extra[key] = value
        return self

    def build(self) -> Response:
        """Build the final response dict."""
        if self._ok:
            response: Response = {"ok": True, "data": self._data}
        else:
            response = {"ok": False, "error": self._error}
            if self._code:
                response["code"] = self._code

        response.update(self._extra)
        return response


# Convenience aliases
success = ok
failure = err
