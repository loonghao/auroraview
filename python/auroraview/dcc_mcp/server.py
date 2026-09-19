# -*- coding: utf-8 -*-
"""MCP/REST server + FileRegistry registration for AuroraView.

Starting a server through this module is what makes an AuroraView window
**discoverable** by the ``dcc-mcp-cli`` control plane: core's
``DccServerBase`` writes a registry row keyed by ``(dcc_type, instance_id)``
into the shared FileRegistry (``services.json``), holds an OS-level sentinel
lock for crash-resilient liveness, and heartbeats the row while the process
lives.

.. important::
   ``gateway_port`` must be non-zero. Setting it to ``0`` is core's explicit
   opt-out and disables FileRegistry self-registration entirely -- the server
   still runs but ``dcc-mcp-cli list`` will never see it.

Typical usage::

    from auroraview.dcc_mcp import AuroraViewAdapter, start_server

    adapter = AuroraViewAdapter(webview, host_dcc="maya")
    server = start_server(adapter)
    server.start()
    ...
    server.stop()
"""

from __future__ import annotations

import logging
import os
import tempfile
from typing import Any, Dict, Optional

from . import adapter_registry
from ._compat import require_core as _require_core
from .adapter import DCC_NAME

logger = logging.getLogger(__name__)

#: Default gateway port used when none is supplied and none is configured.
DEFAULT_GATEWAY_PORT = 8790


def _builtin_skills_dir():
    """Return a stable directory for core's built-in skills.

    Returns:
        ``pathlib.Path`` to an existing (possibly empty) directory. Core
        exposes this value as ``_builtin_skills_dir`` and calls ``.is_dir()``
        on it, so it must be a ``Path`` rather than a ``str``.
    """
    from pathlib import Path

    base = Path(tempfile.gettempdir()) / "auroraview-dcc-mcp" / "builtin-skills"
    base.mkdir(parents=True, exist_ok=True)
    return base


def start_server(
    adapter: Any,
    *,
    gateway_port: Optional[int] = None,
    registry_dir: Optional[str] = None,
    dcc_version: Optional[str] = None,
    adapter_version: Optional[str] = None,
    display_name: Optional[str] = None,
    skill_paths: Optional[list] = None,
    enable_telemetry: bool = False,
) -> Any:
    """Build a ``DccServerBase`` for an AuroraView adapter.

    The returned server is **not** started; call ``.start()`` on it.

    Args:
        adapter: An :class:`~auroraview.dcc_mcp.adapter.AuroraViewAdapter`.
        gateway_port: Gateway/registry port. Non-zero is required for
            FileRegistry registration. Defaults to ``DCC_MCP_GATEWAY_PORT``,
            then :data:`DEFAULT_GATEWAY_PORT`.
        registry_dir: Registry directory. Defaults to ``DCC_MCP_REGISTRY_DIR``
            or core's default live registry.
        dcc_version: Reported AuroraView version.
        adapter_version: Reported adapter package version.
        display_name: Human-readable instance label shown when several
            instances are running. Defaults to the WebView window title.
        skill_paths: Extra directories to scan for skill packages.
        enable_telemetry: Forwarded to core; off by default.

    Returns:
        A configured ``dcc_mcp_core.server_base.DccServerBase``.

    Raises:
        ImportError: If ``dcc-mcp-core`` is not installed.
        ValueError: If ``gateway_port`` resolves to ``0``, which would silently
            disable discovery.
    """
    _require_core()
    from dcc_mcp_core.server_base import DccServerBase, DccServerOptions

    if gateway_port is None:
        gateway_port = int(os.environ.get("DCC_MCP_GATEWAY_PORT") or DEFAULT_GATEWAY_PORT)
    if gateway_port == 0:
        raise ValueError(
            "gateway_port=0 disables FileRegistry registration, so the DCC-MCP "
            "CLI could never discover this instance. Pass a non-zero port."
        )

    context = adapter.get_context() if hasattr(adapter, "get_context") else {}
    if adapter_version is None:
        adapter_version = _adapter_version()
    if dcc_version is None:
        dcc_version = _auroraview_version()
    if display_name is None:
        display_name = context.get("window_title") or "AuroraView"

    options = DccServerOptions.from_env(
        DCC_NAME,
        builtin_skills_dir=_builtin_skills_dir(),
        gateway_port=gateway_port,
        registry_dir=registry_dir or os.environ.get("DCC_MCP_REGISTRY_DIR"),
        dcc_version=dcc_version,
        dcc_pid=os.getpid(),
        dcc_window_title=context.get("window_title"),
        adapter_version=adapter_version,
        enable_telemetry=enable_telemetry,
    )

    server = DccServerBase(options)

    # Register the adapter so skill scripts can reach it. Without this, skills
    # are discoverable and loadable but every invocation fails.
    adapter_registry.register(adapter)

    # NOTE: we deliberately do NOT call register_inprocess_executor().
    # Core's HostExecutionBridge._dispatch_raw requires a dispatcher exposing
    # `is_host_thread`, `dispatch_callable`, or the post/tick queue API; with
    # none of those it raises TypeError, which is swallowed into an error
    # envelope -- so every tool call would fail with a TypeError instead of
    # running. Leaving the dispatcher unset makes core invoke the callable
    # inline, which is the path that works. Host-thread dispatch for tool
    # execution belongs on the QueueDispatcher (see AuroraViewQtHost).
    #
    # If a dispatcher is ever registered here it must satisfy core's
    # protocol; tests assert that by reading core's own registration state
    # (`server._dcc_dispatcher` / `server._execution_bridge`).

    paths = [p for p in (skill_paths or []) if p]
    if paths:
        server.reload_skill_paths(extra_skill_paths=paths)
    return server


def _auroraview_version() -> str:
    """Return the installed AuroraView version, or ``"unknown"``."""
    try:
        import auroraview

        return str(getattr(auroraview, "__version__", "unknown"))
    except Exception:  # noqa: BLE001 - version reporting must never be fatal
        return "unknown"


def _adapter_version() -> str:
    """Return the AuroraView version as the adapter package version."""
    return _auroraview_version()


def registration_context(adapter: Any) -> Dict[str, Any]:
    """Return the WebView descriptor that describes this instance to the registry.

    This is the adapter's own context (``window_title``, ``url``, ``pid``,
    ``cdp_port``, ``host_dcc``), kept separate from the registry row core owns.

    Args:
        adapter: An :class:`~auroraview.dcc_mcp.adapter.AuroraViewAdapter`.

    Returns:
        Descriptor dict.
    """
    return dict(adapter.get_context())
