# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""Unit tests for WebView.show() mode dispatch (RFC 0018 §7 / §13.3).

The ``show()`` method short-circuits before any window/server work when the
process is running under one of the packed-mode env switches:

* ``AURORAVIEW_CLI_DUMP=1``    -> dump CLI metadata and return (§13.3)
* ``AURORAVIEW_CLI_INVOKE=...`` -> run one command headlessly and return (§7)
* ``AURORAVIEW_PACKED=1``       -> serve JSON-RPC over stdio

These tests drive the real :meth:`WebViewLifecycleMixin.show` dispatch on a
bare mixin instance so the branch logic itself is exercised, with the packed
module functions patched to observe which path was taken.
"""

from __future__ import annotations

from unittest.mock import patch

from auroraview.core.mixins.lifecycle import WebViewLifecycleMixin


class _Mixin(WebViewLifecycleMixin):
    """Minimal carrier so ``show()`` can be invoked without a real core."""

    def __init__(self) -> None:
        self._core = None
        self._parent = None


def _patch_packed(**flags):
    """Patch the packed-module functions imported inside ``show()``.

    ``flags`` overrides the default (all modes off) return values. Returns a
    dict of started mocks keyed by the function name.
    """
    defaults = {
        "is_cli_dump_mode": False,
        "is_cli_invoke_mode": False,
        "is_packed_mode": False,
    }
    defaults.update(flags)
    patchers = {}
    mocks = {}
    for name, ret in defaults.items():
        p = patch(f"auroraview.core.packed.{name}", return_value=ret)
        mocks[name] = p.start()
        patchers[name] = p
    for action in ("dump_cli_metadata", "invoke_cli_command", "run_api_server"):
        p = patch(f"auroraview.core.packed.{action}")
        mocks[action] = p.start()
        patchers[action] = p
    return patchers, mocks


def _stop(patchers):
    for p in patchers.values():
        p.stop()


class TestShowDispatch:
    """show() routes to the correct headless handler per env mode."""

    def test_cli_dump_mode_dumps_and_returns(self):
        patchers, mocks = _patch_packed(is_cli_dump_mode=True)
        try:
            wv = _Mixin()
            wv.show()
            mocks["dump_cli_metadata"].assert_called_once_with(wv)
            mocks["invoke_cli_command"].assert_not_called()
            mocks["run_api_server"].assert_not_called()
        finally:
            _stop(patchers)

    def test_cli_invoke_mode_invokes_and_returns(self):
        patchers, mocks = _patch_packed(is_cli_invoke_mode=True)
        try:
            wv = _Mixin()
            wv.show()
            mocks["invoke_cli_command"].assert_called_once_with(wv)
            mocks["dump_cli_metadata"].assert_not_called()
            mocks["run_api_server"].assert_not_called()
        finally:
            _stop(patchers)

    def test_packed_mode_runs_api_server(self):
        patchers, mocks = _patch_packed(is_packed_mode=True)
        try:
            wv = _Mixin()
            wv.show()
            mocks["run_api_server"].assert_called_once_with(wv)
            mocks["dump_cli_metadata"].assert_not_called()
            mocks["invoke_cli_command"].assert_not_called()
        finally:
            _stop(patchers)

    def test_cli_dump_takes_priority_over_invoke_and_packed(self):
        """§13.3 dump short-circuits before invoke/packed checks."""
        patchers, mocks = _patch_packed(
            is_cli_dump_mode=True,
            is_cli_invoke_mode=True,
            is_packed_mode=True,
        )
        try:
            wv = _Mixin()
            wv.show()
            mocks["dump_cli_metadata"].assert_called_once_with(wv)
            mocks["invoke_cli_command"].assert_not_called()
            mocks["run_api_server"].assert_not_called()
        finally:
            _stop(patchers)

    def test_cli_invoke_takes_priority_over_packed(self):
        """§7 invoke short-circuits before the packed API-server path."""
        patchers, mocks = _patch_packed(
            is_cli_invoke_mode=True,
            is_packed_mode=True,
        )
        try:
            wv = _Mixin()
            wv.show()
            mocks["invoke_cli_command"].assert_called_once_with(wv)
            mocks["run_api_server"].assert_not_called()
        finally:
            _stop(patchers)
