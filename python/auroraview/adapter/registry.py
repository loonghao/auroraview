# -*- coding: utf-8 -*-
"""Priority registry shared by the host-adapter and render-backend contracts.

This is a thin generalisation of
``auroraview.utils.thread_dispatcher.registry``. The mechanics are deliberately
identical so there is **one** extension idiom in AuroraView, not two:

* candidates carry an ``int`` priority, highest first
* candidates may be given as a ``"module:ClassName"`` string and are imported
  lazily, only when probed or selected
* an environment variable can pin one candidate by name
* a failed override degrades to priority order **and records a warning** -- it
  does not abort selection

Backends are registered as specs, not instances, so importing
``auroraview.adapter`` never imports a host SDK or a Qt binding.
"""

from __future__ import annotations

import importlib
import logging
from typing import Any, Callable, List, Optional, Tuple, Type, TypeVar, Union

logger = logging.getLogger(__name__)

T = TypeVar("T")

#: A candidate spec: either a class/factory, or a ``"module:ClassName"`` string.
Spec = Union[Callable[[], Any], Type[Any], str]

__all__ = ["Registry", "Selection", "load_spec"]


def load_spec(spec: Spec, base: Optional[Type[Any]] = None) -> Optional[Any]:
    """Resolve a spec to a class or factory.

    Args:
        spec: A class/factory, or a ``"module:ClassName"`` string.
        base: Optional base class the resolved object must subclass.

    Returns:
        The resolved object, or ``None`` when it cannot be imported or does not
        satisfy *base*. Never raises: a broken third-party spec must not take
        down host detection.
    """
    if not isinstance(spec, str):
        if base is not None and isinstance(spec, type) and not issubclass(spec, base):
            logger.warning("Spec %r is not a %s subclass", spec, base.__name__)
            return None
        return spec

    if ":" not in spec:
        logger.warning("Invalid spec (expected 'module:ClassName'): %s", spec)
        return None

    module_path, class_name = spec.rsplit(":", 1)

    try:
        module = importlib.import_module(module_path)
        resolved = getattr(module, class_name)
    except ImportError as exc:
        logger.debug("Could not import %s: %s", module_path, exc)
        return None
    except AttributeError as exc:
        logger.debug("Could not find %s in %s: %s", class_name, module_path, exc)
        return None

    if base is not None and isinstance(resolved, type) and not issubclass(resolved, base):
        logger.warning("%s is not a %s subclass", spec, base.__name__)
        return None

    return resolved


class Selection(object):
    """The outcome of a registry lookup.

    Attributes:
        value: The selected candidate.
        name: Registered name of the selected candidate.
        via_env_override: Whether an environment override drove the choice.
        warnings: Non-fatal problems hit while selecting -- for example an
            override that named a candidate which turned out to be unavailable.
    """

    __slots__ = ("value", "name", "via_env_override", "warnings")

    def __init__(
        self,
        value: Any,
        name: str,
        via_env_override: bool = False,
        warnings: Optional[List[str]] = None,
    ) -> None:
        self.value = value
        self.name = name
        self.via_env_override = via_env_override
        self.warnings = warnings or []

    @property
    def has_warnings(self) -> bool:
        """Whether any warning was recorded."""
        return bool(self.warnings)

    def __repr__(self) -> str:
        return "Selection(name={!r}, via_env_override={!r}, warnings={!r})".format(
            self.name, self.via_env_override, self.warnings
        )


