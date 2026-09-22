# -*- coding: utf-8 -*-
"""Capability probing for host adapters and render backends.

Python mirror of ``auroraview_contract::capability``.

**The rule this module exists to enforce**: probing a capability never raises.
A missing feature is data, not an exception::

    probe(feature) -> CapabilitySupport   # always returns
    support.require(...)                  # the single raising call

Every ``UNSUPPORTED`` answer carries both a reason and a ``how_to_enable``
hint so an integrator can act on it without reading AuroraView source.
``UNKNOWN`` is a first-class answer -- it is how an implementation says "this
depends on runtime state, verify at runtime" without guessing.
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional

__all__ = [
    "Feature",
    "CapabilitySupport",
    "CapabilityError",
    "CapabilityReport",
]


class Feature:
    """Optional capabilities declared by an adapter or a backend.

    Bit flags, combined with ``|``. Values match the Rust ``Features`` bits in
    ``auroraview_contract::capability`` so the two sides can report identically.
    """

    #: The surface can be parented into a host-provided native window.
    NATIVE_EMBEDDING = 1 << 0
    #: The surface can run detached in its own process.
    OUT_OF_PROCESS = 1 << 1
    #: DevTools / inspector is reachable.
    DEVTOOLS = 1 << 2
    #: Chrome DevTools Protocol endpoint is exposed.
    CDP = 1 << 3
    #: Transparent (per-pixel alpha) surfaces.
    TRANSPARENCY = 1 << 4
    #: ``eval_js`` can return the script result to the caller.
    JS_EVAL_RESULT = 1 << 5
    #: Cookie read / write / clear.
    COOKIES = 1 << 6
    #: Custom ``file://`` (or equivalent) protocol handler.
    FILE_PROTOCOL = 1 << 7
    #: The host can marshal work onto its own UI thread.
    MAIN_THREAD_DISPATCH = 1 << 8
    #: More than one surface per process.
    MULTI_WINDOW = 1 << 9

    #: Every declared capability, in declaration order.
    ALL = (
        NATIVE_EMBEDDING
        | OUT_OF_PROCESS
        | DEVTOOLS
        | CDP
        | TRANSPARENCY
        | JS_EVAL_RESULT
        | COOKIES
        | FILE_PROTOCOL
        | MAIN_THREAD_DISPATCH
        | MULTI_WINDOW
    )

    _NAMES = {
        NATIVE_EMBEDDING: "NATIVE_EMBEDDING",
        OUT_OF_PROCESS: "OUT_OF_PROCESS",
        DEVTOOLS: "DEVTOOLS",
        CDP: "CDP",
        TRANSPARENCY: "TRANSPARENCY",
        JS_EVAL_RESULT: "JS_EVAL_RESULT",
        COOKIES: "COOKIES",
        FILE_PROTOCOL: "FILE_PROTOCOL",
        MAIN_THREAD_DISPATCH: "MAIN_THREAD_DISPATCH",
        MULTI_WINDOW: "MULTI_WINDOW",
    }

    @classmethod
    def flags(cls) -> List[int]:
        """Return one entry per declared capability, in declaration order."""
        return list(cls._NAMES)

    @classmethod
    def names(cls, features: int) -> List[str]:
        """Return the names of every capability set in *features*."""
        return [name for flag, name in cls._NAMES.items() if features & flag == flag]

    @classmethod
    def name(cls, feature: int) -> str:
        """Return the name of a single-bit capability set."""
        return cls._NAMES.get(feature, "MULTI_FLAG")


class CapabilitySupport:
    """The answer to "do you support this capability?".

    Never an exception. Use :meth:`require` for the one and only fallible
    conversion.
    """

    SUPPORTED = "supported"
    UNSUPPORTED = "unsupported"
    UNKNOWN = "unknown"

    __slots__ = ("state", "reason", "how_to_enable")

    def __init__(
        self,
        state: str,
        reason: Optional[str] = None,
        how_to_enable: Optional[str] = None,
    ) -> None:
        self.state = state
        self.reason = reason
        self.how_to_enable = how_to_enable

    # ---------------------------------------------------------------- builders

    @classmethod
    def supported(cls) -> "CapabilitySupport":
        """Return a ``supported`` answer."""
        return cls(cls.SUPPORTED)

    @classmethod
    def unsupported(cls, reason: str, how_to_enable: str) -> "CapabilitySupport":
        """Return an ``unsupported`` answer carrying a remediation hint."""
        return cls(cls.UNSUPPORTED, reason, how_to_enable)

    @classmethod
    def unknown(cls, reason: str) -> "CapabilitySupport":
        """Return an ``unknown`` answer (decide at runtime, do not guess)."""
        return cls(cls.UNKNOWN, reason)

    # ------------------------------------------------------------------ access

    @property
    def is_supported(self) -> bool:
        """Whether this answer is ``supported``."""
        return self.state == self.SUPPORTED

    @property
    def is_unknown(self) -> bool:
        """Whether this answer is ``unknown``."""
        return self.state == self.UNKNOWN

    def require(self, subject: str, feature: str) -> None:
        """Raise :class:`CapabilityError` unless the capability is supported.

        This is the only place a probe turns into an exception. Callers that can
        degrade gracefully should keep the :class:`CapabilitySupport` value.
        """
        if not self.is_supported:
            raise CapabilityError(subject, feature, self)

    def as_dict(self) -> Dict[str, Any]:
        """Return a JSON-serialisable view of this answer."""
        return {
            "state": self.state,
            "reason": self.reason,
            "how_to_enable": self.how_to_enable,
        }

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, CapabilitySupport):
            return NotImplemented
        return (
            self.state == other.state
            and self.reason == other.reason
            and self.how_to_enable == other.how_to_enable
        )

    def __hash__(self) -> int:
        return hash((self.state, self.reason, self.how_to_enable))

    def __repr__(self) -> str:
        if self.state == self.SUPPORTED:
            return "CapabilitySupport.supported()"
        if self.state == self.UNKNOWN:
            return "CapabilitySupport.unknown({!r})".format(self.reason)
        return "CapabilitySupport.unsupported({!r}, {!r})".format(self.reason, self.how_to_enable)

    def __str__(self) -> str:
        if self.state == self.SUPPORTED:
            return "supported"
        if self.state == self.UNKNOWN:
            return "unknown: {}".format(self.reason)
        return "unsupported: {} (to enable: {})".format(self.reason, self.how_to_enable)


class CapabilityError(Exception):
    """Raised by :meth:`CapabilitySupport.require`."""

    def __init__(self, subject: str, feature: str, support: "CapabilitySupport") -> None:
        super(CapabilityError, self).__init__(
            "'{}' cannot provide {}: {}".format(subject, feature, support)
        )
        self.subject = subject
        self.feature = feature
        self.support = support


class CapabilityReport:
    """A complete, human readable capability snapshot."""

    def __init__(self, subject: str) -> None:
        self.subject = subject
        self.entries: Dict[str, CapabilitySupport] = {}

    def push(self, feature: int, support: CapabilitySupport) -> None:
        """Append one row."""
        self.entries[Feature.name(feature)] = support

    def get(self, feature: str) -> Optional[CapabilitySupport]:
        """Return the answer for *feature*, or ``None`` if it was not probed."""
        return self.entries.get(feature)

    def supported(self) -> List[str]:
        """Names of every supported capability."""
        return [name for name, support in self.entries.items() if support.is_supported]

    def unsupported(self) -> List[str]:
        """Names of every unsupported capability."""
        return [
            name
            for name, support in self.entries.items()
            if support.state == CapabilitySupport.UNSUPPORTED
        ]

    def unknown(self) -> List[str]:
        """Names of every capability whose answer is unknown."""
        return [name for name, support in self.entries.items() if support.is_unknown]

    def as_dict(self) -> Dict[str, Any]:
        """Return a JSON-serialisable view of the whole report."""
        return {
            "subject": self.subject,
            "supported": self.supported(),
            "unsupported": self.unsupported(),
            "unknown": self.unknown(),
            "entries": {name: support.as_dict() for name, support in self.entries.items()},
        }

    def render(self) -> str:
        """Render a multi-line report."""
        lines = ["capability report: {}".format(self.subject)]
        lines.append("  supported   : {}".format(", ".join(self.supported())))

        unsupported = self.unsupported()
        if unsupported:
            lines.append("  unsupported :")
            for name in unsupported:
                lines.append("    - {}: {}".format(name, self.entries[name]))
        else:
            lines.append("  unsupported : none")

        unknown = self.unknown()
        if unknown:
            lines.append("  unknown     :")
            for name in unknown:
                lines.append("    - {}: {}".format(name, self.entries[name]))

        return "\n".join(lines)

    def __repr__(self) -> str:
        return "CapabilityReport({!r}, {} entries)".format(self.subject, len(self.entries))
