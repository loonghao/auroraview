# -*- coding: utf-8 -*-
"""AuroraView host-adapter and render-backend contracts.

This package is the **dependency-direction firewall** for the AuroraView
package split. It holds contracts, value types and registries -- and no host
implementation.

The invariant
-------------

::

    auroraview (core)   <-- imported by -->   auroraview_maya / _unreal / _unity
          |                                            |
          +----------------> adapter contracts <-------+

Core never imports a host module. Host packages import core and register an
adapter here. Discovery runs through :class:`HostRegistry`, so adding a host is
a registration, not an edit to core.

Quick start -- register a new host
----------------------------------

::

    from auroraview.adapter import HostAdapter, register_host_adapter

    class GodotHostAdapter(HostAdapter):
        id = "godot"
        display_name = "Godot"
        ui_framework = UiFramework.UNKNOWN
        thread_model = ThreadModel.HOST_UI_THREAD
        embed_mode = EmbedMode.FLOATING

        def detect(self):
            try:
                import godot  # noqa: F401
                return True
            except ImportError:
                return False

        def run_deferred(self, func, *args, **kwargs):
            func(*args, **kwargs)

    register_host_adapter(GodotHostAdapter, priority=140)

Or without importing AuroraView at all, from a separate distribution::

    auroraview.adapter.register_host_adapter(
        "auroraview_godot.adapter:GodotHostAdapter", priority=140
    )

The ``"module:ClassName"`` form is resolved lazily, so ``auroraview_godot`` is
imported only when host discovery actually probes it.

Quick start -- pick a render backend
------------------------------------

::

    from auroraview.adapter import select_backend, probe_backend

    selection = select_backend()          # honours AURORAVIEW_BACKEND
    print(selection.name, selection.warnings)

    print(probe_backend("chromium").render())

Capability probing never raises
-------------------------------

Missing features are data, not exceptions. Probing returns a
:class:`CapabilitySupport` -- ``supported``, ``unsupported`` (with a reason and
a ``how_to_enable`` hint), or ``unknown``. The single raising call is
``CapabilitySupport.require(...)``, used only by callers that cannot degrade
gracefully.
"""

from __future__ import annotations

import os
from typing import Any, Callable, List, Optional, Tuple, Type, Union

from .backends import (
    ENV_BACKEND,
    BackendError,
    BackendFamily,
    BackendRegistry,
    ChromiumBackend,
    NativeWebviewBackend,
    RenderBackend,
    RenderSurface,
    SurfaceSpec,
    get_backend_registry,
    probe_backend,
    select_backend,
)
from .base import (
    EmbedMode,
    HostAdapter,
    HostError,
    HostInfo,
    ThreadModel,
    UiFramework,
)
from .capability import (
    CapabilityError,
    CapabilityReport,
    CapabilitySupport,
    Feature,
)
from .hosts import (
    BlenderHostAdapter,
    DispatcherBackedHostAdapter,
    HoudiniHostAdapter,
    MaxHostAdapter,
    MayaHostAdapter,
    NukeHostAdapter,
    PowerPointHostAdapter,
    StandaloneHostAdapter,
    UnrealHostAdapter,
)
from .registry import Registry, Selection, load_spec

__all__ = [
    # capability
    "Feature",
    "CapabilitySupport",
    "CapabilityError",
    "CapabilityReport",
    # registry
    "Registry",
    "Selection",
    "load_spec",
    # host contract
    "UiFramework",
    "ThreadModel",
    "EmbedMode",
    "HostAdapter",
    "HostError",
    "HostInfo",
    "HostRegistry",
    "AdapterPriority",
    "ENV_HOST",
    "get_host_registry",
    "register_host_adapter",
    "unregister_host_adapter",
    "clear_host_adapters",
    "list_host_adapters",
    "detect_host",
    "current_host",
    # built-in host adapters
    "DispatcherBackedHostAdapter",
    "StandaloneHostAdapter",
    "MayaHostAdapter",
    "HoudiniHostAdapter",
    "NukeHostAdapter",
    "BlenderHostAdapter",
    "MaxHostAdapter",
    "UnrealHostAdapter",
    "PowerPointHostAdapter",
    # backend contract
    "BackendFamily",
    "BackendError",
    "BackendRegistry",
    "RenderBackend",
    "RenderSurface",
    "SurfaceSpec",
    "NativeWebviewBackend",
    "ChromiumBackend",
    "ENV_BACKEND",
    "get_backend_registry",
    "select_backend",
    "probe_backend",
]

#: Environment variable used to pin a host adapter.
ENV_HOST = "AURORAVIEW_HOST"


class AdapterPriority(object):
    """Priority constants for host adapters.

    Higher priority adapters are probed first. DCC hosts outrank generic
    adapters; the standalone adapter is always last so discovery always
    terminates.

    These mirror :class:`auroraview.utils.thread_dispatcher.DispatcherPriority`
    so the two registries read the same way.
    """

    MAYA = 200
    HOUDINI = 190
    NUKE = 180
    BLENDER = 170
    MAX = 160
    UNREAL = 150
    #: Adapters at or above this are considered DCC hosts.
    DCC_THRESHOLD = 150
    #: Hosts with no Python dispatch backend yet (PowerPoint, Unity).
    NON_PYTHON = 100
    STANDALONE = 0


#: Spec accepted by :func:`register_host_adapter`.
HostSpec = Union[Type[HostAdapter], Callable[[], HostAdapter], str]


