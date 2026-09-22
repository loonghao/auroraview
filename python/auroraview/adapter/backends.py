# -*- coding: utf-8 -*-
"""Pluggable render-backend contract.

Two engine families must be reachable through the same contract:

* :attr:`BackendFamily.NATIVE` -- the platform webview (WebView2 on Windows,
  WKWebView on macOS, WebKitGTK on Linux). Small, no bundled runtime.
* :attr:`BackendFamily.CHROMIUM` -- a bundled Chromium-class engine. Heavy, but
  the only path that gives a real CDP endpoint, DevTools protocol parity and
  identical rendering across platforms.

Python mirror of ``auroraview_contract::backend``.

Relationship to ``auroraview.core.backend``
-------------------------------------------

``auroraview.core.backend`` already exposes a ``BackendType`` enum and a
``BackendFactory``. That factory is a closed ``if/elif`` over the enum: adding a
backend means editing core, and on Windows ``BackendType.WebView2`` returns the
same implementation as ``BackendType.Wry``. This module keeps the idea and
replaces the closed factory with a registry, so a backend becomes a
registration rather than a core edit.

Probing never raises
--------------------

:meth:`RenderBackend.probe` returns a :class:`~.capability.CapabilitySupport`.
A backend that is not installed reports ``UNSUPPORTED`` **with the install or
build step that would enable it**, rather than raising or returning an opaque
internal error.
"""

from __future__ import annotations

import os
from abc import ABC, abstractmethod
from typing import Any, Dict, List, Optional, Tuple

from .capability import CapabilityReport, CapabilitySupport, Feature
from .registry import Registry, Selection

__all__ = [
    "ENV_BACKEND",
    "BackendFamily",
    "BackendError",
    "SurfaceSpec",
    "RenderSurface",
    "RenderBackend",
    "BackendRegistry",
    "NativeWebviewBackend",
    "ChromiumBackend",
    "get_backend_registry",
    "select_backend",
    "probe_backend",
]

#: Environment variable used to pin a render backend.
ENV_BACKEND = "AURORAVIEW_BACKEND"


class BackendFamily(object):
    """Engine family behind a backend."""

    #: The platform's own webview control.
    NATIVE = "native"
    #: A bundled Chromium-class engine.
    CHROMIUM = "chromium"
    #: Anything else (an embedded browser supplied by an adapter).
    OTHER = "other"


class BackendError(Exception):
    """A backend operation that could not be performed."""

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
        super(BackendError, self).__init__(message)
        self.operation = operation
        self.reason = reason
        self.how_to_enable = how_to_enable


class SurfaceSpec(object):
    """Description of a surface to create."""

    __slots__ = (
        "title",
        "width",
        "height",
        "url",
        "html",
        "parent_handle",
        "transparent",
        "devtools",
    )

    def __init__(
        self,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        url: Optional[str] = None,
        html: Optional[str] = None,
        parent_handle: Optional[int] = None,
        transparent: bool = False,
        devtools: bool = True,
    ) -> None:
        self.title = title
        self.width = width
        self.height = height
        self.url = url
        self.html = html
        self.parent_handle = parent_handle
        self.transparent = transparent
        self.devtools = devtools

    def with_parent_handle(self, handle: int) -> "SurfaceSpec":
        """Return a copy attached to a host window."""
        return SurfaceSpec(
            title=self.title,
            width=self.width,
            height=self.height,
            url=self.url,
            html=self.html,
            parent_handle=handle,
            transparent=self.transparent,
            devtools=self.devtools,
        )

    def __repr__(self) -> str:
        return "SurfaceSpec(title={!r}, url={!r}, parent={!r})".format(
            self.title, self.url, self.parent_handle
        )


class RenderSurface(ABC):
    """A live web surface created by a :class:`RenderBackend`."""

    @abstractmethod
    def navigate(self, url: str) -> None:
        """Navigate to *url*."""
        raise NotImplementedError

    @abstractmethod
    def eval_js(self, script: str) -> None:
        """Evaluate *script*, discarding the result."""
        raise NotImplementedError

    @abstractmethod
    def set_bounds(self, x: int, y: int, width: int, height: int) -> None:
        """Move and resize the surface."""
        raise NotImplementedError

    @abstractmethod
    def set_visible(self, visible: bool) -> None:
        """Show or hide the surface."""
        raise NotImplementedError

    @abstractmethod
    def native_handle(self) -> Optional[int]:
        """Return the native window handle once the surface is realised."""
        raise NotImplementedError

    @abstractmethod
    def process_events(self) -> bool:
        """Pump one round of events.

        Returns:
            ``True`` while the surface is alive, ``False`` once closed.
        """
        raise NotImplementedError

    @abstractmethod
    def close(self) -> None:
        """Tear the surface down."""
        raise NotImplementedError


