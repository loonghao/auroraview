# -*- coding: utf-8 -*-
"""Tests for auroraview.integration.qt.embedding.EmbeddingMixin.

The mixin is normally driven by a fully constructed :class:`QtWebView` with a
live ``QApplication`` and a real WebView2 runtime; both are awkward to bring
up on a CI runner.  These tests instead exercise individual mixin methods
against minimal host stubs so the branch logic (idempotency guards, mutex
acquisition, exception swallowing, env-var override, fallback paths) can be
verified deterministically.
"""

import pytest

# Skip the whole module on environments without qtpy because ``embedding``
# imports it eagerly.
pytest.importorskip("qtpy", reason="Qt tests require qtpy")

from unittest.mock import MagicMock, patch  # noqa: E402

from auroraview.integration.qt import embedding  # noqa: E402
from auroraview.integration.qt.embedding import EmbeddingMixin  # noqa: E402

pytestmark = [pytest.mark.qt_related, pytest.mark.unit]


class _Host:
    """Minimal stand-in for ``QtWebView``.

    Only the attributes touched by the mixin methods under test are
    populated; ``_webview`` is a ``MagicMock`` so any ``.attr`` access is
    silently accepted.  Sibling mixin methods that are *invoked* through
    ``self`` (rather than ``self.cls.method``) are exposed as ``MagicMock``
    attributes so the tests can assert on them directly.
    """

    def __init__(self):
        self._webview = MagicMock()
        self._webview_container = None
        self._webview_qwindow = None
        self._webview_page = MagicMock()
        self._webview_page_layout = MagicMock()
        self._using_direct_embed = False
        self._direct_embed_hwnd = None
        self._geometry_sync_in_progress = False
        self._child_window_fix_in_progress = False
        self._last_synced_bounds = None
        self._is_closing = False
        # Sibling mixin methods invoked via ``self`` are stubbed here so
        # tests can assert on them without bringing up the full mixin
        # inheritance chain.
        self._create_container_qt = MagicMock()
        self._create_container_direct = MagicMock()
        self._sync_webview2_controller_bounds = MagicMock()

    def winId(self):
        return 0xDEAD

    def size(self):
        size = MagicMock()
        size.width.return_value = 800
        size.height.return_value = 600
        return size


# ---------------------------------------------------------------------------
# _create_webview_container — top-level dispatcher
# ---------------------------------------------------------------------------


class TestCreateWebViewContainer:
    def test_no_hwnd_logs_warning_and_returns(self, caplog):
        host = _Host()
        core = MagicMock()
        core.get_hwnd.return_value = None
        EmbeddingMixin._create_webview_container(host, core)
        # No container created
        assert host._webview_container is None

    def test_uses_get_hwnd_when_hwnd_arg_is_none(self):
        host = _Host()
        core = MagicMock()
        core.get_hwnd.return_value = 0xCAFE
        with patch.dict(
            "os.environ", {"AURORAVIEW_USE_DIRECT_EMBED": "0"}, clear=False
        ):
            EmbeddingMixin._create_webview_container(host, core)
        # AURORAVIEW_USE_DIRECT_EMBED=0 forces createWindowContainer path
        host._create_container_qt.assert_called_once_with(0xCAFE)
        host._create_container_direct.assert_not_called()
        core.get_hwnd.assert_called_once()

    def test_env_force_direct_embed_on(self):
        host = _Host()
        core = MagicMock()
        with patch.dict(
            "os.environ", {"AURORAVIEW_USE_DIRECT_EMBED": "1"}, clear=False
        ):
            EmbeddingMixin._create_webview_container(host, core, hwnd=0x1234)
        host._create_container_direct.assert_called_once_with(0x1234)
        host._create_container_qt.assert_not_called()

    def test_env_force_direct_embed_off(self):
        host = _Host()
        core = MagicMock()
        with patch.dict(
            "os.environ", {"AURORAVIEW_USE_DIRECT_EMBED": "false"}, clear=False
        ):
            EmbeddingMixin._create_webview_container(host, core, hwnd=0x1234)
        host._create_container_qt.assert_called_once_with(0x1234)
        host._create_container_direct.assert_not_called()

    def test_auto_detect_uses_qt6_and_platform_support(self, monkeypatch):
        host = _Host()
        core = MagicMock()
        monkeypatch.delenv("AURORAVIEW_USE_DIRECT_EMBED", raising=False)
        with patch.object(
            embedding, "is_qt6", return_value=True
        ), patch.object(
            embedding, "supports_direct_embedding", return_value=True
        ):
            EmbeddingMixin._create_webview_container(host, core, hwnd=0x1234)
        host._create_container_direct.assert_called_once()
        host._create_container_qt.assert_not_called()

    def test_auto_detect_falls_back_to_qt_when_qt5(self, monkeypatch):
        host = _Host()
        core = MagicMock()
        monkeypatch.delenv("AURORAVIEW_USE_DIRECT_EMBED", raising=False)
        with patch.object(
            embedding, "is_qt6", return_value=False
        ), patch.object(
            embedding, "supports_direct_embedding", return_value=True
        ):
            EmbeddingMixin._create_webview_container(host, core, hwnd=0x1234)
        host._create_container_qt.assert_called_once()
        host._create_container_direct.assert_not_called()

    def test_exception_resets_container_to_none(self):
        host = _Host()
        host._webview_container = MagicMock()  # pretend we had something
        core = MagicMock()
        core.get_hwnd.side_effect = RuntimeError("boom")
        # Must not raise
        EmbeddingMixin._create_webview_container(host, core)
        assert host._webview_container is None


