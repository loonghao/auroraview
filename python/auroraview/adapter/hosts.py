# -*- coding: utf-8 -*-
"""Built-in host adapters.

Every adapter here is a **thin metadata wrapper around an existing thread
dispatcher backend**. The dispatchers in
``auroraview.utils.thread_dispatcher.backends`` already know how to run work on
each host's UI thread, and they already get that right; this module adds the
three things the dispatcher deliberately does not model -- identity, discovery
and embedding -- without duplicating a line of dispatch logic.

That is the whole point: a new host is ~15 lines of metadata plus a dispatcher
backend, not a new package.
"""

from __future__ import annotations

from typing import Any, Callable, Optional, TypeVar

from .base import EmbedMode, HostAdapter, ThreadModel, UiFramework
from .capability import CapabilitySupport, Feature

T = TypeVar("T")

__all__ = [
    "DispatcherBackedHostAdapter",
    "StandaloneHostAdapter",
    "MayaHostAdapter",
    "HoudiniHostAdapter",
    "NukeHostAdapter",
    "BlenderHostAdapter",
    "MaxHostAdapter",
    "UnrealHostAdapter",
    "PowerPointHostAdapter",
]


class DispatcherBackedHostAdapter(HostAdapter):
    """A host adapter that delegates thread dispatch to a dispatcher backend.

    Subclasses supply metadata and a dispatcher backend class; discovery and
    dispatch come from the dispatcher.

    The dispatcher backend is resolved **lazily** so that importing this module
    never imports a host SDK.
    """

    #: ``"module:ClassName"`` of the dispatcher backend to delegate to.
    dispatcher_spec: str = ""
    #: Cached dispatcher backend instance.
    _dispatcher: Any = None

    def _backend(self) -> Any:
        """Return the delegated dispatcher backend, importing it on first use."""
        if self._dispatcher is None:
            if not self.dispatcher_spec:
                raise HostAdapterError(
                    "{} does not declare a dispatcher_spec".format(type(self).__name__)
                )
            module_path, class_name = self.dispatcher_spec.rsplit(":", 1)
            module = __import__(module_path, fromlist=[class_name])
            self._dispatcher = getattr(module, class_name)()
        return self._dispatcher

    def detect(self) -> bool:
        """Delegate presence detection to the dispatcher backend."""
        try:
            return bool(self._backend().is_available())
        except Exception:
            # Discovery must never raise: a broken host module means "absent".
            return False

    def run_deferred(self, func: Callable[..., Any], *args: Any, **kwargs: Any) -> None:
        """Run *func* on the host UI thread via the delegated dispatcher."""
        self._backend().run_deferred(func, *args, **kwargs)

    def dispatcher_backend(self) -> Optional[str]:
        """The dispatcher backend this adapter delegates to."""
        return self.dispatcher_spec.rsplit(":", 1)[-1] if self.dispatcher_spec else None


class HostAdapterError(Exception):
    """Raised when a built-in adapter is misconfigured."""


class StandaloneHostAdapter(DispatcherBackedHostAdapter):
    """No host: AuroraView owns its own window and event loop.

    Always detected, so it is the guaranteed terminus of host discovery.
    """

    dispatcher_spec = (
        "auroraview.utils.thread_dispatcher.backends.fallback:FallbackDispatcherBackend"
    )

    id = "standalone"
    display_name = "Standalone (no host)"
    ui_framework = UiFramework.UNKNOWN
    thread_model = ThreadModel.ANY
    embed_mode = EmbedMode.FLOATING

    def detect(self) -> bool:
        """Always available: this is the fallback host."""
        return True

    def capabilities(self) -> int:
        """Standalone owns its event loop, so nothing needs marshalling."""
        return Feature.MULTI_WINDOW | Feature.OUT_OF_PROCESS


class QtHostAdapterMixin(object):
    """Shared metadata for Qt-flavoured DCC hosts."""

    ui_framework = UiFramework.QT
    thread_model = ThreadModel.HOST_UI_THREAD
    embed_mode = EmbedMode.NATIVE_CHILD

    def capabilities(self) -> int:
        """Qt hosts can be embedded as a native child and can dispatch."""
        return (
            Feature.NATIVE_EMBEDDING
            | Feature.MAIN_THREAD_DISPATCH
            | Feature.MULTI_WINDOW
            | Feature.OUT_OF_PROCESS
        )


class MayaHostAdapter(QtHostAdapterMixin, DispatcherBackedHostAdapter):
    """Autodesk Maya (Qt host)."""

    dispatcher_spec = "auroraview.utils.thread_dispatcher.backends.maya:MayaDispatcherBackend"
    id = "maya"
    display_name = "Autodesk Maya"

    def version(self) -> Optional[str]:
        """Maya version, via ``maya.cmds.about``."""
        try:
            from maya import cmds  # type: ignore[import-not-found]

            return str(cmds.about(version=True))
        except Exception:
            return None


