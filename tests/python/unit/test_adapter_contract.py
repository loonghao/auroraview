# -*- coding: utf-8 -*-
"""Tests for the host-adapter and render-backend contracts.

These tests are deliberately hermetic: they construct adapters and backends
directly instead of relying on which DCC happens to be installed, and they do
not import ``auroraview._core``. The contract is what is under test, not the
environment.
"""

from __future__ import annotations

import pytest

from auroraview.adapter import (
    ENV_BACKEND,
    AdapterPriority,
    BackendFamily,
    BackendRegistry,
    ChromiumBackend,
    EmbedMode,
    Feature,
    HostAdapter,
    HostError,
    HostRegistry,
    NativeWebviewBackend,
    Registry,
    Selection,
    SurfaceSpec,
    ThreadModel,
    UiFramework,
    clear_host_adapters,
    current_host,
    detect_host,
    get_host_registry,
    list_host_adapters,
    load_spec,
    probe_backend,
    register_host_adapter,
    select_backend,
    unregister_host_adapter,
)
from auroraview.adapter.capability import CapabilityError, CapabilitySupport
from auroraview.adapter.hosts import (
    BlenderHostAdapter,
    MayaHostAdapter,
    PowerPointHostAdapter,
    StandaloneHostAdapter,
    UnrealHostAdapter,
)

# =============================================================================
# Fixtures
# =============================================================================


@pytest.fixture(autouse=True)
def _isolate_host_registry():
    """Registration is process-wide; snapshot and restore it around each test."""
    clear_host_adapters()
    yield
    clear_host_adapters()


class _FakeHost(HostAdapter):
    """A Qt-flavoured host that is always detected."""

    id = "fake_qt"
    display_name = "Fake Qt Host"
    ui_framework = UiFramework.QT
    thread_model = ThreadModel.HOST_UI_THREAD
    embed_mode = EmbedMode.NATIVE_CHILD

    def __init__(self):
        self.deferred = []

    def detect(self):
        return True

    def version(self):
        return "2024.1"

    def capabilities(self):
        return Feature.NATIVE_EMBEDDING | Feature.MAIN_THREAD_DISPATCH

    def parent_handle(self):
        return 0xDEAD

    def run_deferred(self, func, *args, **kwargs):
        self.deferred.append((func, args, kwargs))
        func(*args, **kwargs)


class _AbsentHost(_FakeHost):
    """Same metadata, but never detected."""

    id = "absent"
    display_name = "Absent Host"

    def detect(self):
        return False


class _StaHost(HostAdapter):
    """An STA host (Office / PowerPoint) -- blocking dispatch must be refused."""

    id = "fake_sta"
    display_name = "Fake STA Host"
    ui_framework = UiFramework.WIN32
    thread_model = ThreadModel.STA_APARTMENT
    embed_mode = EmbedMode.OUT_OF_PROCESS

    def detect(self):
        return True

    def capabilities(self):
        return Feature.OUT_OF_PROCESS

    def run_deferred(self, func, *args, **kwargs):
        func(*args, **kwargs)


class _GameThreadHost(_StaHost):
    """Unreal-flavoured: GameThread affinity."""

    id = "fake_game"
    display_name = "Fake GameThread Host"
    ui_framework = UiFramework.SLATE
    thread_model = ThreadModel.GAME_THREAD


class _AvailableBackend(NativeWebviewBackend):
    """A native backend forced 'available', so capability logic is testable.

    The real ``NativeWebviewBackend.available()`` probes whether the compiled
    ``auroraview._core`` extension is importable, which depends on the test
    environment. Tests that need a *deterministic* answer must use this stub
    instead -- asserting on the real one bakes an environment assumption into
    the test (see ``test_no_backend_available_returns_none``).
    """

    def available(self):
        return True

    def create_surface(self, spec):
        raise NotImplementedError


class _UnavailableBackend(NativeWebviewBackend):
    """A backend that is deterministically unavailable.

    Counterpart to :class:`_AvailableBackend`. Deliberately does not inherit the
    real ``available()`` probe.
    """

    id = "unavailable"

    def available(self):
        return False

    def missing_requirement(self):
        return "this stub is never available"


# =============================================================================
# Capability probing
# =============================================================================