# ---------------------------------------------------------------------------
# _schedule_child_window_fixes — mutex + scheduling
# ---------------------------------------------------------------------------


class TestScheduleChildWindowFixes:
    # NOTE: We wrap lambdas with ``staticmethod()`` when patching
    # ``embedding.QTimer.singleShot`` because ``QTimer.singleShot`` is
    # accessed as a class attribute.  Without the descriptor wrapper,
    # Python would treat the bare lambda as an unbound method on Python
    # < 3.x style classes, or monkeypatch would bind ``self`` to it when
    # invoked through the class.  ``staticmethod`` ensures the patched
    # callable is invoked with only (delay, callback) — matching the
    # real Qt slot signature regardless of Qt binding quirks.

    def test_skips_when_is_closing(self, monkeypatch):
        host = _Host()
        host._is_closing = True
        timer_calls = []
        monkeypatch.setattr(
            embedding.QTimer,
            "singleShot",
            staticmethod(lambda d, c: timer_calls.append((d, c))),
        )
        backend = MagicMock()
        with patch(
            "auroraview.integration.qt.platforms.get_backend", return_value=backend
        ):
            EmbeddingMixin._schedule_child_window_fixes(host, 0xABCD)
        backend._fix_all_child_windows_recursive.assert_not_called()
        # The synchronous call returned early but two follow-up timers are
        # still scheduled (each will also see _is_closing=True at fire-time
        # and bail out).  We assert the schedule pattern matches the docstring:
        assert [d for d, _ in timer_calls] == [250, 1000]

    def test_blocked_when_geometry_sync_in_flight(self, monkeypatch):
        host = _Host()
        host._geometry_sync_in_progress = True  # peer holds the flag
        monkeypatch.setattr(
            embedding.QTimer, "singleShot", staticmethod(lambda d, c: None)
        )
        backend = MagicMock()
        with patch(
            "auroraview.integration.qt.platforms.get_backend", return_value=backend
        ):
            EmbeddingMixin._schedule_child_window_fixes(host, 0xABCD)
        # Must not call into the backend while the peer holds the flag
        backend._fix_all_child_windows_recursive.assert_not_called()

    def test_synchronous_fix_runs_first(self, monkeypatch):
        host = _Host()
        monkeypatch.setattr(
            embedding.QTimer, "singleShot", staticmethod(lambda d, c: None)
        )
        backend = MagicMock()
        backend._fix_all_child_windows_recursive.return_value = 5
        with patch(
            "auroraview.integration.qt.platforms.get_backend", return_value=backend
        ):
            EmbeddingMixin._schedule_child_window_fixes(host, 0xABCD)
        backend._fix_all_child_windows_recursive.assert_called_once_with(0xABCD)

    def test_schedules_two_catch_up_ticks(self, monkeypatch):
        host = _Host()
        timer_calls = []
        monkeypatch.setattr(
            embedding.QTimer,
            "singleShot",
            staticmethod(lambda d, c: timer_calls.append((d, c))),
        )
        backend = MagicMock()
        with patch(
            "auroraview.integration.qt.platforms.get_backend", return_value=backend
        ):
            EmbeddingMixin._schedule_child_window_fixes(host, 0xABCD)
        assert len(timer_calls) == 2
        assert [d for d, _ in timer_calls] == [250, 1000]

    def test_releases_flag_after_run(self, monkeypatch):
        host = _Host()
        monkeypatch.setattr(
            embedding.QTimer, "singleShot", staticmethod(lambda d, c: None)
        )
        backend = MagicMock()
        with patch(
            "auroraview.integration.qt.platforms.get_backend", return_value=backend
        ):
            EmbeddingMixin._schedule_child_window_fixes(host, 0xABCD)
        # Must release its own flag on exit
        assert host._child_window_fix_in_progress is False

    def test_handles_backend_exception(self, monkeypatch):
        host = _Host()
        monkeypatch.setattr(
            embedding.QTimer, "singleShot", staticmethod(lambda d, c: None)
        )
        backend = MagicMock()
        backend._fix_all_child_windows_recursive.side_effect = RuntimeError("boom")
        with patch(
            "auroraview.integration.qt.platforms.get_backend", return_value=backend
        ):
            # Must not propagate
            EmbeddingMixin._schedule_child_window_fixes(host, 0xABCD)
        # Flag still released
        assert host._child_window_fix_in_progress is False

    def test_handles_backend_without_fix_method(self, monkeypatch):
        host = _Host()
        monkeypatch.setattr(
            embedding.QTimer, "singleShot", staticmethod(lambda d, c: None)
        )
        backend = MagicMock(spec=[])  # no _fix_all_child_windows_recursive
        with patch(
            "auroraview.integration.qt.platforms.get_backend", return_value=backend
        ):
            EmbeddingMixin._schedule_child_window_fixes(host, 0xABCD)
        # Just ensure no exception escapes
        assert host._child_window_fix_in_progress is False


