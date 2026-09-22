# -*- coding: utf-8 -*-
"""Tests for the auroraview.diagnostics() runtime diagnostics surface."""

import importlib
import importlib.util
import json
import sys

import pytest

import auroraview
from auroraview import diagnostics, format_diagnostics
from auroraview.utils.thread_dispatcher import (
    ThreadDispatcherBackend,
    clear_dispatcher_backends,
    register_dispatcher_backend,
)

# `auroraview.diagnostics` resolves to the re-exported function, so reach the
# module itself through importlib when a probe needs patching.
_DIAGNOSTICS_MODULE = importlib.import_module("auroraview.diagnostics")

REQUIRED_SECTIONS = (
    "auroraview",
    "platform",
    "python",
    "host",
    "dispatcher",
    "thread",
    "qt",
    "errors",
)


@pytest.fixture(autouse=True)
def _restore_registry():
    """Restore the global backend registry around each test."""
    yield
    clear_dispatcher_backends()


class StubBackend(ThreadDispatcherBackend):
    """Dispatcher backend used to pin the active backend in tests."""

    def is_available(self):
        return True

    def run_deferred(self, func, *args, **kwargs):
        func(*args, **kwargs)

    def run_sync(self, func, *args, **kwargs):
        return func(*args, **kwargs)


class UnavailableBackend(StubBackend):
    """Dispatcher backend that reports itself as unavailable."""

    def is_available(self):
        return False


class ExplodingBackend(StubBackend):
    """Dispatcher backend that fails on every probe."""

    def is_available(self):
        raise RuntimeError("probe failed")

    def is_main_thread(self):
        raise RuntimeError("probe failed")


class TestDiagnosticsReport:
    """Tests for the structure and guarantees of the diagnostics report."""

    def test_exported_from_package(self):
        assert auroraview.diagnostics is diagnostics
        assert "diagnostics" in auroraview.__all__
        assert "format_diagnostics" in auroraview.__all__

    def test_report_contains_all_sections(self):
        report = diagnostics()

        for section in REQUIRED_SECTIONS:
            assert section in report

    def test_report_is_json_serialisable(self):
        report = diagnostics()

        # Raises TypeError if any value is not JSON-serialisable.
        json.dumps(report)

    def test_host_section_shape(self):
        host = diagnostics()["host"]

        assert "name" in host
        assert "is_dcc" in host
        assert isinstance(host["is_dcc"], bool)

    def test_dispatcher_reports_backend_list(self):
        dispatcher = diagnostics()["dispatcher"]

        assert isinstance(dispatcher["backends"], list)
        assert dispatcher["backends"], "built-in backends must be registered"

        for entry in dispatcher["backends"]:
            assert set(entry) == {"name", "priority", "available"}
            assert isinstance(entry["priority"], int)
            assert isinstance(entry["available"], bool)

    def test_dispatcher_records_active_backend_and_priority(self):
        register_dispatcher_backend(StubBackend, priority=10_000, name="Stub")

        dispatcher = diagnostics()["dispatcher"]

        assert dispatcher["backend"] == "Stub"
        assert dispatcher["priority"] == 10_000

    def test_unavailable_backend_is_reported_not_selected(self):
        register_dispatcher_backend(UnavailableBackend, priority=10_000, name="Unavailable")

        dispatcher = diagnostics()["dispatcher"]

        entry = next(e for e in dispatcher["backends"] if e["name"] == "Unavailable")
        assert entry["available"] is False
        assert dispatcher["backend"] != "Unavailable"

    def test_thread_section_compares_python_and_host_view(self):
        thread = diagnostics()["thread"]

        assert thread["is_python_main_thread"] is True
        assert thread["is_host_main_thread"] is True
        assert thread["name"]

    def test_qt_section_is_reported_without_raising(self):
        qt = diagnostics()["qt"]

        assert set(qt) == {"available", "binding", "api", "app_instance"}
        assert isinstance(qt["available"], bool)

    def test_auroraview_section_reports_version(self):
        info = diagnostics()["auroraview"]

        assert info["version"] == auroraview.__version__
        assert isinstance(info["native_core"], bool)

    def test_missing_native_core_is_a_state_not_an_error(self, monkeypatch):
        # The native extension is optional, so an unimportable ``_core`` is a
        # reported state rather than a broken probe.
        monkeypatch.setitem(sys.modules, "auroraview._core", None)

        report = diagnostics()

        assert report["auroraview"]["native_core"] is False
        assert report["auroraview"]["native_core_error"]
        assert not [e for e in report["errors"] if e["probe"] == "auroraview._core"]

        # The reason is still rendered, so triage keeps what it needs.
        text = format_diagnostics(report)
        assert "native core: no" in text
        assert report["auroraview"]["native_core_error"] in text

    def test_qt_probe_reads_qtcore_from_the_detected_binding(self, monkeypatch):
        if importlib.util.find_spec("PySide6") is None:
            pytest.skip("PySide6 is not installed")

        # A binding installed without qtpy must not be reported as a probe
        # failure; QtCore is read from the binding itself in that case.
        monkeypatch.setitem(sys.modules, "qtpy", None)
        monkeypatch.setattr(_DIAGNOSTICS_MODULE, "_QT_BINDINGS", ("PySide6",))

        report = diagnostics()

        assert report["qt"]["binding"] == "PySide6"
        assert report["qt"]["available"] is True
        assert isinstance(report["qt"]["app_instance"], bool)
        assert not [e for e in report["errors"] if e["probe"] == "qt.app_instance"]


