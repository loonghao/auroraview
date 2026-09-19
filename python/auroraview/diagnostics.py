# -*- coding: utf-8 -*-
"""Runtime diagnostics for AuroraView.

:func:`diagnostics` reports everything needed to triage a DCC integration
issue in a single call:

* which host application was detected
* which thread dispatcher backend is in effect, and what else was available
* whether the caller is on the host main thread
* which Qt binding (if any) is loaded
* whether the native ``_core`` extension is importable

Two properties are intentional:

1. **Standard library only.** Importing this module must never pull in a host
   SDK or Qt binding; every optional dependency is probed lazily.
2. **It never raises.** A diagnostics call that fails is worse than one that
   reports a partial picture, so every probe is isolated and failures are
   collected in the ``errors`` list instead of propagating.

Example:
    >>> import auroraview
    >>> report = auroraview.diagnostics()
    >>> report["host"]["name"]
    'Maya'
    >>> print(auroraview.format_diagnostics())
"""

from __future__ import annotations

import importlib
import os
import platform
import sys
import threading
from typing import Any, Dict, List, Optional

from .utils.thread_dispatcher import (
    get_current_dcc_name,
    is_dcc_environment,
    list_dispatcher_backends,
)
from .utils.thread_dispatcher.registry import (
    ENV_DISPATCHER_BACKEND,
    get_dispatcher_backend,
)

__all__ = ["diagnostics", "format_diagnostics"]

# Qt bindings probed in order when qtpy is not installed.
_QT_BINDINGS = ("PySide6", "PySide2", "PyQt6", "PyQt5")


def _record_error(errors: List[Dict[str, str]], probe: str, exc: BaseException) -> None:
    """Append a probe failure to the error list."""
    errors.append({"probe": probe, "error": "{}: {}".format(type(exc).__name__, exc)})


def _auroraview_info(errors: List[Dict[str, str]]) -> Dict[str, Any]:
    """Report the auroraview package itself."""
    info: Dict[str, Any] = {
        "version": None,
        "path": None,
        "native_core": False,
        "native_core_path": None,
    }

    try:
        from . import __version__ as version

        info["version"] = version
    except Exception as exc:  # noqa: BLE001 - diagnostics must not raise
        _record_error(errors, "auroraview.version", exc)

    try:
        import auroraview

        info["path"] = os.path.dirname(os.path.abspath(auroraview.__file__))
    except Exception as exc:  # noqa: BLE001
        _record_error(errors, "auroraview.path", exc)

    try:
        native_core = importlib.import_module("auroraview._core")
        info["native_core"] = True
        info["native_core_path"] = getattr(native_core, "__file__", None)
    except Exception as exc:  # noqa: BLE001
        info["native_core"] = False
        _record_error(errors, "auroraview._core", exc)

    return info


def _platform_info() -> Dict[str, Any]:
    """Report the operating system."""
    return {
        "system": platform.system(),
        "release": platform.release(),
        "version": platform.version(),
        "machine": platform.machine(),
    }


def _python_info() -> Dict[str, Any]:
    """Report the interpreter auroraview is running in."""
    return {
        "version": platform.python_version(),
        "implementation": platform.python_implementation(),
        "executable": sys.executable,
    }


def _host_info(errors: List[Dict[str, str]]) -> Dict[str, Any]:
    """Report the detected DCC host, if any."""
    info: Dict[str, Any] = {"name": None, "is_dcc": False}

    try:
        info["name"] = get_current_dcc_name()
        info["is_dcc"] = is_dcc_environment()
    except Exception as exc:  # noqa: BLE001
        _record_error(errors, "host", exc)

    return info


def _dispatcher_info(errors: List[Dict[str, str]]) -> Dict[str, Any]:
    """Report the thread dispatcher backend in effect and its alternatives."""
    info: Dict[str, Any] = {
        "backend": None,
        "priority": None,
        "env_override": os.environ.get(ENV_DISPATCHER_BACKEND) or None,
        "backends": [],
    }

    try:
        info["backends"] = [
            {"name": name, "priority": priority, "available": available}
            for priority, name, available in list_dispatcher_backends()
        ]
    except Exception as exc:  # noqa: BLE001
        _record_error(errors, "dispatcher.backends", exc)

    try:
        backend = get_dispatcher_backend()
        info["backend"] = backend.get_name()
    except Exception as exc:  # noqa: BLE001
        _record_error(errors, "dispatcher.backend", exc)
        return info

    # Resolve the priority of the selected backend from the registered list.
    for entry in info["backends"]:
        if entry["name"] == info["backend"]:
            info["priority"] = entry["priority"]
            break

    return info


