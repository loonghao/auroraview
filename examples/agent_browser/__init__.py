# -*- coding: utf-8 -*-
"""Agent Browser - Multi-tab browser built with AuroraView TabManager.

This package provides a multi-tab browser using the Rust-based TabManager
which follows Microsoft WebView2Browser architecture.

Usage:
    # As a module
    python -m examples.agent_browser

    # As a script
    python examples/agent_browser.py

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

__all__ = ["main"]


def __getattr__(name):
    """Lazy import from the standalone script."""
    import sys
    from pathlib import Path

    # Add parent directory to path for importing agent_browser.py
    examples_dir = Path(__file__).parent.parent
    if str(examples_dir) not in sys.path:
        sys.path.insert(0, str(examples_dir))

    # Add python directory to path
    project_root = examples_dir.parent
    python_dir = project_root / "python"
    if str(python_dir) not in sys.path:
        sys.path.insert(0, str(python_dir))

    # Import the module
    import importlib.util

    spec = importlib.util.spec_from_file_location(
        "agent_browser_main", examples_dir / "agent_browser.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    if name in __all__:
        return getattr(module, name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