class RenderBackend(ABC):
    """The render-backend contract."""

    # ------------------------------------------------------------------ identity

    @property
    @abstractmethod
    def id(self) -> str:
        """Stable machine-readable id: ``"native"``, ``"chromium"``."""
        raise NotImplementedError

    @property
    @abstractmethod
    def display_name(self) -> str:
        """Human readable name."""
        raise NotImplementedError

    @property
    @abstractmethod
    def family(self) -> str:
        """Engine family: one of the :class:`BackendFamily` constants."""
        raise NotImplementedError

    # --------------------------------------------------------------- availability

    @abstractmethod
    def available(self) -> bool:
        """Whether this backend can create surfaces here.

        Cheap and non-raising: reports presence, creates nothing.
        """
        raise NotImplementedError

    @abstractmethod
    def missing_requirement(self) -> Optional[str]:
        """What is missing when :meth:`available` is ``False``."""
        raise NotImplementedError

    @abstractmethod
    def capabilities(self) -> int:
        """Capabilities declared when available (a :class:`Feature` bit set)."""
        raise NotImplementedError

    # ---------------------------------------------------------------- creation

    @abstractmethod
    def create_surface(self, spec: SurfaceSpec) -> RenderSurface:
        """Create a surface.

        Raises:
            BackendError: with a remediation hint when the backend is registered
                but not usable. Never raises a bare internal error.
        """
        raise NotImplementedError

    # --------------------------------------------------------------- capability

    def probe(self, feature: int) -> CapabilitySupport:
        """Probe one capability. Never raises.

        An unavailable backend cannot answer anything -- not even ``unknown`` --
        so every answer from it is a structured "unsupported, and here is what
        would enable it".
        """
        if not self.available():
            return CapabilitySupport.unsupported(
                "backend '{}' is not available here".format(self.id),
                self.missing_requirement() or "install or link this backend",
            )
        if self.capabilities() & feature == feature:
            return CapabilitySupport.supported()
        return CapabilitySupport.unsupported(
            "backend '{}' does not provide {}".format(self.id, Feature.name(feature)),
            "select a different backend via {}".format(ENV_BACKEND),
        )

    def report(self) -> CapabilityReport:
        """Build a full capability report across every declared feature."""
        report = CapabilityReport(self.id)
        for flag in Feature.flags():
            report.push(flag, self.probe(flag))
        return report

    def __repr__(self) -> str:
        return "<{} id={!r} available={!r}>".format(type(self).__name__, self.id, self.available())


class NativeWebviewBackend(RenderBackend):
    """The platform webview: WebView2 / WKWebView / WebKitGTK.

    Declared here so both families are visible in one place. This is the path
    AuroraView uses today.
    """

    id = "native"
    display_name = "Platform WebView (WebView2 / WKWebView / WebKitGTK)"
    family = BackendFamily.NATIVE

    def available(self) -> bool:
        """Available whenever the compiled ``auroraview._core`` is importable."""
        try:
            from auroraview import _core  # type: ignore[import-not-found]

            return _core is not None
        except Exception:
            return False

    def missing_requirement(self) -> Optional[str]:
        """Nothing is missing: this backend ships with the wheel."""
        return None

    def capabilities(self) -> int:
        """Capabilities of the native path."""
        return (
            Feature.NATIVE_EMBEDDING
            | Feature.OUT_OF_PROCESS
            | Feature.DEVTOOLS
            | Feature.JS_EVAL_RESULT
            | Feature.COOKIES
            | Feature.FILE_PROTOCOL
            | Feature.MULTI_WINDOW
        )

    def probe(self, feature: int) -> CapabilitySupport:
        """Probe, with two features answered specially."""
        if not self.available():
            return CapabilitySupport.unsupported(
                "the native backend needs the compiled auroraview._core extension",
                "install a released auroraview wheel for this interpreter",
            )
        if feature == Feature.CDP:
            return CapabilitySupport.unsupported(
                "the platform webview exposes no CDP endpoint",
                "select the 'chromium' backend ({}=chromium)".format(ENV_BACKEND),
            )
        if feature == Feature.TRANSPARENCY:
            return CapabilitySupport.unknown(
                "transparency depends on the platform compositor; verify at runtime"
            )
        return super(NativeWebviewBackend, self).probe(feature)

    def create_surface(self, spec: SurfaceSpec) -> RenderSurface:
        """Create a native surface.

        Delegated to the existing WebView implementation; this prototype does not
        replace it.
        """
        raise BackendError(
            "create_surface",
            "backend selection is wired, surface creation still goes through auroraview.core",
            "use auroraview.create_webview() until the backend registry owns construction",
        )


