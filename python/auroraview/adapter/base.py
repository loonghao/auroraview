# -*- coding: utf-8 -*-
"""Host adapter contract.

A *host adapter* is everything AuroraView needs to know about a host
application (Maya, Unreal, Unity, PowerPoint, a browser, plain desktop). It
answers the three questions the package split hinges on:

* **how do I register myself** -- :func:`auroraview.adapter.register_host_adapter`
* **how am I discovered** -- :meth:`HostAdapter.detect` via :class:`HostRegistry`
* **how does a surface get embedded** -- :attr:`HostAdapter.embed_mode` plus
  :meth:`HostAdapter.parent_handle`

It is also the only place host-specific code is allowed to live.

Dependency direction
--------------------

``auroraview`` (core) must never import a host module. Adapters import core;
core discovers them through this registry. A new host is therefore a new module
that registers itself, not an edit to a ``DccType`` enum in core.

Relationship to the thread dispatcher
-------------------------------------

:mod:`auroraview.utils.thread_dispatcher` already solves *thread dispatch* and
already has a real plugin registry. This contract does **not** replace it: the
built-in adapters delegate :meth:`HostAdapter.run_deferred` to the matching
dispatcher backend. The dispatcher keeps owning threading; the host adapter adds
identity, discovery and embedding.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any, Callable, Optional, TypeVar

from .capability import CapabilityReport, CapabilitySupport, Feature

T = TypeVar("T")

__all__ = [
    "UiFramework",
    "ThreadModel",
    "EmbedMode",
    "HostError",
    "HostInfo",
    "HostAdapter",
]


class UiFramework(object):
    """UI toolkit the host is built on.

    This decides how a surface is parented, not how it is rendered.
    """

    QT = "qt"
    SLATE = "slate"
    WIN32 = "win32"
    COCOA = "cocoa"
    GTK = "gtk"
    WPF = "wpf"
    #: The host is itself a web surface (Office.js add-in, Electron).
    HTML = "html"
    UNKNOWN = "unknown"

    #: Frameworks that can hand out a native window handle.
    _NATIVE_HANDLE_CAPABLE = frozenset([QT, SLATE, WIN32, COCOA, GTK, WPF])

    @classmethod
    def exposes_native_handle(cls, framework: str) -> bool:
        """Whether a native window handle can be obtained from the host."""
        return framework in cls._NATIVE_HANDLE_CAPABLE


class ThreadModel(object):
    """Thread affinity the host imposes on UI work."""

    #: Qt main thread or a Win32 message thread.
    HOST_UI_THREAD = "host_ui_thread"
    #: Unreal Engine GameThread.
    GAME_THREAD = "game_thread"
    #: COM single-threaded apartment (Office / PowerPoint).
    STA_APARTMENT = "sta_apartment"
    #: No affinity -- any thread may create and drive the surface.
    ANY = "any"

    #: Models where blocking the host thread would freeze the host.
    _BLOCKING_UNSAFE = frozenset([GAME_THREAD, STA_APARTMENT])

    @classmethod
    def blocking_dispatch_is_safe(cls, model: str) -> bool:
        """Whether blocking the host thread while waiting on a result is safe.

        On an STA thread blocking the message pump makes the host report
        "not responding"; on the Unreal GameThread it stalls the editor. Both
        must use fire-and-forget dispatch and return results through an event.
        """
        return model not in cls._BLOCKING_UNSAFE


class EmbedMode(object):
    """How a surface is attached to the host window."""

    #: The surface is parented into a host-provided native window.
    NATIVE_CHILD = "native_child"
    #: The surface runs in a separate process positioned over the host window.
    #:
    #: The only mode covering hosts with no in-process embedding path
    #: (Unity, Unreal C++, PowerPoint via COM).
    OUT_OF_PROCESS = "out_of_process"
    #: The surface is a top-level window owned by AuroraView.
    FLOATING = "floating"


class HostError(Exception):
    """A host operation that could not be performed."""

    def __init__(
        self,
        operation: str,
        reason: str,
        how_to_enable: Optional[str] = None,
    ) -> None:
        if how_to_enable:
            message = "{} failed: {} (to enable: {})".format(operation, reason, how_to_enable)
        else:
            message = "{} failed: {}".format(operation, reason)
        super(HostError, self).__init__(message)
        self.operation = operation
        self.reason = reason
        self.how_to_enable = how_to_enable


class HostInfo(object):
    """A snapshot of a host adapter, safe to log and to pass around."""

    __slots__ = (
        "id",
        "display_name",
        "version",
        "ui_framework",
        "thread_model",
        "embed_mode",
        "parent_handle",
        "dispatcher_backend",
        "capabilities",
    )

    def __init__(
        self,
        id: str,  # noqa: A002 - `id` is the contract field name
        display_name: str,
        version: Optional[str] = None,
        ui_framework: str = UiFramework.UNKNOWN,
        thread_model: str = ThreadModel.ANY,
        embed_mode: str = EmbedMode.FLOATING,
        parent_handle: Optional[int] = None,
        dispatcher_backend: Optional[str] = None,
        capabilities: int = 0,
    ) -> None:
        self.id = id
        self.display_name = display_name
        self.version = version
        self.ui_framework = ui_framework
        self.thread_model = thread_model
        self.embed_mode = embed_mode
        self.parent_handle = parent_handle
        self.dispatcher_backend = dispatcher_backend
        self.capabilities = capabilities

    def as_dict(self) -> dict:
        """Return a JSON-serialisable view."""
        return {
            "id": self.id,
            "display_name": self.display_name,
            "version": self.version,
            "ui_framework": self.ui_framework,
            "thread_model": self.thread_model,
            "embed_mode": self.embed_mode,
            "parent_handle": self.parent_handle,
            "dispatcher_backend": self.dispatcher_backend,
            "capabilities": Feature.names(self.capabilities),
        }

    def __repr__(self) -> str:
        return "HostInfo(id={!r}, framework={!r}, embed={!r})".format(
            self.id, self.ui_framework, self.embed_mode
        )


class HostAdapter(ABC):
    """The host adapter contract.

    Subclasses must be **cheap to construct**: construction happens during
    discovery, before we know whether the host is present. Anything expensive
    belongs in :meth:`detect` or behind a lazy call, and every host module
    import must be function-local so importing this package never pulls in a
    host SDK.
    """

    # ------------------------------------------------------------------ identity

    @property
    @abstractmethod
    def id(self) -> str:
        """Stable machine-readable id, e.g. ``"maya"``, ``"unreal"``."""
        raise NotImplementedError

    @property
    @abstractmethod
    def display_name(self) -> str:
        """Human readable name, e.g. ``"Autodesk Maya"``."""
        raise NotImplementedError

    @property
    @abstractmethod
    def ui_framework(self) -> str:
        """UI toolkit: one of the :class:`UiFramework` constants."""
        raise NotImplementedError

    @property
    @abstractmethod
    def thread_model(self) -> str:
        """Thread affinity: one of the :class:`ThreadModel` constants."""
        raise NotImplementedError

    @property
    @abstractmethod
    def embed_mode(self) -> str:
        """Embedding strategy: one of the :class:`EmbedMode` constants."""
        raise NotImplementedError

    # ----------------------------------------------------------------- discovery

    @abstractmethod
    def detect(self) -> bool:
        """Whether this host is present in the current process.

        Must be cheap and must not raise. Host module imports belong inside this
        method, not at module scope.
        """
        raise NotImplementedError

    def version(self) -> Optional[str]:
        """Host version, when the host exposes one."""
        return None

    def capabilities(self) -> int:
        """Capabilities this host declares (a :class:`Feature` bit set)."""
        return 0

    def parent_handle(self) -> Optional[int]:
        """Native parent window handle, when the host can supply one directly.

        Returning ``None`` is normal: Qt hosts usually need a widget pointer
        from the caller rather than a top-level handle.
        """
        return None

    def dispatcher_backend(self) -> Optional[str]:
        """Name of the thread-dispatcher backend this adapter delegates to.

        Purely informational; the built-in adapters override
        :meth:`run_deferred` instead of being looked up by this name.
        """
        return None

    # ------------------------------------------------------------------ dispatch

    @abstractmethod
    def run_deferred(self, func: Callable[..., Any], *args: Any, **kwargs: Any) -> None:
        """Run *func* on the host UI thread without waiting for it.

        This is the only dispatch primitive the contract requires, and the only
        one that is safe on every thread model we support -- including STA and
        GameThread.
        """
        raise NotImplementedError

    def try_run_sync(self, func: Callable[..., T], *args: Any, **kwargs: Any) -> T:
        """Run *func* on the host UI thread and wait for the result.

        Deliberately **not** abstract. The default refuses, because blocking the
        host thread is exactly the failure mode that makes PowerPoint report
        "not responding" and stalls the Unreal editor. Adapters whose thread
        model permits blocking may override this.
        """
        model = self.thread_model
        if ThreadModel.blocking_dispatch_is_safe(model):
            raise HostError(
                "try_run_sync",
                "host '{}' does not implement blocking dispatch".format(self.id),
                "override try_run_sync() on the '{}' adapter, or use run_deferred() "
                "and surface the result through an event".format(self.id),
            )
        raise HostError(
            "try_run_sync",
            "host '{}' uses the {} thread model, where blocking dispatch would "
            "freeze the host".format(self.id, model),
            "use run_deferred() and surface the result through an event",
        )

    # --------------------------------------------------------------- capability

    def probe(self, feature: int) -> CapabilitySupport:
        """Probe a single capability. Never raises."""
        if self.capabilities() & feature == feature:
            return CapabilitySupport.supported()
        return CapabilitySupport.unsupported(
            "host '{}' does not declare {}".format(self.id, Feature.name(feature)),
            "implement the missing capability on the '{}' host adapter".format(self.id),
        )

    def report(self) -> CapabilityReport:
        """Build a full capability report across every declared feature."""
        report = CapabilityReport(self.id)
        for flag in Feature.flags():
            report.push(flag, self.probe(flag))
        return report

    # ------------------------------------------------------------------ snapshot

    def info(self) -> HostInfo:
        """Build a :class:`HostInfo` snapshot."""
        return HostInfo(
            id=self.id,
            display_name=self.display_name,
            version=self.version(),
            ui_framework=self.ui_framework,
            thread_model=self.thread_model,
            embed_mode=self.embed_mode,
            parent_handle=self.parent_handle(),
            dispatcher_backend=self.dispatcher_backend(),
            capabilities=self.capabilities(),
        )

    def __repr__(self) -> str:
        return "<{} id={!r}>".format(type(self).__name__, self.id)