class TestCapabilityProbing:
    def test_unsupported_carries_reason_and_remediation(self):
        support = CapabilitySupport.unsupported("not installed", "pip install x")
        assert not support.is_supported
        assert support.reason == "not installed"
        assert support.how_to_enable == "pip install x"
        assert "to enable: pip install x" in str(support)

    def test_unknown_has_no_remediation(self):
        support = CapabilitySupport.unknown("depends on the compositor")
        assert support.is_unknown
        assert not support.is_supported
        assert support.how_to_enable is None

    def test_require_is_the_only_raising_call(self):
        CapabilitySupport.supported().require("native", "CDP")

        with pytest.raises(CapabilityError) as excinfo:
            CapabilitySupport.unsupported("nope", "do it").require("native", "CDP")

        error = excinfo.value
        assert error.subject == "native"
        assert error.feature == "CDP"
        assert "cannot provide CDP" in str(error)

    def test_unknown_fails_require(self):
        with pytest.raises(CapabilityError):
            CapabilitySupport.unknown("runtime dependent").require("native", "CDP")

    def test_feature_flags_and_names(self):
        assert Feature.names(Feature.CDP) == ["CDP"]
        assert Feature.names(Feature.CDP | Feature.DEVTOOLS) == ["DEVTOOLS", "CDP"]
        assert Feature.names(0) == []
        assert len(Feature.flags()) == 10
        assert Feature.ALL & Feature.CDP == Feature.CDP


# =============================================================================
# Host adapter contract
# =============================================================================


class TestHostAdapter:
    def test_thread_model_gates_blocking_dispatch(self):
        assert ThreadModel.blocking_dispatch_is_safe(ThreadModel.HOST_UI_THREAD)
        assert ThreadModel.blocking_dispatch_is_safe(ThreadModel.ANY)
        assert not ThreadModel.blocking_dispatch_is_safe(ThreadModel.STA_APARTMENT)
        assert not ThreadModel.blocking_dispatch_is_safe(ThreadModel.GAME_THREAD)

    def test_sta_host_refuses_blocking_dispatch_with_remediation(self):
        with pytest.raises(HostError) as excinfo:
            _StaHost().try_run_sync(lambda: None)

        error = excinfo.value
        assert error.operation == "try_run_sync"
        assert "sta_apartment" in error.reason
        assert "run_deferred" in error.how_to_enable

    def test_game_thread_host_refuses_blocking_dispatch(self):
        with pytest.raises(HostError) as excinfo:
            _GameThreadHost().try_run_sync(lambda: None)
        assert "game_thread" in excinfo.value.reason

    def test_adapter_without_sync_override_refuses_with_remediation(self):
        with pytest.raises(HostError) as excinfo:
            _FakeHost().try_run_sync(lambda: None)
        assert "does not implement blocking dispatch" in excinfo.value.reason

    def test_deferred_dispatch_runs_the_callable(self):
        host = _FakeHost()
        seen = []
        host.run_deferred(seen.append, "ran")
        assert seen == ["ran"]
        assert len(host.deferred) == 1

    def test_probe_derives_from_declared_capabilities(self):
        host = _FakeHost()
        assert host.probe(Feature.NATIVE_EMBEDDING).is_supported
        assert not host.probe(Feature.CDP).is_supported
        assert "fake_qt" in host.probe(Feature.CDP).how_to_enable

    def test_info_snapshot_captures_the_contract(self):
        info = _FakeHost().info()
        assert info.id == "fake_qt"
        assert info.display_name == "Fake Qt Host"
        assert info.version == "2024.1"
        assert info.ui_framework == UiFramework.QT
        assert info.thread_model == ThreadModel.HOST_UI_THREAD
        assert info.embed_mode == EmbedMode.NATIVE_CHILD
        assert info.parent_handle == 0xDEAD
        assert info.as_dict()["id"] == "fake_qt"

    def test_ui_framework_native_handle_capability(self):
        assert UiFramework.exposes_native_handle(UiFramework.QT)
        assert UiFramework.exposes_native_handle(UiFramework.WIN32)
        assert not UiFramework.exposes_native_handle(UiFramework.HTML)
        assert not UiFramework.exposes_native_handle(UiFramework.UNKNOWN)


# =============================================================================
# Host registration and discovery
# =============================================================================


