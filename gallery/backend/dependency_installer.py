"""Dependency installer for AuroraView Gallery samples.

This module provides functionality for:
- Detecting missing dependencies from sample Requirements
- Installing dependencies with pip and streaming progress
- Showing progress via WebView UI before running samples

Example Requirements in sample docstring:
    '''Sample Title

    Description of the sample.

    Requirements:
        - PySide6>=6.0.0
        - requests>=2.0.0
    '''
"""

from __future__ import annotations

import subprocess
import sys
import re
import time
import select
from pathlib import Path
from typing import TYPE_CHECKING, Callable, Optional, Any
from threading import Event

if TYPE_CHECKING:
    from typing import List, Dict

# Try to import importlib.metadata for package version checking
try:
    from importlib.metadata import distributions, version as get_version
    HAS_IMPORTLIB_METADATA = True
except ImportError:
    try:
        from importlib_metadata import distributions, version as get_version
        HAS_IMPORTLIB_METADATA = True
    except ImportError:
        HAS_IMPORTLIB_METADATA = False


def parse_requirements_from_docstring(docstring: str) -> list[str]:
    """Parse Requirements section from a docstring.

    Args:
        docstring: The module docstring to parse.

    Returns:
        List of requirement strings (e.g., ["PySide6>=6.0.0", "requests"]).
    """
    if not docstring:
        return []

    # Find Requirements section
    # Supports:
    #   Requirements:
    #       - package1
    #       - package2>=1.0
    # or:
    #   Requirements: package1, package2

    requirements = []

    # Pattern 1: Multi-line with dashes
    req_match = re.search(
        r"Requirements:\s*\n((?:\s*-\s*[^\n]+\n?)+)",
        docstring,
        re.IGNORECASE | re.MULTILINE,
    )
    if req_match:
        lines = req_match.group(1).strip().split("\n")
        for line in lines:
            line = line.strip()
            if line.startswith("-"):
                req = line[1:].strip()
                if req:
                    requirements.append(req)
        return requirements

    # Pattern 2: Single line comma-separated
    req_match = re.search(
        r"Requirements:\s*([^\n]+)",
        docstring,
        re.IGNORECASE,
    )
    if req_match:
        reqs = req_match.group(1).strip()
        for req in reqs.split(","):
            req = req.strip()
            if req:
                requirements.append(req)

    return requirements


def check_package_installed(package_spec: str) -> bool:
    """Check if a package is installed and meets version requirements.

    Args:
        package_spec: Package specification (e.g., "PySide6>=6.0.0" or "requests").

    Returns:
        True if package is installed and meets requirements.
    """
    if not HAS_IMPORTLIB_METADATA:
        # Fallback: try to import the package
        package_name = re.split(r"[<>=!]", package_spec)[0].strip()
        try:
            __import__(package_name.replace("-", "_"))
            return True
        except ImportError:
            return False

    # Parse package name and version constraint
    match = re.match(r"([a-zA-Z0-9_-]+)\s*(.*)$", package_spec)
    if not match:
        return False

    package_name = match.group(1).strip()
    version_constraint = match.group(2).strip()

    try:
        installed_version = get_version(package_name)
        if not version_constraint:
            return True

        # Simple version comparison (supports >=, >, ==, <=, <)
        from packaging.version import Version
        from packaging.specifiers import SpecifierSet

        spec = SpecifierSet(version_constraint)
        return Version(installed_version) in spec
    except Exception:
        return False


def get_missing_requirements(requirements: list[str]) -> list[str]:
    """Get list of missing requirements.

    Args:
        requirements: List of requirement specifications.

    Returns:
        List of requirements that are not installed.
    """
    missing = []
    for req in requirements:
        if not check_package_installed(req):
            missing.append(req)
    return missing


