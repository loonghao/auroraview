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

The registry holds only weak references. An adapter keeps a reference to its
WebView, so a strong reference here would keep the WebView (and its audit
log) alive for the lifetime of the process -- a real leak for a panel that is
opened and closed repeatedly. Entries disappear on their own once nothing
else holds the adapter.
"""

from __future__ import annotations

import threading
import weakref
from typing import Any, Iterator, List, Optional

_lock = threading.RLock()

#: Insertion-ordered weak references to registered adapters.
_order: "List[weakref.ReferenceType[Any]]" = []


def _deref() -> List[Any]:
    """Return live adapters in registration order, dropping dead references.

    Returns:
        List of live adapters.
    """
    live = []
    dead = False
    for ref in _order:
        obj = ref()
        if obj is None:
            dead = True
        else:
            live.append(obj)
    if dead:
        # Compact so repeated collections do not grow the list forever.
        del _order[:]
        _order.extend(weakref.ref(item) for item in live)
    return live


def register(adapter: Any) -> None:
    """Register a live adapter as available for skill dispatch.

    Args:
        adapter: An :class:`~auroraview.dcc_mcp.adapter.AuroraViewAdapter`.

    Raises:
        ValueError: If ``adapter`` is None.
        TypeError: If ``adapter`` cannot be weakly referenced.
    """
    if adapter is None:
        raise ValueError("cannot register a None adapter")
    ref = weakref.ref(adapter)
    with _lock:
        # Avoid duplicate entries for the same adapter.
        if any(existing is adapter for existing in _deref()):
            return
        _order.append(ref)


def unregister(adapter: Any) -> None:
    """Remove an adapter from the registry.

    Args:
        adapter: The adapter to remove. Unknown adapters are ignored.
    """
    with _lock:
        remaining = [item for item in _deref() if item is not adapter]
        if len(remaining) != len(_order):
            del _order[:]
            _order.extend(weakref.ref(item) for item in remaining)


def adapters() -> List[Any]:
    """Return all currently registered adapters, oldest first.

    Returns:
        A list of live adapters (may be empty).
    """
    with _lock:
        return _deref()


def current() -> Optional[Any]:
    """Return the most recently registered live adapter, or ``None``.

    Returns:
        An adapter, or ``None`` when none is registered.
    """
    with _lock:
        live = _deref()
        return live[-1] if live else None


def clear() -> None:
    """Drop every registered adapter. Intended for tests."""
    with _lock:
        del _order[:]


def iter_adapters() -> Iterator[Any]:
    """Iterate over registered adapters, oldest first."""
    for adapter in adapters():
        yield adapter


#: Public alias: the adapter skill dispatch will use.
current_adapter = current
