"""Setup Python path for examples.

This module provides a unified way to add the auroraview package to sys.path
for all example scripts, regardless of their location in the examples directory.

Usage:
    import _setup_path  # This automatically adds the path
    from auroraview import WebView
"""

import sys
from pathlib import Path


def setup_auroraview_path():
    """Add auroraview package to sys.path.

    This function finds the project root directory and adds the python/
    subdirectory to sys.path, allowing examples to import auroraview
    regardless of their location.

    Returns:
        Path: The path to the auroraview package directory
    """
    # Get the directory containing this file (examples/)
    examples_dir = Path(__file__).parent.resolve()

    # Project root is the parent of examples/
    project_root = examples_dir.parent

    # Python package is in python/ subdirectory
    python_dir = project_root / "python"

    # Add to sys.path if not already present
    python_dir_str = str(python_dir)
    if python_dir_str not in sys.path:
        sys.path.insert(0, python_dir_str)

    return python_dir


# Automatically setup path when this module is imported
_python_dir = setup_auroraview_path()

# Verify the package can be imported
try:
    import auroraview

    _package_available = True
except ImportError:
    _package_available = False
    import warnings

    warnings.warn(
        f"Failed to import auroraview from {_python_dir}. "
        "Make sure the package is built with 'maturin develop' or 'maturin build'.",
        ImportWarning,
    )


def get_project_root():
    """Get the project root directory.

    Returns:
        Path: The project root directory
    """
    return Path(__file__).parent.parent.resolve()


def get_python_dir():
    """Get the Python package directory.

    Returns:
        Path: The python/ directory containing auroraview package
    """
    return get_project_root() / "python"


def is_package_available():
    """Check if auroraview package is available.

    Returns:
        bool: True if auroraview can be imported, False otherwise
    """
    return _package_available


if __name__ == "__main__":
    # Print diagnostic information
    print("=" * 60)
    print("AuroraView Path Setup Diagnostic")
    print("=" * 60)
    print(f"Project root: {get_project_root()}")
    print(f"Python dir: {get_python_dir()}")
    print(f"Package available: {is_package_available()}")
    print("sys.path entries:")
    for i, path in enumerate(sys.path[:5], 1):
        print(f"  {i}. {path}")
    print("=" * 60)