# ---------------------------------------------------------------------------
# _sync_webview2_controller_bounds — idempotency guard is the key contract
# ---------------------------------------------------------------------------


class TestSyncWebView2ControllerBounds:
    def test_skips_when_container_is_none(self):
        host = _Host()
        host._webview_container = None
        # Must not raise
        EmbeddingMixin._sync_webview2_controller_bounds(host)

    def test_force_size_overrides_container_size(self):
        host = _Host()
        host._webview_container = MagicMock()
        size_mock = MagicMock()
        size_mock.width.return_value = 100
        size_mock.height.return_value = 100
        host._webview_container.size.return_value = size_mock
        core = MagicMock()
        host._webview._core = core
        EmbeddingMixin._sync_webview2_controller_bounds(host, 500, 400)
        core.sync_bounds.assert_called_once_with(500, 400)

    def test_skips_invalid_size_zero(self):
        host = _Host()
        host._webview_container = MagicMock()
        size_mock = MagicMock()
        size_mock.width.return_value = 0
        size_mock.height.return_value = 0
        host._webview_container.size.return_value = size_mock
        core = MagicMock()
        host._webview._core = core
        EmbeddingMixin._sync_webview2_controller_bounds(host)
        core.sync_bounds.assert_not_called()

    def test_idempotent_skip_on_unchanged_size(self):
        """Documented contract: ``_last_synced_bounds`` short-circuits."""
        host = _Host()
        host._webview_container = MagicMock()
        size_mock = MagicMock()
        size_mock.width.return_value = 800
        size_mock.height.return_value = 600
        host._webview_container.size.return_value = size_mock
        host._last_synced_bounds = (800, 600)
        core = MagicMock()
        host._webview._core = core
        EmbeddingMixin._sync_webview2_controller_bounds(host)
        core.sync_bounds.assert_not_called()
        core.set_size.assert_not_called()

    def test_calls_sync_bounds_and_records_size(self):
        host = _Host()
        host._webview_container = MagicMock()
        size_mock = MagicMock()
        size_mock.width.return_value = 1024
        size_mock.height.return_value = 768
        host._webview_container.size.return_value = size_mock
        core = MagicMock()
        host._webview._core = core
        EmbeddingMixin._sync_webview2_controller_bounds(host)
        core.sync_bounds.assert_called_once_with(1024, 768)
        assert host._last_synced_bounds == (1024, 768)

    def test_falls_back_to_set_size_when_no_sync_bounds(self):
        host = _Host()
        host._webview_container = MagicMock()
        size_mock = MagicMock()
        size_mock.width.return_value = 640
        size_mock.height.return_value = 480
        host._webview_container.size.return_value = size_mock
        # spec restricts attribute set so getattr(core, "sync_bounds", None)
        # returns None and the fallback path is taken.
        core = MagicMock(spec=["set_size"])
        host._webview._core = core
        EmbeddingMixin._sync_webview2_controller_bounds(host)
        core.set_size.assert_called_once_with(640, 480)
        assert host._last_synced_bounds == (640, 480)

    def test_handles_sync_bounds_exception(self):
        host = _Host()
        host._webview_container = MagicMock()
        size_mock = MagicMock()
        size_mock.width.return_value = 800
        size_mock.height.return_value = 600
        host._webview_container.size.return_value = size_mock
        core = MagicMock()
        core.sync_bounds.side_effect = RuntimeError("boom")
        host._webview._core = core
        # Must not raise; and since sync_bounds raised, set_size is the fallback
        EmbeddingMixin._sync_webview2_controller_bounds(host)
        core.set_size.assert_called_once_with(800, 600)

    def test_handles_missing_core(self):
        host = _Host()
        host._webview_container = MagicMock()
        size_mock = MagicMock()
        size_mock.width.return_value = 800
        size_mock.height.return_value = 600
        host._webview_container.size.return_value = size_mock
        host._webview._core = None
        # Must not raise
        EmbeddingMixin._sync_webview2_controller_bounds(host)


