# -*- coding: utf-8 -*-
"""Host DCC detection for the DCC-MCP integration.

When an AuroraView panel is embedded in a DCC (Maya, Houdini, ...), the
registry row must say *which* DCC session it belongs to. ``dcc-mcp-core``
provides exactly one field for that: ``host_dcc`` on ``WebViewContext``,
documented as "the DCC embedding this WebView, if any".

Without it, a panel running inside Maya registers as a bare
``dcc_type=auroraview`` row, and a downstream agent cannot tell that the row
belongs to the Maya session -- which is what makes coexistence with a real
``dcc-mcp-maya`` adapter ambiguous.

Detection mirrors the environment-variable contract already used by
``DccType::detect()`` in ``crates/auroraview-dcc/src/config.rs``, so Python and
Rust agree on what counts as "running inside Maya". It is implemented in pure
Python so it works on 3.7 and does not depend on the compiled core exporting
``DccType``.

Names are lower-case DCC-MCP style identifiers (``maya``, ``houdini``, ...)
matching the ``dcc:`` values used in the DCC-MCP catalog, not the display
names used by ``DccType::name()`` (``"Maya"``).
"""

from __future__ import annotations

import os
from typing import Dict, Optional, Tuple

#: Environment variable -> DCC-MCP style host name.
#:
#: Order matters: the first match wins. This list intentionally mirrors
#: ``DccType::detect()`` so both implementations agree.
_HOST_ENV_VARS: Tuple[Tuple[str, str], ...] = (
    ("MAYA_LOCATION", "maya"),
    ("HFS", "houdini"),
    ("NUKE_PATH", "nuke"),
    ("BLENDER_SYSTEM_SCRIPTS", "blender"),
    ("3DSMAX_LOCATION", "3dsmax"),
    ("ADSK_3DSMAX_X64_2025", "3dsmax"),
    ("UE_ROOT", "unreal"),
    ("UE4_ROOT", "unreal"),
)

#: Environment variable used to force a host name, overriding detection.
#: Useful in tests and in studio launchers where several DCCs are on PATH.
HOST_DCC_ENV_VAR = "AURORAVIEW_HOST_DCC"

#: Value stored when no host DCC can be determined.
UNKNOWN_HOST = "unknown"


def detect_host_dcc(env: Optional[Dict[str, str]] = None) -> Optional[str]:
    """Return the DCC-MCP style name of the embedding DCC, or ``None``.

    An explicit :data:`AURORAVIEW_HOST_DCC` override wins over detection. When
    the override is set to an empty string, detection is skipped entirely and
    ``None`` is returned, so a caller can force "standalone".

    Args:
        env: Environment mapping to inspect. Defaults to ``os.environ``.

    Returns:
        Lower-case host name (e.g. ``"maya"``), or ``None`` when the WebView is
        not running inside a recognised DCC.
    """
    source = os.environ if env is None else env

    override = source.get(HOST_DCC_ENV_VAR)
    if override is not None:
        override = override.strip()
        if not override:
            return None
        return override.lower()

    for var, name in _HOST_ENV_VARS:
        if source.get(var):
            return name
    return None