class Registry(object):
    """A priority-ordered registry of lazily constructed candidates.

    Entries are kept sorted by descending priority. Python's sort is stable, so
    entries registered with equal priority are tried in registration order.
    """

    def __init__(self, base: Optional[Type[Any]] = None, kind: str = "candidate") -> None:
        self._base = base
        self._kind = kind
        self._entries: List[Tuple[int, Spec, str]] = []

    # ------------------------------------------------------------- registration

    def register(self, spec: Spec, priority: int = 0, name: str = "") -> None:
        """Register a candidate.

        Args:
            spec: A class, a zero-argument factory, or a ``"module:ClassName"``
                string resolved lazily.
            priority: Higher is tried first.
            name: Display name; defaults to the class name with a trailing
                ``Backend``/``Adapter`` suffix stripped.
        """
        display = name or self._spec_name(spec)
        # A name identifies a slot. Registering the same name twice replaces the
        # previous entry rather than appending a duplicate: two entries sharing a
        # name would make `unregister` ambiguous (it removes the first match
        # only) and would let a stale candidate shadow a live one.
        for index, (_priority, existing, existing_name) in enumerate(self._entries):
            if existing_name == display or existing is spec or existing == spec:
                self._entries[index] = (priority, spec, display)
                self._sort()
                return
        self._entries.append((priority, spec, display))
        self._sort()

    def unregister(self, name: str) -> bool:
        """Remove a candidate by name. Returns whether it was present."""
        for index, (_priority, _spec, display) in enumerate(self._entries):
            if display == name:
                self._entries.pop(index)
                return True
        return False

    def clear(self) -> None:
        """Remove every candidate."""
        self._entries = []

    # ------------------------------------------------------------------ reading

    @property
    def entries(self) -> List[Tuple[int, Spec, str]]:
        """``(priority, spec, name)`` tuples in priority order."""
        return list(self._entries)

    def names(self) -> List[str]:
        """Registered names in priority order."""
        return [name for _priority, _spec, name in self._entries]

    def __len__(self) -> int:
        return len(self._entries)

    def __bool__(self) -> bool:
        return bool(self._entries)

    def build(self, name: str) -> Optional[Any]:
        """Instantiate a candidate by name, bypassing availability checks."""
        for _priority, spec, display in self._entries:
            if display == name:
                return self._instantiate(spec, display)
        return None

    # ---------------------------------------------------------------- selection

    def select(
        self,
        predicate: Callable[[Any], bool],
        env_override: Optional[str] = None,
    ) -> Optional[Selection]:
        """Select the first candidate (priority order) satisfying *predicate*.

        When *env_override* names a registered candidate it is tried first. If it
        is unknown or fails *predicate*, selection falls through to priority
        order and a warning is recorded on the returned :class:`Selection` -- an
        unusable override never becomes a hard error.

        A candidate that *raises* while being probed is treated as unusable and
        recorded as a warning rather than propagated: discovery is shared
        infrastructure, and one broken third-party adapter must not take down
        host detection for everyone.
        """
        warnings: List[str] = []
        probed_name: Optional[str] = None

        def check(value: Any, display: str) -> bool:
            """Run the predicate, treating a raising candidate as unusable."""
            try:
                return bool(predicate(value))
            except Exception as exc:
                warnings.append("'{}' raised during selection: {}".format(display, exc))
                return False

        if env_override:
            requested = env_override.strip().lower()
            if requested:
                match = None
                for _priority, spec, display in self._entries:
                    if display.lower() == requested:
                        match = (spec, display)
                        break

                if match is None:
                    warnings.append(
                        "'{}' was requested by override but is not registered".format(
                            env_override.strip()
                        )
                    )
                else:
                    spec, display = match
                    probed_name = display
                    value = self._instantiate(spec, display)
                    if value is not None and check(value, display):
                        return Selection(value, display, True, warnings)
                    warnings.append(
                        "'{}' was requested by override but is not available; "
                        "falling back to priority order".format(display)
                    )

        # The override candidate was already built and probed above; the
        # priority pass must not build it a second time.
        for _priority, spec, display in self._entries:
            if display == probed_name:
                continue
            value = self._instantiate(spec, display)
            if value is not None and check(value, display):
                return Selection(value, display, False, warnings)

        return None

    # ----------------------------------------------------------------- internals

    def _sort(self) -> None:
        self._entries.sort(key=lambda entry: entry[0], reverse=True)

    def _instantiate(self, spec: Spec, display: str) -> Optional[Any]:
        resolved = load_spec(spec, self._base)
        if resolved is None:
            return None
        try:
            return resolved()
        except Exception as exc:  # pragma: no cover - defensive
            logger.warning("Failed to instantiate %s %r: %s", self._kind, display, exc)
            return None

    @staticmethod
    def _spec_name(spec: Spec) -> str:
        if isinstance(spec, str):
            name = spec.rsplit(":", 1)[-1]
        else:
            name = getattr(spec, "__name__", str(spec))
        for suffix in ("Backend", "Adapter"):
            if name.endswith(suffix) and len(name) > len(suffix):
                name = name[: -len(suffix)]
        return name
