# -*- coding: utf-8 -*-
"""Tests for auroraview.integration.qt._locks.

These cross-task mutex helpers are pure Python and do not depend on Qt,
so the tests run on every platform without ``qtpy`` installed.
"""

import pytest

from auroraview.integration.qt._locks import (
    FLAG_CHILD_WINDOW_FIX,
    FLAG_GEOMETRY_SYNC,
    acquire_exclusive,
    acquire_flag,
    release_flag,
    try_acquire_flag,
)

pytestmark = [pytest.mark.unit]


class _Host:
    """Minimal stand-in for a host object used to back getattr/setattr."""


class TestAcquireFlag:
    """Tests for the ``acquire_flag`` context manager."""

    def test_acquires_when_unset(self):
        host = _Host()
        with acquire_flag(host, "_busy") as got:
            assert got is True
            assert host._busy is True

    def test_releases_after_normal_exit(self):
        host = _Host()
        with acquire_flag(host, "_busy"):
            pass
        assert host._busy is False

    def test_releases_after_exception(self):
        host = _Host()
        with pytest.raises(RuntimeError):
            with acquire_flag(host, "_busy"):
                assert host._busy is True
                raise RuntimeError("boom")
        # finally branch must still flip the flag back
        assert host._busy is False

    def test_yields_false_when_already_held(self):
        host = _Host()
        host._busy = True
        with acquire_flag(host, "_busy") as got:
            assert got is False

    def test_does_not_release_peer_when_yields_false(self):
        """A blocked acquire must not flip the existing holder's flag."""
        host = _Host()
        host._busy = True
        with acquire_flag(host, "_busy") as got:
            assert got is False
        # original holder still owns the flag
        assert host._busy is True

    def test_uses_default_false_for_missing_attr(self):
        """A host without the attribute behaves as if the flag was unset."""
        host = _Host()
        assert not hasattr(host, "_missing")
        with acquire_flag(host, "_missing") as got:
            assert got is True
        assert host._missing is False

    def test_nested_acquire_same_flag_is_blocked(self):
        host = _Host()
        with acquire_flag(host, "_busy") as outer_got:
            assert outer_got is True
            with acquire_flag(host, "_busy") as inner_got:
                assert inner_got is False
            # outer is still owner
            assert host._busy is True
        assert host._busy is False


class TestAcquireExclusive:
    """Tests for ``acquire_exclusive`` two-task mutual exclusion."""

    def test_acquires_when_all_free(self):
        host = _Host()
        with acquire_exclusive(host, "_own", "_peer") as got:
            assert got is True
            assert host._own is True

    def test_blocked_by_peer_in_flight(self):
        host = _Host()
        host._peer = True
        with acquire_exclusive(host, "_own", "_peer") as got:
            assert got is False
        # own flag must not be set when blocked
        assert getattr(host, "_own", False) is False

    def test_blocked_by_own_already_held(self):
        host = _Host()
        host._own = True
        with acquire_exclusive(host, "_own", "_peer") as got:
            assert got is False

    def test_does_not_touch_peers_on_block(self):
        host = _Host()
        host._peer = True
        with acquire_exclusive(host, "_own", "_peer") as got:
            assert got is False
        assert host._peer is True  # untouched

    def test_releases_own_after_normal_exit(self):
        host = _Host()
        with acquire_exclusive(host, "_own", "_peer"):
            pass
        assert host._own is False

    def test_releases_own_after_exception(self):
        host = _Host()
        with pytest.raises(ValueError):
            with acquire_exclusive(host, "_own", "_peer"):
                raise ValueError("boom")
        assert host._own is False

    def test_no_peers_behaves_like_acquire_flag(self):
        host = _Host()
        with acquire_exclusive(host, "_own") as got:
            assert got is True
        assert host._own is False

    def test_multiple_peers_any_blocks(self):
        host = _Host()
        host._peer_a = False
        host._peer_b = True
        with acquire_exclusive(host, "_own", "_peer_a", "_peer_b") as got:
            assert got is False

    def test_multiple_peers_all_free_acquires(self):
        host = _Host()
        host._peer_a = False
        host._peer_b = False
        with acquire_exclusive(host, "_own", "_peer_a", "_peer_b") as got:
            assert got is True


class TestTryAcquireRelease:
    """Tests for the raw ``try_acquire_flag`` / ``release_flag`` primitives."""

    def test_try_acquire_returns_true_first_time(self):
        host = _Host()
        assert try_acquire_flag(host, "_busy") is True
        assert host._busy is True

    def test_try_acquire_returns_false_when_held(self):
        host = _Host()
        assert try_acquire_flag(host, "_busy") is True
        assert try_acquire_flag(host, "_busy") is False

    def test_release_clears_flag(self):
        host = _Host()
        try_acquire_flag(host, "_busy")
        release_flag(host, "_busy")
        assert host._busy is False
        # subsequent acquire works again
        assert try_acquire_flag(host, "_busy") is True

    def test_release_raises_when_not_held(self):
        host = _Host()
        with pytest.raises(RuntimeError):
            release_flag(host, "_busy")

    def test_release_error_message_includes_name(self):
        host = _Host()
        with pytest.raises(RuntimeError) as exc_info:
            release_flag(host, "_geometry_sync_in_progress")
        assert "_geometry_sync_in_progress" in str(exc_info.value)

    def test_release_after_external_clear_also_raises(self):
        host = _Host()
        try_acquire_flag(host, "_busy")
        host._busy = False  # external interference
        with pytest.raises(RuntimeError):
            release_flag(host, "_busy")


class TestConstants:
    """Smoke tests for the canonical flag-name constants."""

    def test_flag_constants_are_strings(self):
        assert isinstance(FLAG_GEOMETRY_SYNC, str)
        assert isinstance(FLAG_CHILD_WINDOW_FIX, str)

    def test_constants_are_distinct(self):
        assert FLAG_GEOMETRY_SYNC != FLAG_CHILD_WINDOW_FIX


class TestModuleExports:
    """Tests for the ``__all__`` export list."""

    def test_all_names_exported(self):
        from auroraview.integration.qt import _locks

        for name in (
            "acquire_flag",
            "acquire_exclusive",
            "try_acquire_flag",
            "release_flag",
            "FLAG_GEOMETRY_SYNC",
            "FLAG_CHILD_WINDOW_FIX",
        ):
            assert name in _locks.__all__
            assert hasattr(_locks, name)
