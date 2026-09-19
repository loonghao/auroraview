# -*- coding: utf-8 -*-
"""Optional-dependency handling for the DCC-MCP integration.

Kept in its own module so subpackages can import the guard without creating a
circular import through ``auroraview.dcc_mcp.__init__``.

``dcc-mcp-core`` is optional: AuroraView keeps its "no mandatory third-party
Python dependency" guarantee. When the package is missing, importing
``auroraview.dcc_mcp`` still succeeds and :func:`require_core` raises a clear
:class:`ImportError` at use time.
"""

from __future__ import annotations

from typing import Any

#: ``str`` when ``dcc_mcp_core`` could not be imported, otherwise ``None``.
DCC_MCP_CORE_IMPORT_ERROR = None

try:  # pragma: no cover - depends on environment
    import dcc_mcp_core as _dcc_mcp_core

    HAS_DCC_MCP_CORE = True
except ImportError as _exc:  # pragma: no cover - depends on environment
    _dcc_mcp_core = None  # type: ignore[assignment]
    DCC_MCP_CORE_IMPORT_ERROR = str(_exc)
    HAS_DCC_MCP_CORE = False


def require_core() -> Any:
    """Return the imported ``dcc_mcp_core`` module.

    Returns:
        The ``dcc_mcp_core`` module.

    Raises:
        ImportError: If ``dcc-mcp-core`` is not installed.
    """
    if not HAS_DCC_MCP_CORE:
        raise ImportError(
            "The 'dcc-mcp-core' package is required for AuroraView DCC-MCP "
            "integration. Install it with: pip install auroraview[dcc-mcp]\n"
            f"Original error: {DCC_MCP_CORE_IMPORT_ERROR}"
        )
    return _dcc_mcp_core
