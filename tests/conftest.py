"""
Pytest configuration for AuroraView tests.

This module provides:
- Event loop configuration for Windows
- Common test utilities
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

# Fix for Playwright on Windows with pytest-asyncio
# Playwright's sync API needs ProactorEventLoop for subprocess support
if sys.platform == "win32":
    import asyncio

    # Set the default event loop policy to ProactorEventLoop
    # This is required for Playwright's subprocess spawning
    asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())

# Add project paths
# In CI, the wheel is installed to site-packages, so we should use that.
# Only add source paths for local development when the package is not installed.
PROJECT_ROOT = Path(__file__).parent.parent


def _add_sys_path(path: Path) -> None:
    path_str = str(path)
    if path_str not in sys.path:
        sys.path.insert(0, path_str)


def _is_package_installed() -> bool:
    """Check if auroraview is installed as a package (with _core module)."""
    try:
        # Try to import from site-packages first
        import importlib.util

        spec = importlib.util.find_spec("auroraview._core")
        return spec is not None
    except (ImportError, ModuleNotFoundError):
        return False


# Only add source paths if the package is not installed
# This allows CI to use the installed wheel while local dev uses source
if not _is_package_installed():
    _add_sys_path(PROJECT_ROOT / "python")
    _add_sys_path(PROJECT_ROOT)

# Always add gallery path for gallery-related tests
_add_sys_path(PROJECT_ROOT / "gallery")


@pytest.fixture(scope="session", autouse=True)
def setup_event_loop_policy():
    """Ensure correct event loop policy for Playwright."""
    if sys.platform == "win32":
        import asyncio

        asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())
    yield


@pytest.fixture
def isolated_dispatcher(monkeypatch):
    """Run a test against a dispatcher registry pinned to the fallback backend.

    ``auroraview.utils.thread_dispatcher.registry`` stores its backend list, its
    resolved backend and its lazy-init flag in module globals, so the backend a
    test receives depends on whatever ran earlier in the same pytest process:

    * ``tests/python/unit/integration/qt/test_core_zombie_guards.py`` builds a
      ``QApplication`` for a module-scoped fixture. Qt keeps that instance alive
      as a process-wide singleton, so ``QCoreApplication.instance()`` stays
      non-``None`` for every later test even though pytest never runs a Qt event
      loop. ``QtDispatcherBackend`` (priority 100) then outranks
      ``FallbackDispatcherBackend`` (priority 0), and the
      ``QTimer.singleShot(0, ...)`` work it queues is never dispatched --
      deferred calls silently never run and ``run_sync`` from a worker thread
      blocks forever.
    * Tests calling ``register_dispatcher_backend()`` without unregistering leak
      entries into every test that follows.

    Dispatcher suites exercise registration, priority, environment overrides and
    timeouts -- not Qt or DCC integration -- so pinning the registry to the
    fallback backend (which runs deferred work inline on the calling thread)
    makes them behave identically whether or not a stray ``QApplication``
    exists. A test that deliberately registers its own backend still wins,
    because registration itself is untouched. Every global is restored
    afterwards, so these suites no longer leak state into their neighbours.
    """
    from auroraview.utils.thread_dispatcher import registry
    from auroraview.utils.thread_dispatcher.backends import FallbackDispatcherBackend

    monkeypatch.setattr(
        registry,
        "_DISPATCHER_BACKENDS",
        [(registry.DispatcherPriority.FALLBACK, FallbackDispatcherBackend, "Fallback")],
    )
    monkeypatch.setattr(registry, "_cached_backend", None)
    monkeypatch.setattr(registry, "_builtins_registered", True)
    monkeypatch.delenv(registry.ENV_DISPATCHER_BACKEND, raising=False)


# Markers are declared in `[tool.pytest.ini_options] markers` in pyproject.toml,
# the single source of truth for pytest configuration. Registering them here
# again would only work for runs that happen to collect this conftest, which is
# how `cdp` / `e2e` / `slow` / `integration` silently escaped `--strict-markers`
# for runs rooted outside `tests/`.