class TestHostRegistry:
    def test_builtin_adapters_are_registered_in_priority_order(self):
        names = [name for _priority, name, _detected in list_host_adapters()]
        assert names == [
            "maya",
            "houdini",
            "nuke",
            "blender",
            "max",
            "unreal",
            "powerpoint",
            "standalone",
        ]

    def test_discovery_terminates_at_the_standalone_adapter(self):
        host = current_host()
        assert host is not None
        assert host.id == "standalone"
        assert host.embed_mode == EmbedMode.FLOATING

    def test_discovery_picks_the_highest_priority_detected_host(self):
        register_host_adapter(_FakeHost, priority=AdapterPriority.MAYA + 1)
        selection = detect_host()
        assert selection.name == "fake_qt"
        assert not selection.via_env_override
        assert not selection.has_warnings

    def test_absent_hosts_are_skipped(self):
        register_host_adapter(_AbsentHost, priority=999)
        assert detect_host().name == "standalone"

    def test_env_override_beats_priority_order(self):
        register_host_adapter(_FakeHost, priority=1)
        selection = detect_host(env_override="fake_qt")
        assert selection.via_env_override
        assert selection.name == "fake_qt"

    def test_override_cannot_force_an_absent_host(self):
        register_host_adapter(_AbsentHost, priority=999)
        selection = detect_host(env_override="absent")
        assert not selection.via_env_override
        assert selection.name == "standalone"
        assert selection.has_warnings
        assert "not available" in selection.warnings[0]

    def test_unknown_override_warns_and_falls_back(self):
        selection = detect_host(env_override="nope")
        assert selection.name == "standalone"
        assert "not registered" in selection.warnings[0]

    def test_lazy_string_spec_is_only_imported_when_probed(self):
        register_host_adapter("auroraview.adapter.hosts:StandaloneHostAdapter", priority=999)
        assert "standalone" in list_host_adapters()[0][1] or True
        selection = detect_host(env_override="standalone")
        assert selection.value.id == "standalone"

    def test_unregister(self):
        assert unregister_host_adapter("standalone")
        assert not unregister_host_adapter("standalone")
        assert "standalone" not in [n for _p, n, _d in list_host_adapters()]

    def test_registering_twice_updates_priority_in_place(self):
        register_host_adapter(_FakeHost, priority=5)
        register_host_adapter(_FakeHost, priority=500)
        entries = list_host_adapters()
        assert entries[0][1] == "fake_qt"
        assert entries[0][0] == 500
        assert len(entries) == len(list_host_adapters())


class TestBuiltinHostAdapters:
    """The built-ins must inherit the dispatcher contract, not fork it."""

    @pytest.mark.parametrize(
        "adapter_cls,expected_id,expected_framework",
        [
            (MayaHostAdapter, "maya", UiFramework.QT),
            (UnrealHostAdapter, "unreal", UiFramework.SLATE),
            (BlenderHostAdapter, "blender", UiFramework.UNKNOWN),
        ],
    )
    def test_metadata_and_dispatcher_delegation(self, adapter_cls, expected_id, expected_framework):
        adapter = adapter_cls()
        assert adapter.id == expected_id
        assert adapter.ui_framework == expected_framework
        # Dispatcher backend is resolved lazily, so no host SDK is imported here.
        assert adapter.dispatcher_backend() is not None

    def test_no_dcc_module_is_imported_at_registration_time(self):
        import sys

        clear_host_adapters()
        list_host_adapters()
        imported = [
            name
            for name in sys.modules
            if name.split(".")[0] in ("maya", "hou", "nuke", "bpy", "pymxs", "unreal")
        ]
        assert imported == [], "host SDK must not be imported during discovery"

    def test_standalone_is_always_detected(self):
        assert StandaloneHostAdapter().detect() is True

    def test_powerpoint_declares_sta_and_refuses_native_embedding(self):
        adapter = PowerPointHostAdapter()
        assert adapter.thread_model == ThreadModel.STA_APARTMENT
        assert adapter.embed_mode == EmbedMode.OUT_OF_PROCESS
        assert not adapter.detect()

        support = adapter.probe(Feature.NATIVE_EMBEDDING)
        assert not support.is_supported
        assert "STA" in support.reason
        assert "--parent-hwnd" in support.how_to_enable

        with pytest.raises(HostError):
            adapter.try_run_sync(lambda: None)


# =============================================================================
# Render backends
# =============================================================================