class HoudiniHostAdapter(QtHostAdapterMixin, DispatcherBackedHostAdapter):
    """SideFX Houdini (Qt host)."""

    dispatcher_spec = "auroraview.utils.thread_dispatcher.backends.houdini:HoudiniDispatcherBackend"
    id = "houdini"
    display_name = "SideFX Houdini"


class NukeHostAdapter(QtHostAdapterMixin, DispatcherBackedHostAdapter):
    """Foundry Nuke (Qt host)."""

    dispatcher_spec = "auroraview.utils.thread_dispatcher.backends.nuke:NukeDispatcherBackend"
    id = "nuke"
    display_name = "Foundry Nuke"


class MaxHostAdapter(QtHostAdapterMixin, DispatcherBackedHostAdapter):
    """Autodesk 3ds Max (Qt via PySide host)."""

    dispatcher_spec = "auroraview.utils.thread_dispatcher.backends.max:MaxDispatcherBackend"
    id = "max"
    display_name = "Autodesk 3ds Max"


class BlenderHostAdapter(DispatcherBackedHostAdapter):
    """Blender -- not Qt-based, so embedding is floating/out-of-process."""

    dispatcher_spec = "auroraview.utils.thread_dispatcher.backends.blender:BlenderDispatcherBackend"
    id = "blender"
    display_name = "Blender"
    ui_framework = UiFramework.UNKNOWN
    thread_model = ThreadModel.HOST_UI_THREAD
    embed_mode = EmbedMode.FLOATING

    def capabilities(self) -> int:
        """Blender exposes no native parent handle through this contract."""
        return Feature.MAIN_THREAD_DISPATCH | Feature.OUT_OF_PROCESS

    def version(self) -> Optional[str]:
        """Blender version string."""
        try:
            import bpy  # type: ignore[import-not-found]

            return str(bpy.app.version_string)
        except Exception:
            return None


class UnrealHostAdapter(DispatcherBackedHostAdapter):
    """Unreal Engine -- Slate UI, GameThread affinity.

    Note the thread model: blocking the GameThread stalls the editor, so
    :meth:`~auroraview.adapter.base.HostAdapter.try_run_sync` refuses by
    default.
    """

    dispatcher_spec = "auroraview.utils.thread_dispatcher.backends.unreal:UnrealDispatcherBackend"
    id = "unreal"
    display_name = "Unreal Engine"
    ui_framework = UiFramework.SLATE
    thread_model = ThreadModel.GAME_THREAD
    embed_mode = EmbedMode.NATIVE_CHILD

    def capabilities(self) -> int:
        """Unreal can host a native child surface; dispatch is GameThread-bound."""
        return (
            Feature.NATIVE_EMBEDDING
            | Feature.MAIN_THREAD_DISPATCH
            | Feature.MULTI_WINDOW
            | Feature.OUT_OF_PROCESS
        )


class PowerPointHostAdapter(DispatcherBackedHostAdapter):
    """Microsoft PowerPoint -- COM STA apartment.

    The strictest host we model. STA means the host thread must never block, so
    embedding is out-of-process only and
    :meth:`~auroraview.adapter.base.HostAdapter.try_run_sync` refuses.

    There is no dispatcher backend for PowerPoint yet, so this adapter is
    **registered but never detected** -- it documents the target shape and is
    the template for the first non-Python host.
    """

    dispatcher_spec = ""
    id = "powerpoint"
    display_name = "Microsoft PowerPoint"
    ui_framework = UiFramework.WIN32
    thread_model = ThreadModel.STA_APARTMENT
    embed_mode = EmbedMode.OUT_OF_PROCESS

    def detect(self) -> bool:
        """Not detectable yet: no PowerPoint dispatcher backend exists."""
        return False

    def run_deferred(
        self, func: Callable[..., Any], *args: Any, **kwargs: Any
    ) -> None:  # pragma: no cover - no backend yet
        """Not implemented: STA dispatch needs a COM-aware backend."""
        raise NotImplementedError(
            "PowerPoint dispatch requires a COM STA dispatcher backend; run out-of-process instead"
        )

    def capabilities(self) -> int:
        """Only out-of-process embedding is safe for an STA host."""
        return Feature.OUT_OF_PROCESS

    def probe(self, feature: int) -> CapabilitySupport:
        """Report out-of-process as the one supported path."""
        if feature == Feature.NATIVE_EMBEDDING:
            return CapabilitySupport.unsupported(
                "an STA apartment must not host a foreign window child",
                "run out-of-process and pass --parent-hwnd",
            )
        return super(PowerPointHostAdapter, self).probe(feature)
