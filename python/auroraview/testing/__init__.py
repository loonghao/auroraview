"""AuroraView Testing Framework.

AI-friendly testing and inspection for AuroraView WebView applications.

Quick Start:
    >>> from auroraview.testing import Inspector
    >>> inspector = Inspector.connect("http://localhost:9222")
    >>> print(inspector.snapshot())
    >>> inspector.click("@3")
    >>> inspector.close()

Or with context manager:
    >>> with Inspector.connect("http://localhost:9222") as page:
    ...     print(page.snapshot())
    ...     page.click("@3")
"""

from __future__ import annotations

# Try to import from Rust extension first
_RUST_BACKEND = False

try:
    from auroraview._core.testing import (
        Inspector,
        Snapshot,
        RefInfo,
        ActionResult,
    )

    _RUST_BACKEND = True
except ImportError:
    # Fall back to pure Python implementation
    from auroraview.testing.inspector import (
        Inspector,
        Snapshot,
        RefInfo,
        ActionResult,
    )

__all__ = [
    "Inspector",
    "Snapshot",
    "RefInfo",
    "ActionResult",
    "_RUST_BACKEND",
]
