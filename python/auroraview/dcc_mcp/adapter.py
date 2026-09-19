# -*- coding: utf-8 -*-
"""AuroraView adapter for the DCC-MCP ``WebViewAdapter`` contract.

:class:`AuroraViewAdapter` bridges a live AuroraView WebView onto
``dcc_mcp_core.WebViewAdapter``, the contract core provides for hosts whose
capability profile is narrower than a full DCC -- explicitly including
"AuroraView-style browser / tool panels".

The adapter deliberately **reuses** AuroraView's existing WebView API
(``eval_js`` / ``load_url`` / ``load_html`` / the bundled
``window.auroraview.screenshot`` helper) instead of introducing a second
bridge. It adds no new execution path; it is a thin, typed dispatch layer.

Tools exposed through :meth:`AuroraViewAdapter.execute`:

============ ==========================================================
Tool         Behaviour
============ ==========================================================
``eval_js``  Evaluate JavaScript and return the value (JSON string).
``screenshot`` Capture the page via ``window.auroraview.screenshot``.
``load_url`` Load a URL (``http://``, ``https://``, ``file://``).
``load_html`` Load raw HTML content.
============ ==========================================================
"""

from __future__ import annotations

import json
import logging
import os
import time
from typing import Any, Callable, Dict, List, Optional

from ._compat import require_core as _require_core

logger = logging.getLogger(__name__)

#: Default millisecond budget for a single JavaScript round-trip.
DEFAULT_JS_TIMEOUT_MS = 5000

#: Poll interval (seconds) used while waiting for an async JS result.
_JS_POLL_INTERVAL = 0.01

#: DCC type AuroraView registers as in the DCC-MCP registry.
DCC_NAME = "auroraview"


class WebViewToolSpec:
    """Declarative metadata for one tool exposed by the adapter.

    Attributes:
        name: Tool name used by ``execute()``.
        description: Human readable summary surfaced to agents.
        parameters: JSON-Schema-ish description of accepted parameters.
    """

    __slots__ = ("name", "description", "parameters")

    def __init__(self, name: str, description: str, parameters: Optional[Dict[str, Any]] = None):
        self.name = name
        self.description = description
        self.parameters = parameters or {}

    def to_dict(self) -> Dict[str, Any]:
        """Return the tool as a plain dict (JSON-serialisable)."""
        return {
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters,
        }


#: The four tools exposed in the first DCC-MCP slice. They mirror the
#: capabilities already offered by AuroraView's own MCP surface so agents see
#: one consistent tool set rather than two competing ones.
TOOL_SPECS = (
    WebViewToolSpec(
        "eval_js",
        "Evaluate JavaScript in the AuroraView WebView and return the result.",
        {
            "type": "object",
            "properties": {
                "script": {"type": "string", "description": "JavaScript code to evaluate."},
                "timeout_ms": {
                    "type": "integer",
                    "description": "Optional evaluation budget in milliseconds.",
                },
            },
            "required": ["script"],
        },
    ),
    WebViewToolSpec(
        "screenshot",
        "Capture the current page as a PNG (base64) via window.auroraview.screenshot.",
        {
            "type": "object",
            "properties": {
                "format": {"type": "string", "description": "Image format (default: png)."},
                "full_page": {
                    "type": "boolean",
                    "description": "Capture the full page instead of the viewport.",
                },
            },
        },
    ),
    WebViewToolSpec(
        "load_url",
        "Load a URL into the AuroraView WebView.",
        {
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to load (http://, https://, or file://).",
                },
            },
            "required": ["url"],
        },
    ),
    WebViewToolSpec(
        "load_html",
        "Load raw HTML content into the AuroraView WebView.",
        {
            "type": "object",
            "properties": {
                "html": {"type": "string", "description": "HTML content to load."},
            },
            "required": ["html"],
        },
    ),
)

_TOOLS_BY_NAME = {spec.name: spec for spec in TOOL_SPECS}


def _resolve_core(view: Any) -> Any:
    """Return the underlying Rust core object for ``view``.

    Accepts either a low-level :class:`auroraview.core.WebView` or a
    :class:`auroraview.QtWebView`, which wraps one via ``_webview``.

    Args:
        view: AuroraView WebView or QtWebView instance.

    Returns:
        The Rust core object, or ``None`` when it cannot be resolved.
    """
    core = getattr(view, "_core", None)
    if core is not None:
        return core
    inner = getattr(view, "_webview", None)
    if inner is not None:
        return getattr(inner, "_core", None)
    return None