def _thread_info(errors: List[Dict[str, str]]) -> Dict[str, Any]:
    """Report the calling thread from both the Python and host points of view."""
    current = threading.current_thread()

    info: Dict[str, Any] = {
        "name": current.name,
        "ident": current.ident,
        "is_python_main_thread": current is threading.main_thread(),
        "is_host_main_thread": None,
    }

    try:
        info["is_host_main_thread"] = get_dispatcher_backend().is_main_thread()
    except Exception as exc:  # noqa: BLE001
        _record_error(errors, "thread.is_host_main_thread", exc)

    return info


def _qt_info(errors: List[Dict[str, str]]) -> Dict[str, Any]:
    """Report the Qt binding in use, without importing anything eagerly."""
    info: Dict[str, Any] = {
        "available": False,
        "binding": None,
        "api": os.environ.get("QT_API") or None,
        "app_instance": None,
    }

    try:
        import qtpy

        info["binding"] = getattr(qtpy, "API_NAME", None)
    except Exception:  # noqa: BLE001 - qtpy absent, probe raw bindings instead
        for binding_name in _QT_BINDINGS:
            try:
                importlib.import_module(binding_name)
            except ImportError:
                continue
            info["binding"] = binding_name
            break

    info["available"] = info["binding"] is not None
    if not info["available"]:
        return info

    try:
        from qtpy.QtCore import QCoreApplication

        info["app_instance"] = QCoreApplication.instance() is not None
    except Exception as exc:  # noqa: BLE001
        _record_error(errors, "qt.app_instance", exc)

    return info


def diagnostics() -> Dict[str, Any]:
    """Collect a host and thread-safety diagnostics report.

    Returns:
        A JSON-serialisable dictionary with the keys ``auroraview``,
        ``platform``, ``python``, ``host``, ``dispatcher``, ``thread``,
        ``qt`` and ``errors``. Probe failures are recorded in ``errors``
        rather than raised, so the report is always as complete as possible.
    """
    errors: List[Dict[str, str]] = []

    report: Dict[str, Any] = {
        "auroraview": _auroraview_info(errors),
        "platform": _platform_info(),
        "python": _python_info(),
        "host": _host_info(errors),
        "dispatcher": _dispatcher_info(errors),
        "thread": _thread_info(errors),
        "qt": _qt_info(errors),
        "errors": errors,
    }

    return report


def format_diagnostics(report: Optional[Dict[str, Any]] = None) -> str:
    """Render a diagnostics report as human-readable text.

    Args:
        report: A report produced by :func:`diagnostics`. A fresh report is
                collected when omitted.

    Returns:
        Multi-line text suitable for logs and bug reports.
    """
    if report is None:
        report = diagnostics()

    lines = [
        "AuroraView diagnostics",
        "======================",
        "auroraview : {} (native core: {})".format(
            report["auroraview"]["version"],
            "yes" if report["auroraview"]["native_core"] else "no",
        ),
        "platform   : {} {} ({})".format(
            report["platform"]["system"],
            report["platform"]["release"],
            report["platform"]["machine"],
        ),
        "python     : {} ({})".format(
            report["python"]["version"], report["python"]["implementation"]
        ),
        "host       : {} (dcc: {})".format(
            report["host"]["name"] or "none",
            "yes" if report["host"]["is_dcc"] else "no",
        ),
        "dispatcher : {} (priority: {})".format(
            report["dispatcher"]["backend"] or "none",
            report["dispatcher"]["priority"],
        ),
    ]

    override = report["dispatcher"]["env_override"]
    if override:
        lines.append("             env override: {}={}".format(ENV_DISPATCHER_BACKEND, override))

    lines.append(
        "thread     : {} (python main: {}, host main: {})".format(
            report["thread"]["name"],
            "yes" if report["thread"]["is_python_main_thread"] else "no",
            "yes" if report["thread"]["is_host_main_thread"] else "no",
        )
    )
    lines.append(
        "qt         : {} (app instance: {})".format(
            report["qt"]["binding"] or "none",
            "yes" if report["qt"]["app_instance"] else "no",
        )
    )

    lines.append("backends   :")
    for entry in report["dispatcher"]["backends"]:
        lines.append(
            "  - {:<12} priority={:<4} available={}".format(
                entry["name"], entry["priority"], "yes" if entry["available"] else "no"
            )
        )

    if report["errors"]:
        lines.append("errors     :")
        for entry in report["errors"]:
            lines.append("  - {}: {}".format(entry["probe"], entry["error"]))

    return "\n".join(lines)