class HostRegistry(object):
    """Registry of host adapters.

    Wraps the shared :class:`~auroraview.adapter.registry.Registry` so host
    adapters use exactly the same priority / lazy-spec / env-override mechanics
    as render backends.

    The process-wide instance is available via :func:`get_host_registry`.
    """

    def __init__(self) -> None:
        self._registry: Registry = Registry(base=HostAdapter, kind="host adapter")
        self._builtins_registered = False

    def register(self, adapter: "HostSpec", priority: int = 0, name: str = "") -> None:
        """Register an adapter class, factory, or ``"module:ClassName"`` spec.

        When *name* is omitted it defaults to the adapter's contract ``id``, so
        ``AURORAVIEW_HOST=maya`` matches the registered name.
        """
        if not name and isinstance(adapter, type):
            adapter_id = getattr(adapter, "id", None)
            if isinstance(adapter_id, str):
                name = adapter_id
        self._registry.register(adapter, priority=priority, name=name)

    def unregister(self, name: str) -> bool:
        """Remove an adapter by name."""
        return self._registry.unregister(name)

    def clear(self) -> None:
        """Remove every adapter."""
        self._registry.clear()
        self._builtins_registered = False

    def names(self) -> List[str]:
        """Registered adapter names in priority order."""
        return self._registry.names()

    def __len__(self) -> int:
        return len(self._registry)

    def list(self) -> List[Tuple[int, str, bool]]:
        """``(priority, name, detected)`` for every adapter, highest first."""
        result: List[Tuple[int, str, bool]] = []
        for priority, _spec, name in self._registry.entries:
            adapter = self._registry.build(name)
            result.append((priority, name, bool(adapter and adapter.detect())))
        return result

    def detect(self, env_override: Optional[str] = None) -> Optional[Selection]:
        """Detect the host, honouring an override (defaults to ``AURORAVIEW_HOST``)."""
        if env_override is None:
            env_override = os.environ.get(ENV_HOST, "")
        return self._registry.select(lambda adapter: adapter.detect(), env_override=env_override)

    def current(self) -> Optional[HostInfo]:
        """Return a :class:`HostInfo` snapshot of the detected host."""
        selection = self.detect()
        if selection is None:
            return None
        return selection.value.info()

    def register_builtins(self) -> None:
        """Register the built-in host adapters. Idempotent."""
        if self._builtins_registered:
            return
        self._builtins_registered = True
        self.register(MayaHostAdapter, priority=AdapterPriority.MAYA)
        self.register(HoudiniHostAdapter, priority=AdapterPriority.HOUDINI)
        self.register(NukeHostAdapter, priority=AdapterPriority.NUKE)
        self.register(BlenderHostAdapter, priority=AdapterPriority.BLENDER)
        self.register(MaxHostAdapter, priority=AdapterPriority.MAX)
        self.register(UnrealHostAdapter, priority=AdapterPriority.UNREAL)
        self.register(PowerPointHostAdapter, priority=AdapterPriority.NON_PYTHON)
        self.register(StandaloneHostAdapter, priority=AdapterPriority.STANDALONE)


_BUILTINS_REGISTERED = False
_HOST_REGISTRY: Optional[HostRegistry] = None


def get_host_registry() -> HostRegistry:
    """Return the process-wide host registry, registering built-ins on first use."""
    global _HOST_REGISTRY, _BUILTINS_REGISTERED

    if _HOST_REGISTRY is None:
        _HOST_REGISTRY = HostRegistry()

    _HOST_REGISTRY.register_builtins()
    return _HOST_REGISTRY


def _registry() -> HostRegistry:
    """Backwards-compatible alias for :func:`get_host_registry`."""
    return get_host_registry()


def register_host_adapter(adapter: HostSpec, priority: int = 0, name: str = "") -> None:
    """Register a host adapter.

    Args:
        adapter: A :class:`HostAdapter` subclass, a zero-argument factory, or a
            ``"module:ClassName"`` string. The string form is imported lazily,
            which is how a separate ``auroraview_<host>`` distribution plugs in
            without AuroraView importing it.
        priority: Higher is probed first; see :class:`AdapterPriority`.
        name: Optional display name; defaults to the class name.

    Registering the same adapter twice updates its priority in place.
    """
    _registry().register(adapter, priority=priority, name=name)


def unregister_host_adapter(name: str) -> bool:
    """Remove a host adapter by name. Returns whether it was present."""
    return _registry().unregister(name)


def clear_host_adapters() -> None:
    """Remove every registered host adapter.

    Mainly for tests: ``register_host_adapter`` mutates process-wide state.
    """
    _registry().clear()


def list_host_adapters() -> List[Tuple[int, str, bool]]:
    """Return ``(priority, name, detected)`` for every adapter, highest first.

    Mirrors
    :func:`auroraview.utils.thread_dispatcher.list_dispatcher_backends` so
    diagnostics output looks the same in both languages.
    """
    return _registry().list()


def detect_host(env_override: Optional[str] = None) -> Optional[Selection]:
    """Detect the current host.

    Args:
        env_override: Explicit override; defaults to the ``AURORAVIEW_HOST``
            environment variable.

    Returns:
        A :class:`Selection` whose ``.value`` is the detected
        :class:`HostAdapter`. Falls back through priority order to the
        standalone adapter, so this only returns ``None`` if even that was
        unregistered.
    """
    return _registry().detect(env_override=env_override)


def current_host(env_override: Optional[str] = None) -> Optional[HostInfo]:
    """Return a :class:`HostInfo` snapshot of the detected host."""
    if env_override is None:
        return _registry().current()
    selection = detect_host(env_override=env_override)
    if selection is None:
        return None
    return selection.value.info()