def _resolve_process_events(view: Any) -> Optional[Callable[[], Any]]:
    """Return a callable that pumps the WebView message queue, if available.

    Args:
        view: AuroraView WebView or QtWebView instance.

    Returns:
        Zero-argument callable, or ``None`` when event pumping is unavailable.
    """
    for candidate in (view, getattr(view, "_webview", None)):
        if candidate is None:
            continue
        fn = getattr(candidate, "process_events", None)
        if callable(fn):
            return fn
    return None


class AuroraViewAdapter:
    """Expose a live AuroraView WebView through the DCC-MCP WebView contract.

    Subclasses ``dcc_mcp_core.WebViewAdapter`` when that package is installed.
    The base class advertises an all-``False`` capability map
    (``scene`` / ``timeline`` / ``selection`` / ``undo`` / ``render``), which is
    accurate for AuroraView: a WebView host owns no scene graph and no
    selection model.

    Args:
        view: Live AuroraView WebView or QtWebView instance.
        host_dcc: Name of the DCC embedding this WebView, if any
            (e.g. ``"maya"``). ``None`` for standalone windows.
        cdp_port: Chrome DevTools Protocol port, when the host enabled it.
            ``0``/``None`` when not available.

    Example::

        adapter = AuroraViewAdapter(webview, host_dcc="maya")
        adapter.execute("eval_js", {"script": "document.title"})
    """

    #: DCC short-name this adapter registers as.
    dcc_name = DCC_NAME

    #: Capability descriptor advertised to the gateway/registry. AuroraView has
    #: no scene, timeline, selection, undo stack, or render farm of its own.
    capabilities = {"scene": False, "timeline": False, "selection": False, "undo": False, "render": False}

    def __init__(
        self,
        view: Any,
        host_dcc: Optional[str] = None,
        cdp_port: Optional[int] = None,
    ):
        _require_core()
        if view is None:
            raise ValueError("AuroraViewAdapter requires a live WebView instance")
        self._view = view
        self._host_dcc = host_dcc
        self._cdp_port = cdp_port or 0
        self._audit: List[Dict[str, Any]] = []

    # ------------------------------------------------------------------
    # WebViewAdapter contract
    # ------------------------------------------------------------------

    def get_context(self) -> Dict[str, Any]:
        """Return the descriptor dict for the current WebView host session.

        Keys follow the ``WebViewContext`` convention understood by core:
        ``window_title``, ``url``, ``pid``, ``cdp_port``, ``host_dcc``.

        Returns:
            Descriptor dict (always JSON-serialisable).
        """
        view = self._view
        url = ""
        title = ""
        try:
            url = getattr(view, "current_url", "") or ""
        except Exception:  # pragma: no cover - defensive, GUI dependent
            url = ""
        try:
            title = getattr(view, "title", "") or ""
        except Exception:  # pragma: no cover - defensive, GUI dependent
            title = ""

        context = {
            "window_title": title,
            "url": url,
            "pid": os.getpid(),
            "cdp_port": self._cdp_port,
        }
        if self._host_dcc:
            context["host_dcc"] = self._host_dcc
        return context

    def list_tools(self) -> List[Dict[str, Any]]:
        """Return tool declaration dicts exposed by this host.

        Returns:
            List of tool metadata dicts.
        """
        return [spec.to_dict() for spec in TOOL_SPECS]

    def execute(self, tool: str, params: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Invoke ``tool`` on the host WebView and return its result.

        Args:
            tool: One of ``eval_js``, ``screenshot``, ``load_url``, ``load_html``.
            params: Tool parameters.

        Returns:
            Result dict. Always contains ``"ok"``; on failure also ``"error"``.
        """
        params = dict(params or {})
        handler = getattr(self, "_tool_" + str(tool), None)
        if handler is None:
            return self._fail(tool, "unknown tool: {0}".format(tool))

        started = time.monotonic()
        try:
            result = handler(params)
        except Exception as exc:  # noqa: BLE001 - surfaced to the agent as data
            logger.warning("AuroraView tool %r failed: %s", tool, exc, exc_info=True)
            return self._fail(tool, str(exc))

        self._record(tool, params, started, error=None)
        payload = {"ok": True, "tool": tool}
        payload.update(result or {})
        return payload

    def get_audit_log(self, limit: int = 100) -> List[Dict[str, Any]]:
        """Return up to ``limit`` most recent audit entries.

        Args:
            limit: Maximum number of entries to return.

        Returns:
            List of audit dicts (most recent first).
        """
        if limit <= 0:
            return []
        return list(self._audit[-limit:])

    # ------------------------------------------------------------------
    # Capability helpers (delegated/overridden from the core base class)
    # ------------------------------------------------------------------

    @classmethod
    def advertised_capabilities(cls) -> Dict[str, bool]:
        """Return a fresh copy of the advertised capability map."""
        return dict(cls.capabilities)

    @classmethod
    def supports(cls, capability: str) -> bool:
        """Return ``True`` when ``capability`` is advertised as enabled."""
        return bool(cls.capabilities.get(capability, False))

    @classmethod
    def matches_requirements(cls, required) -> bool:
        """Return ``True`` when every required capability is advertised."""
        return all(cls.supports(key) for key in required)

    # ------------------------------------------------------------------
    # Tool implementations
    # ------------------------------------------------------------------

    def _tool_eval_js(self, params: Dict[str, Any]) -> Dict[str, Any]:
        script = params.get("script")
        if not script:
            raise ValueError("'script' is required for eval_js")
        timeout_ms = int(params.get("timeout_ms") or DEFAULT_JS_TIMEOUT_MS)
        value = self._eval_js_sync(str(script), timeout_ms)
        return {"result": value}

    def _tool_screenshot(self, params: Dict[str, Any]) -> Dict[str, Any]:
        fmt = str(params.get("format") or "png")
        full_page = bool(params.get("full_page", False))
        script = (
            "window.auroraview.screenshot({{format: {0}, fullPage: {1}}})"
        ).format(json.dumps(fmt), "true" if full_page else "false")
        timeout_ms = int(params.get("timeout_ms") or DEFAULT_JS_TIMEOUT_MS)
        data = self._eval_js_sync(script, timeout_ms)
        return {"format": fmt, "data": data}

    def _tool_load_url(self, params: Dict[str, Any]) -> Dict[str, Any]:
        url = params.get("url")
        if not url:
            raise ValueError("'url' is required for load_url")
        self._view.load_url(str(url))
        return {"url": str(url)}

    def _tool_load_html(self, params: Dict[str, Any]) -> Dict[str, Any]:
        html = params.get("html")
        if html is None:
            raise ValueError("'html' is required for load_html")
        self._view.load_html(str(html))
        return {"length": len(str(html))}

    # ------------------------------------------------------------------
    # Internals
    # ------------------------------------------------------------------

    def _eval_js_sync(self, script: str, timeout_ms: int) -> Any:
        """Evaluate ``script`` and synchronously wait for its value.

        Reuses AuroraView's existing ``eval_js_future`` / ``get_js_result``
        round-trip when the Rust core provides it, pumping the WebView event
        queue while waiting. Falls back to fire-and-forget ``eval_js`` when the
        round-trip API is unavailable.

        Args:
            script: JavaScript to evaluate.
            timeout_ms: Wait budget in milliseconds.

        Returns:
            The JavaScript result (typically a JSON string), or ``None``.
        """
        core = _resolve_core(self._view)
        pump = _resolve_process_events(self._view)

        if core is None or not hasattr(core, "eval_js_future"):
            logger.warning("Core lacks eval_js_future; falling back to fire-and-forget eval_js")
            self._view.eval_js(script)
            if pump is not None:
                pump()
            return None

        callback_id = core.eval_js_future(script, timeout_ms)
        deadline = time.monotonic() + (timeout_ms / 1000.0)
        while True:
            result = core.get_js_result(callback_id) or {}
            status = result.get("status", "pending")
            if status == "complete":
                return result.get("result")
            if status == "error":
                raise RuntimeError(str(result.get("error") or "JavaScript execution failed"))
            if status == "timeout":
                raise TimeoutError("JavaScript timed out after {0}ms".format(timeout_ms))
            if time.monotonic() >= deadline:
                raise TimeoutError("JavaScript timed out after {0}ms".format(timeout_ms))
            if pump is not None:
                pump()
            time.sleep(_JS_POLL_INTERVAL)

    def _fail(self, tool: str, message: str) -> Dict[str, Any]:
        self._record(tool, None, time.monotonic(), error=message)
        return {"ok": False, "tool": tool, "error": message}

    def _record(
        self,
        tool: str,
        params: Optional[Dict[str, Any]],
        started: float,
        error: Optional[str],
    ) -> None:
        """Append one entry to the in-memory audit log."""
        entry = {
            "tool": tool,
            "duration_ms": round((time.monotonic() - started) * 1000.0, 3),
            "error": error,
        }
        if params:
            entry["params"] = {k: v for k, v in params.items() if k != "html"}
        self._audit.append(entry)
        # Bound memory: keep the audit log from growing without limit.
        if len(self._audit) > 1000:
            del self._audit[: len(self._audit) - 1000]
