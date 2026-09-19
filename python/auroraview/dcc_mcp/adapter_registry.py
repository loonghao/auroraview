# -*- coding: utf-8 -*-
"""Process-wide registry of live AuroraView adapters.

Skill scripts need a way to reach the adapter that owns the WebView. The
scripts run under whatever interpreter the host uses, so the lookup has to
work without any configuration: this module keeps a table of adapters that
:func:`~auroraview.dcc_mcp.server.start_server` populates, and the skill
dispatcher reads from.

This closes the loop that makes tool *invocation* possible:

    start_server(adapter)  ->  registry.register(adapter)
    skill script           ->  registry.current()  ->  adapter.execute(...)

Without it a skill can be discovered and loaded but every call fails, which
looks healthy right up until an agent actually tries to use a tool.

The registry stores weak references so a dropped adapter cannot keep a
WebView alive past its lifetime.
"""

from __future__ import annotations

import threading
import weakref
from typing import Any, Iterator, List, Optional

_lock = threading.RLock()

#: Live adapters, most recently registered last.
_adapters: "weakref.WeakSet[Any]" = weakref.WeakSet()
_order: List[Any] = []


def register(adapter: Any) -> None:
    """Register a live adapter as available for skill dispatch.

    Args:
        adapter: An :class:`~auroraview.dcc_mcp.adapter.AuroraViewAdapter`.
    """
    if adapter is None:
        raise ValueError("cannot register a None adapter")
    with _lock:
        _adapters.add(adapter)
        # Keep an insertion-ordered view. WeakSet has no ordering, so track
        # strong refs in a list and prune dead ones lazily.
        _order.append(adapter)


def unregister(adapter: Any) -> None:
    """Remove an adapter from the registry.

    Args:
        adapter: The adapter to remove. Unknown adapters are ignored.
    """
    with _lock:
        _adapters.discard(adapter)
        _prune()


def adapters() -> List[Any]:
    """Return all currently registered adapters, oldest first.

    Returns:
        A list of live adapters (may be empty).
    """
    with _lock:
        _prune()
        return list(_order)


def current() -> Optional[Any]:
    """Return the most recently registered live adapter, or ``None``.

    Returns:
        An adapter, or ``None`` when none is registered.
    """
    with _lock:
        _prune()
        return _order[-1] if _order else None


def clear() -> None:
    """Drop every registered adapter. Intended for tests."""
    with _lock:
        _adapters.clear()
        del _order[:]


def _prune() -> None:
    """Drop entries whose adapter has been garbage collected.

    Caller must hold ``_lock``.
    """
    alive = [item for item in _order if item in _adapters]
    if len(alive) != len(_order):
        del _order[:]
        _order.extend(alive)


def iter_adapters() -> Iterator[Any]:
    """Iterate over registered adapters, oldest first."""
    for adapter in adapters():
        yield adapter


#: Public alias: the adapter skill dispatch will use.
current_adapter = current
