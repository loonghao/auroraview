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


class TestEntryPointSignatures:
    """RFC 0017 §3.4: every Python entry point MUST accept ``capture_file_drop``.

    The PR shipping RFC 0017 originally missed several public surfaces
    (``WebView.__init__``, ``WebView.run_embedded``, ``create_webview``,
    ``WebViewFactory.run_embedded``). This guard pins the signature so a
    future refactor cannot silently drop the kwarg again.

    These tests intentionally use ``inspect.signature`` instead of
    constructing instances; instantiating ``WebView`` requires the Rust
    ``_core`` extension, which is unavailable in pure-Python lint passes.
    """

    @staticmethod
    def _accepts(func, name: str) -> bool:
        import inspect

        sig = inspect.signature(func)
        params = sig.parameters
        if name in params:
            return True
        # Tolerate **kwargs only when the callable explicitly documents
        # the kwarg downstream; we still want the *named* parameter at
        # the entry-point boundary so users get IDE completion + type
        # checking. Returning False here is the strict contract.
        return False

    def test_webview_init_accepts_capture_file_drop(self):
        from auroraview.core.webview import WebView

        assert self._accepts(WebView.__init__, "capture_file_drop"), (
            "WebView.__init__ must declare capture_file_drop as a named "
            "parameter so the kwarg reaches _CoreWebView."
        )

    def test_webview_create_accepts_capture_file_drop(self):
        from auroraview.core.mixins.factory import WebViewFactoryMixin

        assert self._accepts(WebViewFactoryMixin.create, "capture_file_drop")

    def test_webview_run_embedded_accepts_capture_file_drop(self):
        from auroraview.core.mixins.factory import WebViewFactoryMixin

        assert self._accepts(WebViewFactoryMixin.run_embedded, "capture_file_drop")

    def test_webviewfactory_create_accepts_capture_file_drop(self):
        from auroraview.core.factory import WebViewFactory

        assert self._accepts(WebViewFactory.create, "capture_file_drop")

    def test_webviewfactory_run_embedded_accepts_capture_file_drop(self):
        from auroraview.core.factory import WebViewFactory

        assert self._accepts(WebViewFactory.run_embedded, "capture_file_drop")

    def test_create_webview_accepts_capture_file_drop(self):
        from auroraview.api import create_webview

        assert self._accepts(create_webview, "capture_file_drop")

    def test_qtwebview_init_accepts_capture_file_drop(self):
        # QtWebView lives behind a Qt import; skip cleanly if Qt isn't
        # available in this CI matrix slice.
        import pytest

        try:
            from auroraview.integration.qt._core import QtWebView
        except ImportError:
            pytest.skip("Qt/PySide not available in this environment")

        assert self._accepts(QtWebView.__init__, "capture_file_drop")


class TestWebViewConfigBranchExtraction:
    """RFC 0017 §4: ``WebView.__init__(config=...)`` must read
    ``capture_file_drop`` from ``WebViewConfig.content`` and forward it to
    the Rust ``_CoreWebView`` constructor unchanged.

    This guards the ``config is not None`` branch in
    ``python/auroraview/core/webview.py`` (the line:

        capture_file_drop = config.content.capture_file_drop

    ) which was previously uncovered because every other test path
    constructed ``WebView`` via flat kwargs instead of the structured
    ``WebViewConfig`` entry point.

    We patch the ``_CoreWebView`` symbol on the module so the test never
    boots a real WebView2 instance — we only need the kwargs the wrapper
    forwards into the Rust binding to make the assertion.
    """

    @staticmethod
    def _build_with_config(capture_value):
        """Construct a ``WebView`` from a ``WebViewConfig`` and return the
        kwargs that ``_CoreWebView`` was called with.

        ``capture_value`` is the tri-state value placed onto
        ``ContentConfig.capture_file_drop`` (``True`` / ``False`` /
        ``None``). The mocked ``_CoreWebView`` raises immediately so we
        never run the rest of ``WebView.__init__`` (window manager,
        ready events, etc.) — we only care about the arguments captured
        by ``MagicMock``.
        """
        from unittest.mock import MagicMock

        import pytest

        import auroraview.core.webview as webview_module
        from auroraview.core.config import WebViewConfig

        cfg = WebViewConfig.from_kwargs(capture_file_drop=capture_value)

        original_core = webview_module._CoreWebView
        mock_core = MagicMock(side_effect=RuntimeError("__stop_init__"))

        try:
            webview_module._CoreWebView = mock_core
            with pytest.raises(RuntimeError, match="__stop_init__"):
                webview_module.WebView(config=cfg)
        finally:
            webview_module._CoreWebView = original_core

        assert mock_core.call_count == 1, (
            "WebView(config=...) should call _CoreWebView exactly once"
        )
        return mock_core.call_args.kwargs

    def test_config_branch_forwards_true(self):
        kwargs = self._build_with_config(True)
        # Critical: this is the line under test
        # (`capture_file_drop = config.content.capture_file_drop`).
        assert kwargs["capture_file_drop"] is True

    def test_config_branch_forwards_false(self):
        kwargs = self._build_with_config(False)
        # Explicit False must NOT be collapsed to None or True by the
        # config-extraction branch.
        assert kwargs["capture_file_drop"] is False

    def test_config_branch_forwards_none(self):
        kwargs = self._build_with_config(None)
        # Tri-state contract: None reaches the Rust binding unchanged so
        # the Rust side can apply its own `unwrap_or(false)` default.
        assert kwargs["capture_file_drop"] is None
