# -*- coding: utf-8 -*-
"""RFC 0017 §4.1: Python tri-state contract for ``capture_file_drop``.

These tests guard the `Optional[bool]` semantics across the Python config
boundary:

    * ``None`` (omitted)  → preserved as ``None`` for Rust to default
    * ``True`` (explicit) → preserved as ``True``
    * ``False`` (explicit)→ preserved as ``False`` (NOT collapsed to ``None``)

The transport rule under test is documented in
``python/auroraview/core/config.py::ContentConfig`` and the
``WebViewConfig.from_kwargs`` / ``to_kwargs`` round-trip.

Companion guards:

    * ``tests/python/integration/test_capture_file_drop_passthrough.py`` —
      asserts the same value reaches the Rust PyO3 binding intact.
    * ``scripts/ci/check_capture_file_drop_defaults.py`` — static grep
      that forbids ``setdefault`` / ``or False`` patterns in the
      passthrough chain.
"""

from __future__ import annotations

from auroraview.core.config import ContentConfig, WebViewConfig


class TestContentConfigDefault:
    """Default factory must produce ``None``, not ``False``."""

    def test_default_is_none(self):
        cfg = ContentConfig()
        assert cfg.capture_file_drop is None

    def test_explicit_true(self):
        cfg = ContentConfig(capture_file_drop=True)
        assert cfg.capture_file_drop is True

    def test_explicit_false(self):
        # Critical: explicit False must NOT be collapsed to None.
        cfg = ContentConfig(capture_file_drop=False)
        assert cfg.capture_file_drop is False


class TestFromKwargsTristate:
    """``WebViewConfig.from_kwargs`` must preserve all three states."""

    def test_kwarg_omitted_is_none(self):
        cfg = WebViewConfig.from_kwargs()
        assert cfg.content.capture_file_drop is None

    def test_kwarg_explicit_true(self):
        cfg = WebViewConfig.from_kwargs(capture_file_drop=True)
        assert cfg.content.capture_file_drop is True

    def test_kwarg_explicit_false(self):
        cfg = WebViewConfig.from_kwargs(capture_file_drop=False)
        assert cfg.content.capture_file_drop is False

    def test_kwarg_explicit_none(self):
        # Explicitly passing None must be indistinguishable from omitting.
        cfg = WebViewConfig.from_kwargs(capture_file_drop=None)
        assert cfg.content.capture_file_drop is None


class TestToKwargsTristate:
    """``WebViewConfig.to_kwargs`` must NOT flatten ``None`` to ``False``."""

    def test_round_trip_none_stays_none(self):
        cfg = WebViewConfig.from_kwargs()
        kwargs = cfg.to_kwargs()
        # The flatten must happen on the Rust side (unwrap_or(false)),
        # never in `to_kwargs`.
        assert "capture_file_drop" in kwargs
        assert kwargs["capture_file_drop"] is None

    def test_round_trip_true_stays_true(self):
        cfg = WebViewConfig.from_kwargs(capture_file_drop=True)
        assert cfg.to_kwargs()["capture_file_drop"] is True

    def test_round_trip_false_stays_false(self):
        cfg = WebViewConfig.from_kwargs(capture_file_drop=False)
        assert cfg.to_kwargs()["capture_file_drop"] is False