class TestDiagnosticsRobustness:
    """diagnostics() must never raise, even when probes fail."""

    def test_failing_backend_is_skipped_not_fatal(self):
        register_dispatcher_backend(ExplodingBackend, priority=10_000, name="Exploding")

        report = diagnostics()

        # The failing backend must not be selected, and the report must be whole.
        assert set(REQUIRED_SECTIONS) <= set(report)
        assert report["dispatcher"]["backend"] != "Exploding"

        entry = next(e for e in report["dispatcher"]["backends"] if e["name"] == "Exploding")
        assert entry["available"] is False

    def test_probe_failure_is_recorded_not_raised(self, monkeypatch):
        def boom():
            raise RuntimeError("host probe failed")

        monkeypatch.setattr(_DIAGNOSTICS_MODULE, "get_current_dcc_name", boom)

        report = diagnostics()

        # The report is still produced; the failure lands in `errors`.
        assert set(REQUIRED_SECTIONS) <= set(report)
        assert report["host"]["name"] is None
        assert any(entry["probe"] == "host" for entry in report["errors"])

    def test_env_override_is_reported(self, monkeypatch):
        monkeypatch.setenv("AURORAVIEW_DISPATCHER", "fallback")

        dispatcher = diagnostics()["dispatcher"]

        assert dispatcher["env_override"] == "fallback"

    def test_env_override_absent_is_none(self, monkeypatch):
        monkeypatch.delenv("AURORAVIEW_DISPATCHER", raising=False)

        assert diagnostics()["dispatcher"]["env_override"] is None


class TestFormatDiagnostics:
    """Tests for the human-readable rendering."""

    def test_renders_all_sections(self):
        text = format_diagnostics()

        for label in ("auroraview", "platform", "python", "host", "dispatcher"):
            assert label in text

    def test_renders_backend_table(self):
        text = format_diagnostics()

        assert "backends   :" in text
        assert "Fallback" in text

    def test_renders_provided_report_without_recollecting(self):
        report = diagnostics()
        report["host"]["name"] = "SentinelHost"

        assert "SentinelHost" in format_diagnostics(report)

    def test_renders_errors_when_present(self, monkeypatch):
        def boom():
            raise RuntimeError("host probe failed")

        monkeypatch.setattr(_DIAGNOSTICS_MODULE, "get_current_dcc_name", boom)

        text = format_diagnostics()

        assert "errors     :" in text
        assert "host probe failed" in text

    def test_omits_errors_section_when_clean(self):
        clear_dispatcher_backends()

        text = format_diagnostics()

        # No failing probe, so the errors block must not be emitted.
        assert "errors     :" not in text