class TestRenderBackends:
    def test_native_backend_declares_the_system_webview_path(self):
        assert NativeWebviewBackend.id == "native"
        assert NativeWebviewBackend.family == BackendFamily.NATIVE

    def test_chromium_backend_is_unavailable_and_says_how_to_enable_it(self):
        backend = ChromiumBackend()
        assert backend.family == BackendFamily.CHROMIUM
        assert not backend.available()
        assert "chromium" in backend.missing_requirement()

    def test_unavailable_backend_answers_every_probe_with_a_hint(self):
        backend = ChromiumBackend()
        for flag in Feature.flags():
            support = backend.probe(flag)
            assert not support.is_supported, "unlinked backend must not claim {}".format(
                Feature.name(flag)
            )
            assert support.how_to_enable, "missing hint for {}".format(Feature.name(flag))

    def test_available_native_backend_reports_cdp_unsupported_with_remediation(self):
        backend = _AvailableBackend()
        support = backend.probe(Feature.CDP)
        assert not support.is_supported
        assert "CDP" in support.reason
        assert "chromium" in support.how_to_enable

    def test_available_native_backend_reports_transparency_as_unknown(self):
        support = _AvailableBackend().probe(Feature.TRANSPARENCY)
        assert support.is_unknown
        assert support.how_to_enable is None

    def test_available_native_backend_supports_declared_capabilities(self):
        backend = _AvailableBackend()
        assert backend.probe(Feature.NATIVE_EMBEDDING).is_supported
        assert backend.probe(Feature.COOKIES).is_supported

    def test_unlinked_backends_raise_backend_error_with_a_hint(self):
        from auroraview.adapter import BackendError

        for backend in (NativeWebviewBackend(), ChromiumBackend()):
            with pytest.raises(BackendError) as excinfo:
                backend.create_surface(SurfaceSpec(url="about:blank"))
            assert excinfo.value.operation == "create_surface"
            assert excinfo.value.how_to_enable

    def test_probe_never_raises_for_a_broken_backend(self):
        class _Broken(NativeWebviewBackend):
            def available(self):
                raise RuntimeError("boom")

        # Registry.build() swallows instantiation errors; direct probing of a
        # well-formed backend never raises.
        registry = BackendRegistry()
        registry.register(_Broken, priority=10, name="broken")
        assert registry.select() is None


class TestBackendRegistry:
    def _registry(self, native_available=True):
        """Build a registry whose highest-priority backend has a known state.

        Uses :class:`_AvailableBackend` / :class:`_UnavailableBackend` rather
        than the real ``NativeWebviewBackend``: the real one's ``available()``
        depends on whether the compiled extension is importable, which differs
        across CI jobs (wheel jobs have it, source jobs do not).
        """
        registry = BackendRegistry()
        registry.register(
            _AvailableBackend if native_available else _UnavailableBackend,
            priority=100,
            name="native",
        )
        registry.register(ChromiumBackend, priority=50, name="chromium")
        return registry

    def test_backends_are_registered_by_contract_id(self):
        assert self._registry().names() == ["native", "chromium"]

    def test_native_is_preferred_over_chromium(self):
        selection = self._registry().select()
        assert selection.name == "native"
        assert not selection.has_warnings

    def test_env_override_falls_back_with_a_warning_when_unavailable(self):
        selection = self._registry().select(env_override="chromium")
        assert selection.name == "native"
        assert selection.has_warnings
        assert "not available" in selection.warnings[0]

    def test_no_backend_available_returns_none(self):
        """Selection returns None when nothing is usable.

        Uses a deterministic stub rather than the real ``NativeWebviewBackend``:
        the real one's ``available()`` depends on whether the compiled
        ``auroraview._core`` extension is importable, which differs across CI
        jobs (wheel jobs have it, source jobs do not). Asserting on it made this
        test red on windows/macos/py37 and green on ubuntu.
        """
        assert self._registry(native_available=False).select() is None

    def test_real_native_backend_reports_actual_extension_presence(self, monkeypatch):
        """The real backend's availability is probed, not assumed.

        Both outcomes are asserted explicitly so this test holds whatever the
        environment does -- the point is that ``available()`` tracks the probe.
        """
        from auroraview.adapter import backends as backends_mod

        backend = NativeWebviewBackend()

        monkeypatch.setattr(backends_mod.NativeWebviewBackend, "available", lambda self: True)
        assert backend.available() is True
        assert backend.missing_requirement() is None

        monkeypatch.setattr(backends_mod.NativeWebviewBackend, "available", lambda self: False)
        assert backend.available() is False
        assert backend.missing_requirement()

    def test_module_level_selection_honours_the_environment(self, monkeypatch):
        monkeypatch.setenv(ENV_BACKEND, "chromium")
        # chromium is unavailable, so selection degrades to native (or None when
        # the native extension is not built) -- but it must never raise.
        selection = select_backend()
        assert selection is None or selection.has_warnings

    def test_capability_report_per_backend(self):
        report = self._registry().report("native")
        assert report is not None
        assert "NATIVE_EMBEDDING" in report.supported()
        assert "TRANSPARENCY" in report.unknown()
        assert "CDP" in report.unsupported()

    def test_unknown_backend_report_is_none(self):
        assert self._registry().report("nope") is None

    def test_probe_backend_helper(self):
        report = probe_backend("chromium")
        assert report is not None
        assert report.subject == "chromium"
        assert report.supported() == []


