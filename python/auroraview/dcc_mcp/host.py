# -*- coding: utf-8 -*-
"""Qt host lifecycle for the AuroraView DCC-MCP integration.

:class:`AuroraViewQtHost` wires a DCC-MCP dispatcher tick onto the Qt event
loop so tool invocations are executed on the thread that owns the WebView --
never on an HTTP or Tokio worker thread.

It implements the three hooks core's ``HostAdapter`` template requires:

* :meth:`AuroraViewQtHost.is_background` -- ``True`` when no Qt event loop runs.
* :meth:`AuroraViewQtHost.attach_tick` -- registers the tick with ``QTimer``.
* :meth:`AuroraViewQtHost.detach_tick` -- idempotent teardown.

AuroraView is a Qt host, so the DCC's own event loop drives dispatch; core
never takes over the message pump.
"""

from __future__ import annotations

import logging
from typing import Any, Callable, Optional

from ._compat import require_core as _require_core

logger = logging.getLogger(__name__)

#: Default interval between dispatcher ticks while the host is idle (seconds).
DEFAULT_IDLE_INTERVAL = 0.05


def _host_adapter_base() -> Any:
    """Return core's ``HostAdapter`` base class, importing it on demand.

    Returns:
        The ``HostAdapter`` class.

    Raises:
        ImportError: If ``dcc-mcp-core`` is not installed.
    """
    _require_core()
    from dcc_mcp_core.host import HostAdapter

    return HostAdapter


class AuroraViewQtHost:
    """Drive a DCC-MCP dispatcher from the Qt event loop.

    Args:
        dispatcher: A DCC-MCP dispatcher exposing a tick entry point
            (``QueueDispatcher`` / ``BlockingDispatcher`` / any object
            satisfying the ``TickableDispatcher`` protocol).
        tick_interval_active: Seconds between ticks while work is pending.
        tick_interval_idle: Seconds between ticks while idle.
        name: Logical name used in log messages.

    Example::

        host = AuroraViewQtHost(dispatcher)
        host.start()   # ticks now run on the Qt event loop
        ...
        host.stop()
    """

    def __init__(
        self,
        dispatcher: Any,
        tick_interval_active: float = 0.0,
        tick_interval_idle: float = DEFAULT_IDLE_INTERVAL,
        name: str = "auroraview-host",
    ):
        self._base = _host_adapter_base()(
            dispatcher,
            tick_interval_active=tick_interval_active,
            tick_interval_idle=tick_interval_idle,
            name=name,
        )
        self._timer = None
        self._tick_fn: Optional[Callable[[], Any]] = None

    # ------------------------------------------------------------------
    # HostAdapter hooks
    # ------------------------------------------------------------------

    def is_background(self) -> bool:
        """Return ``True`` when no Qt event loop is running (headless mode).

        Returns:
            ``True`` when ``QCoreApplication`` is absent or no Qt binding is
            importable; ``False`` when a Qt event loop can host the tick.
        """
        try:
            from qtpy import QtCore  # type: ignore[import-not-found]
        except Exception:  # noqa: BLE001 - Qt is an optional dependency
            return True
        return QtCore.QCoreApplication.instance() is None

    def attach_tick(self, tick_fn: Callable[[], Any]) -> None:
        """Register ``tick_fn`` with a ``QTimer`` on the Qt event loop.

        Args:
            tick_fn: Zero-argument callable returning the next interval in
                seconds, or ``None`` to cancel.
        """
        from qtpy import QtCore  # type: ignore[import-not-found]

        self.detach_tick()
        timer = QtCore.QTimer()
        timer.setTimerType(QtCore.Qt.TimerType.PreciseTimer)
        timer.timeout.connect(tick_fn)
        timer.start(max(1, int(DEFAULT_IDLE_INTERVAL * 1000)))
        self._timer = timer
        self._tick_fn = tick_fn
        logger.debug("AuroraViewQtHost attached dispatcher tick to QTimer")

    def detach_tick(self) -> None:
        """Stop and drop the tick timer. Safe to call repeatedly."""
        timer = self._timer
        if timer is None:
            return
        try:
            timer.stop()
        except Exception:  # pragma: no cover - defensive teardown
            logger.debug("Failed to stop AuroraViewQtHost timer", exc_info=True)
        self._timer = None
        self._tick_fn = None

    # ------------------------------------------------------------------
    # Lifecycle delegation
    # ------------------------------------------------------------------

    def start(self) -> None:
        """Start the dispatcher on the Qt event loop (or a thread when headless)."""
        self._base.start()

    def stop(self, timeout: float = 5.0) -> None:
        """Stop the dispatcher and detach the timer.

        Args:
            timeout: Seconds to wait for the dispatcher to drain.
        """
        try:
            self._base.stop(timeout=timeout)
        finally:
            self.detach_tick()

    def is_running(self) -> bool:
        """Return ``True`` while the dispatcher is running.

        ``is_running`` is a property on core's ``HostAdapter``, not a method.
        """
        return bool(self._base.is_running)

    def __enter__(self) -> "AuroraViewQtHost":
        self.start()
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.stop()