def install_requirements(
    requirements: list[str],
    on_progress: Optional[Callable[[dict], None]] = None,
    python_exe: Optional[str] = None,
    cancel_event: Optional[Event] = None,
) -> dict[str, Any]:
    """Install requirements using pip with progress streaming.

    Args:
        requirements: List of package specifications to install.
        on_progress: Optional callback for progress updates.
            Called with dict: {"type": "start"|"output"|"complete"|"error",
                               "package": str, "line": str, "success": bool}
        python_exe: Python executable to use. Defaults to sys.executable.
        cancel_event: Optional threading.Event to cancel installation.

    Returns:
        Result dict: {"success": bool, "installed": list, "failed": list, "output": str, "cancelled": bool}
    """
    if not requirements:
        return {"success": True, "installed": [], "failed": [], "output": ""}

    python_exe = python_exe or sys.executable
    installed = []
    failed = []
    all_output = []
    cancelled = False

    for i, req in enumerate(requirements):
        if cancel_event and cancel_event.is_set():
            cancelled = True
            break

        package_name = re.split(r"[<>=!]", req)[0].strip()
        start_time = time.strftime("%H:%M:%S")

        if on_progress:
            on_progress({
                "type": "start",
                "package": package_name,
                "index": i,
                "total": len(requirements),
                "message": f"[{start_time}] Installing {package_name}...",
            })

        try:
            # Run pip install with real-time output
            process = subprocess.Popen(
                [python_exe, "-m", "pip", "install", req, "--progress-bar", "on"],
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=1,
            )

            output_lines = []
            # Use non-blocking read on Unix or poll on Windows
            import platform
            is_windows = platform.system() == "Windows"
            
            while True:
                # Check cancel event frequently
                if cancel_event and cancel_event.is_set():
                    # Use kill() for immediate termination
                    try:
                        process.kill()
                        process.wait(timeout=1)  # Wait briefly for cleanup
                    except Exception:
                        pass
                    cancelled = True
                    if on_progress:
                        on_progress({
                            "type": "output",
                            "package": package_name,
                            "line": f"[{time.strftime('%H:%M:%S')}] ⏹️ Cancelling...",
                        })
                    break

                # Non-blocking read with timeout
                if is_windows:
                    # On Windows, use poll() to check if process is done
                    if process.poll() is not None:
                        # Process finished, read remaining output
                        remaining = process.stdout.read()
                        if remaining:
                            for line in remaining.splitlines():
                                line = line.rstrip()
                                if line:
                                    output_lines.append(line)
                                    all_output.append(line)
                                    if on_progress:
                                        on_progress({
                                            "type": "output",
                                            "package": package_name,
                                            "line": f"[{time.strftime('%H:%M:%S')}] {line}",
                                        })
                        break
                    
                    # Try to read with short timeout simulation
                    line = process.stdout.readline()
                    if not line:
                        # Small sleep to allow cancel check
                        time.sleep(0.1)
                        continue
                else:
                    # On Unix, use select for non-blocking read
                    ready, _, _ = select.select([process.stdout], [], [], 0.1)
                    if not ready:
                        # No data ready, check if process finished
                        if process.poll() is not None:
                            break
                        continue
                    
                    line = process.stdout.readline()
                    if not line:
                        break

                line = line.rstrip()
                if not line:
                    continue

                output_lines.append(line)
                all_output.append(line)

                if on_progress:
                    # Filter out some noisy pip lines if needed, but here we want detail
                    on_progress({
                        "type": "output",
                        "package": package_name,
                        "line": f"[{time.strftime('%H:%M:%S')}] {line}",
                    })

            if cancelled:
                break

            process.wait()

            end_time = time.strftime("%H:%M:%S")
            if process.returncode == 0:
                installed.append(req)
                if on_progress:
                    on_progress({
                        "type": "complete",
                        "package": package_name,
                        "success": True,
                        "message": f"[{end_time}] Successfully installed {package_name}",
                    })
            else:
                failed.append(req)
                if on_progress:
                    on_progress({
                        "type": "error",
                        "package": package_name,
                        "success": False,
                        "message": f"[{end_time}] Failed to install {package_name} (exit {process.returncode})",
                        "output": "\n".join(output_lines),
                    })

        except Exception as e:
            failed.append(req)
            error_msg = str(e)
            all_output.append(f"Error: {error_msg}")
            if on_progress:
                on_progress({
                    "type": "error",
                    "package": package_name,
                    "success": False,
                    "message": f"[{time.strftime('%H:%M:%S')}] Error installing {package_name}: {error_msg}",
                })

    return {
        "success": not cancelled and len(failed) == 0,
        "installed": installed,
        "failed": failed,
        "output": "\n".join(all_output),
        "cancelled": cancelled,
    }


class DependencyInstaller:
    """High-level dependency installer with WebView UI support."""

    def __init__(self, webview=None):
        """Initialize installer.

        Args:
            webview: Optional WebView instance for progress UI.
        """
        self._webview = webview
        self._installing = False

    def check_sample_dependencies(self, sample_path: Path) -> dict[str, Any]:
        """Check if a sample has missing dependencies.

        Args:
            sample_path: Path to the sample Python file.

        Returns:
            Dict with "requirements", "missing", "needs_install".
        """
        try:
            source = sample_path.read_text(encoding="utf-8")
            import ast
            tree = ast.parse(source)
            docstring = ast.get_docstring(tree) or ""
        except Exception:
            return {"requirements": [], "missing": [], "needs_install": False}

        requirements = parse_requirements_from_docstring(docstring)
        missing = get_missing_requirements(requirements)

        return {
            "requirements": requirements,
            "missing": missing,
            "needs_install": len(missing) > 0,
        }

    def install_missing(
        self,
        missing: list[str],
        on_progress: Optional[Callable[[dict], None]] = None,
        cancel_event: Optional[Event] = None,
    ) -> dict[str, Any]:
        """Install missing dependencies.

        Args:
            missing: List of missing package specifications.
            on_progress: Progress callback.
            cancel_event: Optional cancel event.

        Returns:
            Installation result.
        """
        if self._installing:
            return {"success": False, "error": "Installation already in progress"}

        self._installing = True
        try:
            result = install_requirements(missing, on_progress, cancel_event=cancel_event)
            return result
        finally:
            self._installing = False