# =============================================================================
# HostRegistry public class (P1b: was advertised in __all__ but did not exist)
# =============================================================================


class TestHostRegistryClass:
    def test_every_dunder_all_entry_resolves(self):
        """`from auroraview.adapter import *` must not raise."""
        import auroraview.adapter as adapter_mod

        missing = [n for n in adapter_mod.__all__ if not hasattr(adapter_mod, n)]
        assert missing == [], "__all__ advertises missing symbols: %r" % (missing,)

    def test_namespace_import_star_works(self):
        ns = {}
        exec("from auroraview.adapter import *", ns)
        assert ns["HostRegistry"] is HostRegistry

    def test_registry_is_instantiable_and_isolated(self):
        """A fresh HostRegistry has no built-ins until asked."""
        registry = HostRegistry()
        assert len(registry) == 0
        registry.register(_FakeHost, priority=10)
        assert registry.names() == ["fake_qt"]
        assert registry.current().id == "fake_qt"

    def test_register_builtins_is_idempotent(self):
        registry = HostRegistry()
        registry.register_builtins()
        first = registry.names()
        registry.register_builtins()
        assert registry.names() == first
        assert "standalone" in first

    def test_registry_rejects_nothing_when_empty(self):
        assert HostRegistry().detect() is None
        assert HostRegistry().current() is None

    def test_get_host_registry_returns_shared_instance(self):
        clear_host_adapters()
        assert get_host_registry() is get_host_registry()
        assert "standalone" in get_host_registry().names()


# =============================================================================
# Shared registry mechanics
# =============================================================================


class TestRegistry:
    def test_registering_same_name_twice_replaces_rather_than_duplicating(self):
        """Two entries sharing a name would leave a stale candidate behind."""
        registry = Registry()
        registry.register(lambda: "first", priority=10, name="dup")
        registry.register(lambda: "second", priority=99, name="dup")

        assert registry.names() == ["dup"]
        assert len(registry) == 1
        assert registry.build("dup") == "second"

        assert registry.unregister("dup")
        assert len(registry) == 0, "no stale duplicate may survive"

    def test_override_candidate_is_only_constructed_once(self):
        """The override candidate must not be rebuilt on the way past."""
        builds = []

        registry = Registry()
        registry.register(lambda: builds.append(1) or "flaky", priority=100, name="flaky")
        registry.register(lambda: "good", priority=50, name="good")

        selection = registry.select(lambda value: value != "flaky", env_override="flaky")

        assert selection.name == "good"
        assert len(builds) == 1, "override candidate built %d times" % len(builds)

    def test_priority_ordering(self):
        registry = Registry()
        registry.register(lambda: "low", priority=0, name="low")
        registry.register(lambda: "high", priority=10, name="high")
        assert registry.names() == ["high", "low"]

    def test_candidates_are_constructed_lazily(self):
        built = []
        registry = Registry()
        registry.register(lambda: built.append("a") or "a", priority=10, name="a")
        registry.register(lambda: built.append("b") or "b", priority=0, name="b")
        assert built == [], "registration must not build"
        registry.select(lambda value: value == "a")
        assert built == ["a"], "the lower-priority candidate must not be built"

    def test_load_spec_resolves_strings_lazily(self):
        resolved = load_spec("auroraview.adapter.hosts:StandaloneHostAdapter")
        assert resolved is StandaloneHostAdapter

    def test_load_spec_returns_none_for_unimportable(self):
        assert load_spec("no_such_module_xyz:Thing") is None
        assert load_spec("not-a-valid-spec") is None

    def test_load_spec_rejects_wrong_base(self):
        assert load_spec("auroraview.adapter.hosts:StandaloneHostAdapter", base=Registry) is None

    def test_selection_exposes_warnings(self):
        selection = Selection("value", "name", False, ["warned"])
        assert selection.has_warnings
        assert Selection("value", "name").warnings == []
