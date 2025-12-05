# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""WebView API Binding Mixin.

This module provides API binding methods for the WebView class.
"""

from __future__ import annotations

import json
import logging
from typing import TYPE_CHECKING, Any, Callable, Dict, Optional

if TYPE_CHECKING:
    pass

logger = logging.getLogger(__name__)


class WebViewApiMixin:
    """Mixin providing API binding methods.

    Provides methods for binding Python functions to JavaScript:
    - register_protocol: Register a custom protocol handler
    - bind_call: Bind a Python callable as an auroraview.call target
    - bind_api: Bind all public methods of an object
    """

    # Type hints for attributes from main class
    _core: Any
    eval_js: Callable[[str], None]
    emit: Callable[..., None]

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
        """
        try:
            json_str = json.dumps(payload)
        except Exception as exc:  # pragma: no cover
            logger.error("Failed to JSON-encode __auroraview_call_result payload: %s", exc)
            print(
                f"[AuroraView DEBUG] Failed to JSON-encode __auroraview_call_result payload: {exc}"
            )
            return

        script = (
            "window.dispatchEvent(new CustomEvent('__auroraview_call_result', "
            f"{{ detail: JSON.parse({json_str!r}) }}));"
        )
        print(f"[AuroraView DEBUG] _emit_call_result_js dispatching payload to JS: {payload}")
        try:
            self.eval_js(script)
        except Exception as exc:  # pragma: no cover
            logger.error("Failed to dispatch __auroraview_call_result via eval_js: %s", exc)
            print(
                f"[AuroraView DEBUG] Failed to dispatch __auroraview_call_result via eval_js: {exc}"
            )

    def bind_call(self, method: str, func: Optional[Callable[..., Any]] = None):
        """Bind a Python callable as an ``auroraview.call`` target.

        The JavaScript side sends messages of the form::

            {"id": "<request-id>", "params": ...}

        This helper unwraps the ``params`` payload, calls ``func`` and then
        emits a ``__auroraview_call_result`` event back to JavaScript so that
        the Promise returned by ``auroraview.call`` can resolve or reject.

        Usage::

            def echo(params):
                return params

            webview.bind_call("api.echo", echo)

        Or as a decorator::

            @webview.bind_call("api.echo")
            def echo(params):
                return params

        NOTE: Currently only synchronous callables are supported.
        """

        # Decorator usage: @webview.bind_call("api.echo")
        if func is None:

            def decorator(fn: Callable[..., Any]) -> Callable[..., Any]:
                self.bind_call(method, fn)
                return fn

            return decorator

        def _handler(raw: Dict[str, Any]) -> None:
            print(f"[AuroraView DEBUG] _handler invoked for method={method} with raw={raw}")

            call_id = raw.get("id") or raw.get("__auroraview_call_id")
            has_params_key = "params" in raw
            params = raw.get("params")

            try:
                if not has_params_key:
                    result = func()
                elif isinstance(params, dict):
                    result = func(**params)
                elif isinstance(params, list):
                    result = func(*params)
                else:
                    result = func(params)
                ok = True
                error_info: Optional[Dict[str, Any]] = None
            except Exception as exc:  # pragma: no cover
                ok = False
                result = None
                error_info = {
                    "name": exc.__class__.__name__,
                    "message": str(exc),
                }
                logger.exception("Error in bound call '%s'", method)

            if not call_id:
                return

            payload: Dict[str, Any] = {"id": call_id, "ok": ok}
            if ok:
                payload["result"] = result
            else:
                payload["error"] = error_info

            print(
                f"[AuroraView DEBUG] bind_call sending result: method={method}, id={call_id}, ok={ok}"
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
            self._emit_call_result_js(payload)

        # Register wrapper with core IPC handler
        self._core.on(method, _handler)
        logger.info("Bound auroraview.call handler: %s", method)

        # For decorator-style usage, return the original function
        return func

    def bind_api(self, api: Any, namespace: str = "api") -> None:
        """Bind all public methods of an object under a namespace.

        This is a convenience helper so that you can expose a Python "API" object
        to JavaScript without writing many ``bind_call`` lines by hand.

        Example::

            class API:
                def echo(self, message: str) -> str:
                    return message

            api = API()
            webview.bind_api(api)  # JS: await auroraview.api.echo({"message": "hi"})

        Args:
            api: Object whose public callables should be exposed.
            namespace: Logical namespace prefix used on the JS side (default: "api").
        """

        method_names = []
        for name in dir(api):
            if name.startswith("_"):
                continue

            attr = getattr(api, name)
            if not callable(attr):
                continue

            method_name = f"{namespace}.{name}"
            self.bind_call(method_name, attr)
            method_names.append(name)
            logger.info("Bound auroraview.call handler via bind_api: %s", method_name)

        # Register API methods in Rust (will be injected into JavaScript)
        if method_names:
            try:
                self._core.register_api_methods(namespace, method_names)
                logger.info(
                    "Registered %d API methods in Rust for namespace '%s'",
                    len(method_names),
                    namespace,
                )
            except AttributeError:
                logger.warning(
                    "register_api_methods not available in core, using JS fallback"
                )
                self._inject_api_methods_via_js(namespace, method_names)
            except Exception as e:
                logger.warning(
                    f"register_api_methods failed ({e}), using JS fallback"
                )
                self._inject_api_methods_via_js(namespace, method_names)

    def _inject_api_methods_via_js(self, namespace: str, method_names: list) -> None:
        """Inject API methods via JavaScript when Rust method is unavailable."""
        methods_json = ", ".join(f"'{m}'" for m in method_names)
        js_code = f"""
        (function() {{
            if (window.auroraview && window.auroraview._registerApiMethods) {{
                window.auroraview._registerApiMethods('{namespace}', [{methods_json}]);
            }} else {{
                console.warn('[AuroraView] Event bridge not ready, API methods will be registered on page load');
            }}
        }})();
        """
        try:
            self._core.eval_js(js_code)
        except Exception as e:
            logger.debug(f"Could not inject API methods immediately: {e}")