class ChromiumBackend(RenderBackend):
    """A bundled Chromium-class backend.

    Present so the second path is a first-class citizen of the contract rather
    than a TODO. It is **not available** in any current build, which is exactly
    the situation the capability contract exists to describe: :meth:`available`
    is ``False`` and every probe returns a structured ``UNSUPPORTED`` carrying
    the step that would enable it.
    """

    id = "chromium"
    display_name = "Chromium (bundled engine, CDP-capable)"
    family = BackendFamily.CHROMIUM

    def available(self) -> bool:
        """Not available: no Chromium engine is bundled today."""
        return False

    def missing_requirement(self) -> Optional[str]:
        """How to enable the Chromium path."""
        return (
            "the Chromium backend is not installed; add the 'chromium' extra "
            "(pip install 'auroraview[chromium]') once it is published"
        )

    def capabilities(self) -> int:
        """Capabilities the Chromium path would provide once available."""
        return (
            Feature.CDP
            | Feature.DEVTOOLS
            | Feature.JS_EVAL_RESULT
            | Feature.COOKIES
            | Feature.MULTI_WINDOW
            | Feature.OUT_OF_PROCESS
        )

    def create_surface(self, spec: SurfaceSpec) -> RenderSurface:
        """Refuse with a remediation hint instead of an opaque failure."""
        raise BackendError(
            "create_surface",
            "the chromium backend is not installed",
            self.missing_requirement() or "",
        )


def _contract_id(candidate: Any) -> str:
    """Return the contract ``id`` of a class, or "" when it has none."""
    if isinstance(candidate, type):
        value = getattr(candidate, "id", None)
        if isinstance(value, str):
            return value
    return ""


class BackendRegistry(object):
    """Registry of render backends.

    Wraps the shared :class:`~.registry.Registry` so backends use exactly the
    same priority / lazy-spec / env-override mechanics as host adapters.
    """

    def __init__(self) -> None:
        self._registry: Registry = Registry(base=RenderBackend, kind="backend")
        #: Priority of the native backend (higher is tried first).
        self.native_priority = 100
        #: Priority of the chromium backend.
        self.chromium_priority = 50

    def register(self, backend: Any, priority: int = 0, name: str = "") -> None:
        """Register a backend class, factory, or ``"module:ClassName"`` spec.

        When *name* is omitted it defaults to the backend's contract ``id``, so
        ``AURORAVIEW_BACKEND=native`` matches the registered name.
        """
        if not name:
            name = _contract_id(backend)
        self._registry.register(backend, priority=priority, name=name)

    def unregister(self, name: str) -> bool:
        """Remove a backend by name."""
        return self._registry.unregister(name)

    def names(self) -> List[str]:
        """Registered backend names in priority order."""
        return self._registry.names()

    def __len__(self) -> int:
        return len(self._registry)

    def list(self) -> List[Tuple[int, str, bool]]:
        """``(priority, name, available)`` for every registered backend."""
        result: List[Tuple[int, str, bool]] = []
        for priority, _spec, name in self._registry.entries:
            backend = self._registry.build(name)
            result.append((priority, name, bool(backend and backend.available())))
        return result

    def select(self, env_override: Optional[str] = None) -> Optional[Selection]:
        """Select the best available backend, honouring an override."""
        if env_override is None:
            env_override = os.environ.get(ENV_BACKEND, "")
        return self._registry.select(lambda backend: backend.available(), env_override=env_override)

    def report(self, name: str) -> Optional[CapabilityReport]:
        """Capability report for one backend, or ``None`` if not registered."""
        backend = self._registry.build(name)
        if backend is None:
            return None
        return backend.report()

    def report_all(self) -> Dict[str, CapabilityReport]:
        """Capability report for every registered backend."""
        reports: Dict[str, CapabilityReport] = {}
        for name in self._registry.names():
            report = self.report(name)
            if report is not None:
                reports[name] = report
        return reports


_REGISTRY: Optional[BackendRegistry] = None


def get_backend_registry() -> BackendRegistry:
    """Return the process-wide backend registry, preloaded on first use."""
    global _REGISTRY
    if _REGISTRY is None:
        registry = BackendRegistry()
        registry.register(NativeWebviewBackend, priority=registry.native_priority)
        registry.register(ChromiumBackend, priority=registry.chromium_priority)
        _REGISTRY = registry
    return _REGISTRY


def select_backend(env_override: Optional[str] = None) -> Optional[Selection]:
    """Select the best available render backend.

    Returns:
        A :class:`~.registry.Selection`, or ``None`` if no backend is usable.
    """
    return get_backend_registry().select(env_override=env_override)


def probe_backend(name: str) -> Optional[CapabilityReport]:
    """Return the capability report for a registered backend."""
    return get_backend_registry().report(name)
