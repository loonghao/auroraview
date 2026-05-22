# -*- coding: utf-8 -*-
"""Lightweight cross-task mutex flags for Qt integration.

These helpers exist to coordinate work that touches the same Win32 / WebView2
resources from multiple QTimer callbacks (notably the WebView2 child-window
fixer in :mod:`embedding` and the delayed geometry sync in :mod:`lifecycle`).

Qt timers do not preempt each other on the GUI thread, but nested event
dispatch (``QApplication.processEvents`` / modal dialogs / blocking COM
calls) can re-enter another pending callback while the first one is still
executing.  Without these guards, a ``SetWindowPos`` cascade combined with
``ICoreWebView2Controller::put_Bounds`` has been observed to deadlock the
DCC main thread (Maya freezes after the second ``Delayed geometry sync
completed`` line).

Reentrancy model
----------------

Although ``getattr`` and ``setattr`` are two separate bytecode operations,
the only way control can transfer to another Qt timer callback is through
a function call that itself dispatches the event loop (``processEvents`` /
modal dialog / blocking COM call).  Neither ``getattr`` nor ``setattr``
does that, so on the GUI thread the check-and-set pair inside
:func:`acquire_flag` is effectively atomic against Qt timer reentrancy.

Usage
-----

Single-flag mutual exclusion::

    from auroraview.integration.qt._locks import acquire_flag

    with acquire_flag(self, "_geometry_sync_in_progress") as got:
        if not got:
            return
        do_work()

Two-flag exclusion (this task and a peer task must not overlap)::

    from auroraview.integration.qt._locks import acquire_exclusive

    with acquire_exclusive(
        self,
        "_geometry_sync_in_progress",
        "_child_window_fix_in_progress",
    ) as got:
        if not got:
            return  # caller may reschedule, see lifecycle.py
        do_work()

The contextmanager forms make acquire/release pairing impossible to get
wrong; prefer them over the raw :func:`try_acquire_flag` /
:func:`release_flag` primitives below.
"""

from contextlib import contextmanager
from typing import Any, Iterator


@contextmanager
def acquire_flag(host: Any, name: str) -> Iterator[bool]:
    """Context manager for a boolean guard attribute on ``host``.

    Yields ``True`` if the flag was previously unset (the caller now owns
    the lock; it will be released automatically on context exit).  Yields
    ``False`` if another caller already owns the lock; the caller must
    bail out without doing the guarded work.

    Pairing acquire/release through ``with`` removes the possibility of
    forgetting a ``release`` in an early-return branch.
    """
    if getattr(host, name, False):
        yield False
        return
    setattr(host, name, True)
    try:
        yield True
    finally:
        setattr(host, name, False)


@contextmanager
def acquire_exclusive(host: Any, own_flag: str, *peer_flags: str) -> Iterator[bool]:
    """Acquire ``own_flag`` only if all ``peer_flags`` are currently free.

    This generalises the "two-task mutual exclusion" pattern used between
    the WebView2 child-window fixer and the delayed geometry sync in DCC
    hosts: if any peer is currently in flight (or this task itself is
    already running), yield ``False`` so the caller can bail out / reschedule;
    otherwise acquire ``own_flag`` and release it on context exit.

    Yields ``True`` only when the caller now owns ``own_flag`` and no
    peer is in flight.  Yields ``False`` either when a peer holds its
    flag or when ``own_flag`` was already set; callers cannot
    distinguish the two reasons (and intentionally do not need to).
    """
    for peer in peer_flags:
        if getattr(host, peer, False):
            yield False
            return
    if getattr(host, own_flag, False):
        yield False
        return
    setattr(host, own_flag, True)
    try:
        yield True
    finally:
        setattr(host, own_flag, False)


def try_acquire_flag(host: Any, name: str) -> bool:
    """Check-and-set a boolean guard attribute on ``host``.

    Returns ``True`` if the flag was previously unset (the caller now owns
    the lock and must release it via :func:`release_flag`).  Returns
    ``False`` if another caller already owns the lock; the caller must
    bail out without doing the guarded work.

    Prefer :func:`acquire_flag` for new code; this primitive is kept for
    cases where ``with`` is awkward (e.g. acquiring in one function and
    releasing in another).
    """
    if getattr(host, name, False):
        return False
    setattr(host, name, True)
    return True


def release_flag(host: Any, name: str) -> None:
    """Release a guard previously acquired via :func:`try_acquire_flag`.

    The ``assert`` below is a debug-only safety net and is stripped under
    ``python -O`` (or when bytecode optimisers like PyOxidizer drop
    asserts).  For production-grade pairing safety, prefer the
    :func:`acquire_flag` / :func:`acquire_exclusive` context managers
    which use ``with`` and cannot be mismatched.
    """
    assert getattr(host, name, False), (
        f"release_flag called without a matching try_acquire_flag: {name}"
    )
    setattr(host, name, False)


__all__ = [
    "acquire_flag",
    "acquire_exclusive",
    "try_acquire_flag",
    "release_flag",
]
