# -*- coding: utf-8 -*-
"""Tests for the auroraview.diagnostics() runtime diagnostics surface."""

import importlib
import importlib.util
import json
import sys
import types

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


class NoMainThreadBackend(StubBackend):
    """Selectable backend whose host main-thread probe fails."""

    def is_main_thread(self):
        raise RuntimeError("main thread probe failed")


class ImportShim:
    """Stand-in for :mod:`importlib` injecting import failures.

    ``auroraview.diagnostics`` reaches for ``importlib.import_module`` through
    the module attribute, so swapping the attribute is enough to make one
    specific module blow up. Everything else delegates to the real importlib.
    """

    def __init__(self, failures=None, modules=None):
        self.failures = dict(failures or {})
        self.modules = dict(modules or {})

    def import_module(self, name, package=None):
        if name in self.failures:
            raise self.failures[name]
        if name in self.modules:
            return self.modules[name]
        return importlib.import_module(name, package)


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

    def test_version_probe_failure_is_recorded(self, monkeypatch):
        monkeypatch.delattr(auroraview, "__version__")

        report = diagnostics()

        assert report["auroraview"]["version"] is None
        assert any(entry["probe"] == "auroraview.version" for entry in report["errors"])

    def test_path_probe_failure_is_recorded(self, monkeypatch):
        monkeypatch.setattr(auroraview, "__file__", None)

        report = diagnostics()

        assert report["auroraview"]["path"] is None
        assert any(entry["probe"] == "auroraview.path" for entry in report["errors"])

    def test_backend_list_failure_is_recorded(self, monkeypatch):
        def boom():
            raise RuntimeError("backends probe failed")

        monkeypatch.setattr(_DIAGNOSTICS_MODULE, "list_dispatcher_backends", boom)

        report = diagnostics()

        assert report["dispatcher"]["backends"] == []
        assert any(entry["probe"] == "dispatcher.backends" for entry in report["errors"])

    def test_active_backend_failure_is_recorded(self, monkeypatch):
        def boom():
            raise RuntimeError("backend probe failed")

        # The thread probe resolves the backend through the same entry point,
        # so a single failure must not take the whole report down with it.
        monkeypatch.setattr(_DIAGNOSTICS_MODULE, "get_dispatcher_backend", boom)

        report = diagnostics()

        assert report["dispatcher"]["backend"] is None
        assert report["dispatcher"]["priority"] is None
        assert report["thread"]["is_host_main_thread"] is None
        assert any(entry["probe"] == "dispatcher.backend" for entry in report["errors"])
        assert any(entry["probe"] == "thread.is_host_main_thread" for entry in report["errors"])

    def test_host_main_thread_failure_is_recorded(self):
        register_dispatcher_backend(NoMainThreadBackend, priority=10_000, name="NoMainThread")

        report = diagnostics()

        assert report["dispatcher"]["backend"] == "NoMainThread"
        assert report["thread"]["is_host_main_thread"] is None
        assert any(entry["probe"] == "thread.is_host_main_thread" for entry in report["errors"])

    def test_qt_app_instance_failure_is_recorded(self, monkeypatch):
        # A fake qtpy keeps this independent of which binding the runner has.
        monkeypatch.setitem(sys.modules, "qtpy", types.SimpleNamespace(API_NAME="PySide6"))
        monkeypatch.setattr(
            _DIAGNOSTICS_MODULE,
            "importlib",
            ImportShim({"qtpy.QtCore": RuntimeError("QtCore is broken")}),
        )

        report = diagnostics()

        assert report["qt"]["binding"] == "PySide6"
        assert report["qt"]["available"] is True
        assert report["qt"]["app_instance"] is None
        assert any(entry["probe"] == "qt.app_instance" for entry in report["errors"])

    def test_qt_binding_probe_survives_non_import_error(self, monkeypatch):
        # Regression for the escape path: qtpy absent and a half-installed
        # binding raising a non-ImportError must not leave diagnostics().
        monkeypatch.setitem(sys.modules, "qtpy", None)
        monkeypatch.setattr(_DIAGNOSTICS_MODULE, "_QT_BINDINGS", ("PySide6",))
        monkeypatch.setattr(
            _DIAGNOSTICS_MODULE,
            "importlib",
            ImportShim({"PySide6": RuntimeError("PySide6 is broken in this install")}),
        )

        report = diagnostics()

        assert set(REQUIRED_SECTIONS) <= set(report)
        assert report["qt"]["binding"] is None
        assert report["qt"]["available"] is False
        assert any(entry["probe"] == "qt.binding" for entry in report["errors"])

    def test_qt_binding_probe_keeps_going_after_a_broken_binding(self, monkeypatch):
        # A broken binding is recorded, and the remaining ones are still tried.
        fake_qtcore = types.SimpleNamespace(
            QCoreApplication=types.SimpleNamespace(instance=lambda: None)
        )
        monkeypatch.setitem(sys.modules, "qtpy", None)
        monkeypatch.setattr(_DIAGNOSTICS_MODULE, "_QT_BINDINGS", ("BrokenQt", "WorkingQt"))
        monkeypatch.setattr(
            _DIAGNOSTICS_MODULE,
            "importlib",
            ImportShim(
                failures={"BrokenQt": OSError("BrokenQt is half-installed")},
                modules={"WorkingQt": types.SimpleNamespace(), "WorkingQt.QtCore": fake_qtcore},
            ),
        )

        report = diagnostics()

        assert report["qt"]["binding"] == "WorkingQt"
        assert report["qt"]["available"] is True
        assert report["qt"]["app_instance"] is False
        assert any(entry["probe"] == "qt.binding" for entry in report["errors"])
        assert not [e for e in report["errors"] if e["probe"] == "qt.app_instance"]

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
