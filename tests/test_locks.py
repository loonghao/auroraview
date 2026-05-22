# -*- coding: utf-8 -*-
"""Tests for auroraview.integration.qt._locks."""

import sys
from pathlib import Path
from types import SimpleNamespace

# Allow importing auroraview from the python/ directory
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "python"))

import pytest

from auroraview.integration.qt._locks import (
    acquire_exclusive,
    acquire_flag,
    release_flag,
    try_acquire_flag,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _host():
    """Return a fresh object that can carry arbitrary attributes."""
    return SimpleNamespace()


# ---------------------------------------------------------------------------
# acquire_flag
# ---------------------------------------------------------------------------


def test_acquire_flag_basic():
    """Yields True on first acquire, then resets on context exit."""
    h = _host()
    with acquire_flag(h, "_lock") as got:
        assert got is True
        assert h._lock is True
    assert h._lock is False


def test_acquire_flag_reentrant():
    """Second acquire on the same flag yields False (caller must bail out)."""
    h = _host()
    with acquire_flag(h, "_lock") as got1:
        assert got1 is True
        with acquire_flag(h, "_lock") as got2:
            assert got2 is False
        # still True because outer context owns it
        assert h._lock is True
    assert h._lock is False


def test_acquire_flag_multiple_names():
    """Different flag names are independent."""
    h = _host()
    with acquire_flag(h, "_a") as got_a:
        assert got_a is True
        with acquire_flag(h, "_b") as got_b:
            assert got_b is True
        assert h._b is False
    assert h._a is False


# ---------------------------------------------------------------------------
# acquire_exclusive
# ---------------------------------------------------------------------------


def test_acquire_exclusive_basic():
    """Own flag is acquired when no peer is in flight."""
    h = _host()
    with acquire_exclusive(h, "_own", "_peer") as got:
        assert got is True
        assert h._own is True
    assert h._own is False


def test_acquire_exclusive_peer_blocks():
    """Yielding False when a peer flag is already set."""
    h = _host()
    h._peer = True
    with acquire_exclusive(h, "_own", "_peer") as got:
        assert got is False
    # own flag must not have been set
    assert getattr(h, "_own", False) is False


def test_acquire_exclusive_own_already_set():
    """Yielding False when own flag is already set (reentrant guard)."""
    h = _host()
    h._own = True
    with acquire_exclusive(h, "_own", "_peer") as got:
        assert got is False


def test_acquire_exclusive_multiple_peers():
    """Any peer being set blocks acquisition."""
    h = _host()
    h._p2 = True
    with acquire_exclusive(h, "_own", "_p1", "_p2", "_p3") as got:
        assert got is False


def test_acquire_exclusive_no_peers():
    """acquire_exclusive with only own flag (no peers) behaves like acquire_flag."""
    h = _host()
    with acquire_exclusive(h, "_own") as got:
        assert got is True
        assert h._own is True
    assert h._own is False


# ---------------------------------------------------------------------------
# try_acquire_flag / release_flag
# ---------------------------------------------------------------------------


def test_try_acquire_flag_success():
    h = _host()
    assert try_acquire_flag(h, "_lock") is True
    assert h._lock is True
    release_flag(h, "_lock")
    assert h._lock is False


def test_try_acquire_flag_already_set():
    h = _host()
    assert try_acquire_flag(h, "_lock") is True
    assert try_acquire_flag(h, "_lock") is False
    release_flag(h, "_lock")


def test_release_flag_assertion_without_acquire():
    """release_flag without a matching acquire triggers an assertion."""
    h = _host()
    h._lock = True  # simulate out-of-band set to pass the assert condition
    # The assert checks that the flag is True; if we set it to True manually
    # the assert passes but the semantics are wrong.  We test the *intended*
    # failure mode by calling release_flag when the flag is False.
    h._lock = False
    with pytest.raises(AssertionError):
        release_flag(h, "_lock")
