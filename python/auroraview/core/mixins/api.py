# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView API Binding Mixin.

This module provides API binding methods for the WebView class.
"""

from __future__ import annotations

import logging
from threading import Lock
from typing import TYPE_CHECKING, Any, Callable, Dict, Optional, Set

from ..response import normalize

# Use Rust's high-performance JSON serialization (2-3x faster than json.dumps)
try:
    from ..._core import json_dumps
except ImportError:
    # Fallback to Python's json for development without compiled extension
    import json

    def json_dumps(obj: Any) -> str:  # type: ignore[misc]
        """Fallback JSON serialization."""
        return json.dumps(obj)

if TYPE_CHECKING:
    pass

logger = logging.getLogger(__name__)


class WebViewApiMixin:
    """Mixin providing API binding methods.

    Provides methods for binding Python functions to JavaScript:
    - register_protocol: Register a custom protocol handler
    - bind_call: Bind a Python callable as an auroraview.call target
    - bind_api: Bind all public methods of an object

    Thread-Safety:
        All binding operations are protected by a lock to prevent race conditions
        when multiple threads attempt to bind APIs simultaneously.
    """

    # Type hints for attributes from main class
    _core: Any
    eval_js: Callable[[str], None]
    emit: Callable[..., None]

    # Registry of bound functions and lock for thread safety
    _bound_functions: Dict[str, Callable[..., Any]]
    _bound_namespaces: Set[str]  # Namespaces that have been bound (for idempotency)
    _bind_lock: Lock
    _is_loaded: bool  # Page loaded state

    def _init_api_registry(self) -> None:
        """Initialize the API binding registry.

        This should be called during WebView initialization to set up
        the internal registry for tracking bound functions and page state.
        """
        if not hasattr(self, "_bound_functions"):
            self._bound_functions = {}
        if not hasattr(self, "_bound_namespaces"):
            self._bound_namespaces = set()
        if not hasattr(self, "_bind_lock"):
            self._bind_lock = Lock()
        if not hasattr(self, "_is_loaded"):
            self._is_loaded = False

    def _ensure_api_registry(self) -> None:
        """Ensure the API registry is initialized (lazy initialization)."""
        if (
            not hasattr(self, "_bound_functions")
            or not hasattr(self, "_bound_namespaces")
            or not hasattr(self, "_bind_lock")
        ):
            self._init_api_registry()

    def _set_loaded(self, loaded: bool = True) -> None:
        """Set the page loaded state.

        This should be called when the page finishes loading.

        Args:
            loaded: Whether the page is loaded (default True)
        """
        self._ensure_api_registry()
        self._is_loaded = loaded
        logger.debug("Page loaded state set to %s", loaded)

    def is_loaded(self) -> bool:
        """Check if the page has finished loading.

        Returns:
            True if page is loaded, False otherwise.
        """
        self._ensure_api_registry()
        return self._is_loaded

    def is_method_bound(self, method: str) -> bool:
        """Check if a method is already bound.

        Args:
            method: Method name to check (e.g., "api.echo")

        Returns:
            True if the method is already bound, False otherwise.
        """
        self._ensure_api_registry()
        return method in self._bound_functions

    def get_bound_methods(self) -> list:
        """Get list of all bound method names.

        Returns:
            List of bound method names.
        """
        self._ensure_api_registry()
        return list(self._bound_functions.keys())

    def register_protocol(self, scheme: str, handler: Callable[[str], Dict[str, Any]]) -> None:
        """Register a custom protocol handler.

        Args:
            scheme: Protocol scheme (e.g., "maya", "fbx")
            handler: Python function that takes URI string and returns dict with:
                - data (bytes): Response data
                - mime_type (str): MIME type (e.g., "image/png")
                - status (int): HTTP status code (e.g., 200, 404)

        Example:
            >>> def handle_fbx(uri: str) -> dict:
            ...     path = uri.replace("fbx://", "")
            ...     try:
            ...         with open(f"C:/models/{path}", "rb") as f:
            ...             return {
            ...                 "data": f.read(),
            ...                 "mime_type": "application/octet-stream",
            ...                 "status": 200
            ...             }
            ...     except FileNotFoundError:
            ...         return {
            ...             "data": b"Not Found",
            ...             "mime_type": "text/plain",
            ...             "status": 404
            ...         }
            ...
            >>> webview.register_protocol("fbx", handle_fbx)
        """
        self._core.register_protocol(scheme, handler)
        logger.debug(f"Registered custom protocol: {scheme}")

    def _emit_call_result_js(self, payload: Dict[str, Any]) -> None:
        """Internal helper to emit __auroraview_call_result via eval_js.

        This is a compatibility path for environments where the core
        event bridge does not reliably dispatch DOM CustomEvents.
        Uses window.auroraview.trigger() for consistent event handling.
        """
        try:
            # Use Rust's high-performance JSON serialization
            json_str = json_dumps(payload)
        except Exception as exc:  # pragma: no cover
            logger.error("Failed to JSON-encode __auroraview_call_result payload: %s", exc)
            print(
                f"[AuroraView DEBUG] Failed to JSON-encode __auroraview_call_result payload: {exc}"
            )
            return

        # Use auroraview.trigger() for consistent event handling
        script = (
            "(function() {"
            "  if (window.auroraview && window.auroraview.trigger) {"
            f"    window.auroraview.trigger('__auroraview_call_result', JSON.parse({json_str!r}));"
            "  } else {"
            "    console.error('[AuroraView] Event bridge not ready, cannot emit call_result');"
            "  }"
            "})();"
        )
        print(f"[AuroraView DEBUG] _emit_call_result_js dispatching payload to JS: {payload}")
        try:
            self.eval_js(script)
        except Exception as exc:  # pragma: no cover
            logger.error("Failed to dispatch __auroraview_call_result via eval_js: %s", exc)
            print(
                f"[AuroraView DEBUG] Failed to dispatch __auroraview_call_result via eval_js: {exc}"
            )

    def bind_call(
        self,
        method: str,
        func: Optional[Callable[..., Any]] = None,
        *,
        allow_rebind: bool = True,
        mcp: bool = True,
        mcp_name: Optional[str] = None,
    ):
        """Bind a Python callable as an ``auroraview.call`` target.

        The JavaScript side sends messages of the form::

            {"id": "<request-id>", "params": ...}

        This helper unwraps the ``params`` payload, calls ``func`` and then
        emits a ``__auroraview_call_result`` event back to JavaScript so that
        the Promise returned by ``auroraview.call`` can resolve or reject.

        **Auto-normalization**: Return values are automatically wrapped into
        the standard response format ``{ok: bool, data: any, error?: string}``.
        Developers can return values in any of these ways:

        1. **Simple return** (recommended): Just return the data directly.
           The framework wraps it as ``{ok: True, data: <return_value>}``::

               @webview.bind_call("api.get_user")
               def get_user(id: int):
                   return {"name": "Alice", "id": id}  # Auto-wrapped

        2. **Explicit response**: Return a dict with ``ok`` field for full control::

               @webview.bind_call("api.delete_user")
               def delete_user(id: int):
                   if not user_exists(id):
                       return {"ok": False, "error": "User not found"}
                   delete(id)
                   return {"ok": True, "data": {"deleted": id}}

        3. **Using helpers**: Use ``ok()``/``err()`` for cleaner code::

               from auroraview import ok, err

               @webview.bind_call("api.create_user")
               def create_user(name: str):
                   if not name:
                       return err("Name is required")
                   return ok({"id": 123, "name": name})

        Usage::

            def echo(params):
                return params

            webview.bind_call("api.echo", echo)

        Or as a decorator::

            @webview.bind_call("api.echo")
            def echo(params):
                return params

        Args:
            method: Method name (e.g., "api.echo")
            func: Python callable to bind
            allow_rebind: If True (default), allows rebinding an already bound method.
                         If False, skips binding if method is already bound.
            mcp: If True (default), exposes this method as an MCP tool.
                 If False, this method will NOT be exposed to MCP clients.
            mcp_name: Optional custom name for MCP tool. If provided, this name
                      will be used instead of the method name for MCP registration.
                      Useful for providing more user-friendly tool names.

        Returns:
            The original function (for decorator usage)

        NOTE: Currently only synchronous callables are supported.

        MCP Integration:
            When auto_expose_api is enabled, methods with mcp=True are automatically
            registered as MCP tools. Use mcp=False to hide internal methods, and
            mcp_name to customize the tool name for AI assistants.
        """
        self._ensure_api_registry()

        # Decorator usage: @webview.bind_call("api.echo")
        if func is None:

            def decorator(fn: Callable[..., Any]) -> Callable[..., Any]:
                self.bind_call(method, fn, allow_rebind=allow_rebind, mcp=mcp, mcp_name=mcp_name)
                return fn

            return decorator

        # Thread-safe binding with duplicate detection
        with self._bind_lock:
            if method in self._bound_functions:
                if not allow_rebind:
                    logger.debug("Method '%s' already bound, skipping (allow_rebind=False)", method)
                    return func
                logger.debug("Rebinding method '%s' with new function", method)
            else:
                logger.debug("Binding new method '%s'", method)

            # Store the function reference
            self._bound_functions[method] = func

            # Store MCP metadata
            if not hasattr(self, "_mcp_metadata"):
                self._mcp_metadata = {}
            self._mcp_metadata[method] = {
                "mcp": mcp,
                "mcp_name": mcp_name,
            }
            logger.debug(
                "MCP metadata for '%s': exposed=%s, custom_name=%s",
                method,
                mcp,
                mcp_name,
            )

        def _handler(raw: Dict[str, Any]) -> None:
            print(f"[AuroraView DEBUG] _handler invoked for method={method} with raw={raw}")

            call_id = raw.get("id") or raw.get("__auroraview_call_id")
            has_params_key = "params" in raw
            params = raw.get("params")

            # Get the latest bound function (allows for hot-reload scenarios)
            current_func = self._bound_functions.get(method, func)

            try:
                if not has_params_key:
                    result = current_func()
                elif isinstance(params, dict):
                    result = current_func(**params)
                elif isinstance(params, list):
                    result = current_func(*params)
                else:
                    result = current_func(params)

                # Auto-normalize: wrap any return value into standard response format
                # If result is already {ok, data/error}, pass through unchanged
                # Otherwise wrap as {ok: True, data: result}
                normalized = normalize(result)
                call_ok = normalized.get("ok", True)
                if call_ok:
                    call_result = normalized.get("data")
                    error_info = None
                else:
                    call_result = None
                    # Extract error info from normalized response
                    error_msg = normalized.get("error", "Unknown error")
                    error_info = {"name": "Error", "message": error_msg}
                    if "code" in normalized:
                        error_info["code"] = normalized["code"]

            except Exception as exc:  # pragma: no cover
                call_ok = False
                call_result = None
                error_info = {
                    "name": exc.__class__.__name__,
                    "message": str(exc),
                }
                logger.exception("Error in bound call '%s'", method)

            if not call_id:
                return

            payload: Dict[str, Any] = {"id": call_id, "ok": call_ok}
            if call_ok:
                payload["result"] = call_result
            else:
                payload["error"] = error_info

            print(
                f"[AuroraView DEBUG] bind_call sending result: method={method}, id={call_id}, ok={call_ok}"
            )

            try:
                self.emit("__auroraview_call_result", payload)
            except Exception:
                logger.debug(
                    "WebView.emit for __auroraview_call_result raised; falling back to eval_js"
                )
                print(
                    "[AuroraView DEBUG] WebView.emit for __auroraview_call_result raised; "
                    "falling back to eval_js"
                )
                # Only use eval_js fallback if emit failed
                self._emit_call_result_js(payload)

        # Register wrapper with core IPC handler
        self._core.on(method, _handler)
        logger.info("Bound auroraview.call handler: %s", method)

        # Register API method in JavaScript (high-performance path via Rust)
        # Parse namespace and method name from full method path (e.g., "api.echo" -> "api", "echo")
        if "." in method:
            parts = method.split(".", 1)
            namespace = parts[0]
            method_name = parts[1]
            self._core.register_api_methods(namespace, [method_name])
            logger.debug("Registered JS API method: %s.%s", namespace, method_name)

        # For decorator-style usage, return the original function
        return func

    def bind_api(
        self,
        api: Any,
        namespace: str = "api",
        *,
        allow_rebind: bool = False,
    ) -> None:
        """Bind all public methods of an object under a namespace.

        This is a convenience helper so that you can expose a Python "API" object
        to JavaScript without writing many ``bind_call`` lines by hand.

        Idempotency:
            This method is idempotent at the namespace level. If a namespace has
            already been bound, subsequent calls will be silently skipped unless
            ``allow_rebind=True`` is explicitly specified. This prevents accidental
            duplicate bindings and eliminates the need for callers to track binding
            state.

        Example::

            class API:
                def echo(self, message: str) -> str:
                    return message

            api = API()
            webview.bind_api(api)  # JS: await auroraview.api.echo({"message": "hi"})
            webview.bind_api(api)  # Safe: silently skipped (idempotent)

        Args:
            api: Object whose public callables should be exposed.
            namespace: Logical namespace prefix used on the JS side (default: "api").
            allow_rebind: If True, allows rebinding an already bound namespace.
                         If False (default), skips binding if namespace is already
                         bound (idempotent behavior).

        Thread-Safety:
            This method is thread-safe and uses locking internally.

        Performance:
            Optimized to minimize Python-Rust boundary crossings and redundant operations:
            - Namespace-level idempotency check (O(1) set lookup)
            - Single pass collection of methods with callable references
            - Batch IPC handler registration
            - Single Rust call for JS method registration
        """
        self._ensure_api_registry()

        # Namespace-level idempotency check
        with self._bind_lock:
            if namespace in self._bound_namespaces:
                if not allow_rebind:
                    logger.debug(
                        "Namespace '%s' already bound, skipping (idempotent)",
                        namespace,
                    )
                    return
                logger.info(
                    "Rebinding namespace '%s' (allow_rebind=True)",
                    namespace,
                )

        # Collect methods with their callables in a single pass
        # Dict[method_name, (short_name, callable)] to avoid duplicate getattr()
        methods_to_bind: Dict[str, tuple] = {}
        skipped_count = 0

        with self._bind_lock:
            for name in dir(api):
                if name.startswith("_"):
                    continue

                attr = getattr(api, name)
                if not callable(attr):
                    continue

                method_name = f"{namespace}.{name}"

                # Check for duplicate binding
                if method_name in self._bound_functions:
                    if not allow_rebind:
                        skipped_count += 1
                        continue
                    # allow_rebind=True: will rebind below

                # Store both name and callable to avoid second getattr()
                methods_to_bind[method_name] = (name, attr)

        if not methods_to_bind:
            if skipped_count > 0:
                logger.debug(
                    "All %d methods in namespace '%s' already bound (allow_rebind=False)",
                    skipped_count,
                    namespace,
                )
            return

        # Batch bind all methods - optimized inner loop
        method_names = []
        callbacks_to_register = []  # Collect callbacks for batch registration

        for method_name, (short_name, func) in methods_to_bind.items():
            # Store the function reference
            self._bound_functions[method_name] = func

            # Create the handler for this method
            handler = self._create_ipc_handler(method_name, func)
            callbacks_to_register.append((method_name, handler))
            method_names.append(short_name)

        # Batch register all callbacks with Rust core (single log entry)
        if callbacks_to_register:
            if hasattr(self._core, "on_batch"):
                # Use batch registration if available (more efficient)
                self._core.on_batch(callbacks_to_register)
            else:
                # Fallback to individual registration
                for method_name, handler in callbacks_to_register:
                    self._core.on(method_name, handler)

        # Single log entry for all methods (instead of 2x per method)
        logger.info(
            "Bound %d API methods for namespace '%s': %s",
            len(method_names),
            namespace,
            ", ".join(method_names),
        )

        if skipped_count > 0:
            logger.debug(
                "Skipped %d already-bound methods in namespace '%s'",
                skipped_count,
                namespace,
            )

        # Register API methods in Rust (high-performance path)
        # Single Rust call generates optimized JS via Askama templates
        self._core.register_api_methods(namespace, method_names)

        # Mark namespace as bound (for idempotency)
        with self._bind_lock:
            self._bound_namespaces.add(namespace)

    def is_namespace_bound(self, namespace: str) -> bool:
        """Check if a namespace has been bound.

        Args:
            namespace: The namespace to check (e.g., "api").

        Returns:
            True if the namespace has been bound, False otherwise.
        """
        self._ensure_api_registry()
        return namespace in self._bound_namespaces

    def _create_ipc_handler(self, method: str, func: Callable[..., Any]) -> Callable:
        """Create an IPC handler for a bound method (internal).

        This creates a handler function without registering it, allowing for
        batch registration of multiple handlers.

        Args:
            method: Full method name (e.g., "api.echo")
            func: Python callable to invoke

        Returns:
            Handler function to be registered with the IPC system.
        """

        def _handler(raw: Dict[str, Any]) -> None:
            call_id = raw.get("id") or raw.get("__auroraview_call_id")
            has_params_key = "params" in raw
            params = raw.get("params")

            # Get the latest bound function (allows for hot-reload scenarios)
            current_func = self._bound_functions.get(method, func)

            try:
                if not has_params_key:
                    result = current_func()
                elif isinstance(params, dict):
                    result = current_func(**params)
                elif isinstance(params, list):
                    result = current_func(*params)
                else:
                    result = current_func(params)

                # Auto-normalize: wrap any return value into standard response format
                normalized = normalize(result)
                call_ok = normalized.get("ok", True)
                if call_ok:
                    call_result = normalized.get("data")
                    error_info = None
                else:
                    call_result = None
                    error_msg = normalized.get("error", "Unknown error")
                    error_info = {"name": "Error", "message": error_msg}
                    if "code" in normalized:
                        error_info["code"] = normalized["code"]

            except Exception as exc:  # pragma: no cover
                call_ok = False
                call_result = None
                error_info = {
                    "name": exc.__class__.__name__,
                    "message": str(exc),
                }
                logger.exception("Error in bound call '%s'", method)

            if not call_id:
                return

            payload: Dict[str, Any] = {"id": call_id, "ok": call_ok}
            if call_ok:
                payload["result"] = call_result
            else:
                payload["error"] = error_info

            try:
                self.emit("__auroraview_call_result", payload)
            except Exception:
                logger.debug(
                    "WebView.emit for __auroraview_call_result raised; falling back to eval_js"
                )
                # Only use eval_js fallback if emit failed
                self._emit_call_result_js(payload)

        return _handler

    def _register_ipc_handler(self, method: str, func: Callable[..., Any]) -> None:
        """Register IPC handler for a bound method (internal, no locking).

        This is an optimized internal method used by bind_api for batch registration.
        For individual method binding, use bind_call() instead.

        Args:
            method: Full method name (e.g., "api.echo")
            func: Python callable to invoke
        """
        handler = self._create_ipc_handler(method, func)
        # Register wrapper with core IPC handler
        self._core.on(method, handler)