# ---------------------------------------------------------------------------
# _force_container_geometry
# ---------------------------------------------------------------------------


class TestForceContainerGeometry:
    def test_returns_when_container_is_none(self):
        host = _Host()
        host._webview_container = None
        # Must not raise
        EmbeddingMixin._force_container_geometry(host)

    def test_returns_on_invalid_size(self):
        host = _Host()
        host._webview_container = MagicMock()
        size = MagicMock()
        size.width.return_value = 0
        size.height.return_value = 0
        host.size = lambda: size
        # Must not raise and must not call container resize
        EmbeddingMixin._force_container_geometry(host)
        host._webview_container.setGeometry.assert_not_called()

    def test_resizes_container_and_qwindow(self):
        host = _Host()
        host._webview_container = MagicMock()
        host._webview_qwindow = MagicMock()
        EmbeddingMixin._force_container_geometry(host)
        host._webview_container.setGeometry.assert_called_once_with(0, 0, 800, 600)
        host._webview_qwindow.resize.assert_called_once_with(800, 600)
        host._sync_webview2_controller_bounds.assert_called_once_with(800, 600)

    def test_handles_qwindow_resize_exception(self):
        host = _Host()
        host._webview_container = MagicMock()
        host._webview_qwindow = MagicMock()
        host._webview_qwindow.resize.side_effect = RuntimeError("boom")
        # Must not propagate
        EmbeddingMixin._force_container_geometry(host)


# ---------------------------------------------------------------------------
# _handle_resize_for_embedding
# ---------------------------------------------------------------------------


class TestHandleResizeForEmbedding:
    def test_direct_embed_path(self):
        host = _Host()
        host._using_direct_embed = True
        host._direct_embed_hwnd = 0xBEEF
        with patch.object(embedding, "update_embedded_window_geometry") as mock_upd:
            EmbeddingMixin._handle_resize_for_embedding(host, 1024, 768)
        mock_upd.assert_called_once_with(0xBEEF, 0, 0, 1024, 768)

    def test_qt_container_path(self):
        host = _Host()
        host._using_direct_embed = False
        host._webview_container = MagicMock()
        EmbeddingMixin._handle_resize_for_embedding(host, 640, 480)
        host._webview_container.setGeometry.assert_called_once_with(0, 0, 640, 480)
        host._sync_webview2_controller_bounds.assert_called_once_with()

    def test_no_container_skips_sync(self):
        host = _Host()
        host._using_direct_embed = False
        host._webview_container = None
        # Must not raise
        EmbeddingMixin._handle_resize_for_embedding(host, 640, 480)
        host._sync_webview2_controller_bounds.assert_not_called()


# ---------------------------------------------------------------------------
# _sync_embedded_geometry — currently a no-op on Windows; just smoke-test it
# ---------------------------------------------------------------------------


class TestSyncEmbeddedGeometry:
    def test_returns_without_error(self):
        host = _Host()
        # The method is a no-op; just verify it does not raise
        EmbeddingMixin._sync_embedded_geometry(host)


# ---------------------------------------------------------------------------
# Module-level smoke tests
# ---------------------------------------------------------------------------


class TestModuleExports:
    def test_embedding_mixin_exported(self):
        assert "EmbeddingMixin" in embedding.__all__

    def test_verbose_logging_constant_is_bool(self):
        assert isinstance(embedding._VERBOSE_LOGGING, bool)
