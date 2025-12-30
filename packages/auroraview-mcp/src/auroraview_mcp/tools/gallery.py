"""Gallery tools for AuroraView MCP Server.

Provides tools for:
- Running and managing Gallery application
- Discovering and running example samples
- Process management for sample applications
"""

from __future__ import annotations

import asyncio
import os
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from auroraview_mcp.server import mcp


@dataclass
class ProcessInfo:
    """Information about a running sample process."""

    pid: int
    name: str
    process: subprocess.Popen[bytes]
    port: int | None = None
    is_gallery: bool = False


@dataclass
class ProcessManager:
    """Manages running sample processes."""

    _processes: dict[int, ProcessInfo] = field(default_factory=dict)

    def add(self, info: ProcessInfo) -> None:
        """Add a process to tracking."""
        self._processes[info.pid] = info

    def remove(self, pid: int) -> ProcessInfo | None:
        """Remove and return a process."""
        return self._processes.pop(pid, None)

    def get(self, pid: int) -> ProcessInfo | None:
        """Get process info by PID."""
        return self._processes.get(pid)

    def get_by_name(self, name: str) -> ProcessInfo | None:
        """Get process info by sample name."""
        for info in self._processes.values():
            if info.name == name:
                return info
        return None

    def get_gallery(self) -> ProcessInfo | None:
        """Get running Gallery process."""
        for info in self._processes.values():
            if info.is_gallery:
                return info
        return None

    def list_all(self) -> list[ProcessInfo]:
        """List all tracked processes."""
        return list(self._processes.values())

    def cleanup(self) -> None:
        """Clean up terminated processes."""
        terminated = []
        for pid, info in self._processes.items():
            if info.process.poll() is not None:
                terminated.append(pid)
        for pid in terminated:
            del self._processes[pid]


# Global process manager
_process_manager = ProcessManager()


def get_project_root() -> Path:
    """Get the project root directory."""
    # Try environment variable first
    env_root = os.environ.get("AURORAVIEW_PROJECT_ROOT")
    if env_root:
        return Path(env_root)

    # Try to find relative to this package
    # Structure: packages/auroraview-mcp/src/auroraview_mcp/tools/gallery.py
    current = Path(__file__).resolve()
    for _ in range(6):  # Go up to project root
        current = current.parent
        # Check for project markers
        if (current / "Cargo.toml").exists() and (current / "gallery").exists():
            return current

    raise FileNotFoundError("Could not find project root directory")


def get_gallery_dir() -> Path:
    """Get the Gallery directory path."""
    env_dir = os.environ.get("AURORAVIEW_GALLERY_DIR")
    if env_dir:
        return Path(env_dir)

    return get_project_root() / "gallery"


def get_examples_dir() -> Path:
    """Get the examples directory path."""
    # Try environment variable first
    env_dir = os.environ.get("AURORAVIEW_EXAMPLES_DIR")
    if env_dir:
        return Path(env_dir)

    return get_project_root() / "examples"


def get_sample_info(sample_path: Path) -> dict[str, Any] | None:
    """Get sample information from a Python file or directory.

    Args:
        sample_path: Path to sample file (.py) or directory.

    Returns:
        Sample info dict or None if invalid.
    """
    # Handle both files and directories
    if sample_path.is_dir():
        # Look for main.py or first .py file
        main_file = sample_path / "main.py"
        if not main_file.exists():
            py_files = list(sample_path.glob("*.py"))
            if not py_files:
                return None
            main_file = py_files[0]
        name = sample_path.name
    elif sample_path.suffix == ".py":
        main_file = sample_path
        name = sample_path.stem
        # Remove common suffixes
        for suffix in ["_demo", "_example", "_test"]:
            if name.endswith(suffix):
                name = name[: -len(suffix)]
                break
    else:
        return None

    if not main_file.exists():
        return None

    # Read content
    try:
        content = main_file.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return None

    # Extract metadata from docstring
    title = name.replace("_", " ").title()
    description = ""
    category = "uncategorized"
    tags: list[str] = []

    # Try to extract from docstring
    if '"""' in content:
        start = content.find('"""') + 3
        end = content.find('"""', start)
        if end > start:
            docstring = content[start:end].strip()
            lines = docstring.split("\n")
            if lines:
                # First line is title
                first_line = lines[0].strip()
                if " - " in first_line:
                    title = first_line.split(" - ")[0].strip()
                else:
                    title = first_line.rstrip(".")

                # Rest is description
                if len(lines) > 1:
                    desc_lines = []
                    for line in lines[1:]:
                        stripped = line.strip()
                        if stripped and not stripped.startswith(("-", "Features", "Use")):
                            desc_lines.append(stripped)
                        if len(desc_lines) >= 2:
                            break
                    description = " ".join(desc_lines)

    return {
        "name": name,
        "title": title,
        "description": description[:200] if description else f"Demo: {title}",
        "category": category,
        "tags": tags,
        "path": str(sample_path),
        "main_file": str(main_file),
        "source_file": main_file.name,
    }


def scan_samples(examples_dir: Path) -> list[dict[str, Any]]:
    """Scan examples directory for samples.

    Args:
        examples_dir: Path to examples directory.

    Returns:
        List of sample info dicts.
    """
    samples = []

    if not examples_dir.exists():
        return samples

    # Scan .py files directly in examples dir
    for py_file in sorted(examples_dir.glob("*.py")):
        if py_file.name.startswith("__"):
            continue

        info = get_sample_info(py_file)
        if info:
            samples.append(info)

    # Also scan subdirectories
    for subdir in sorted(examples_dir.iterdir()):
        if subdir.is_dir() and not subdir.name.startswith(("_", ".")):
            info = get_sample_info(subdir)
            if info:
                samples.append(info)

    return samples


# ==================== Gallery Tools ====================


@mcp.tool()
async def run_gallery(
    port: int | None = None,
    dev_mode: bool = False,
) -> dict[str, Any]:
    """Start the AuroraView Gallery application.

    Launches the Gallery showcase application for browsing and running samples.

    Args:
        port: Optional CDP port for debugging (default: 9222).
        dev_mode: If True, run with Vite dev server for hot reload.

    Returns:
        Process information:
        - pid: Process ID
        - status: "running"
        - port: CDP port
        - gallery_dir: Gallery directory path
    """
    # Check if gallery is already running
    existing = _process_manager.get_gallery()
    if existing and existing.process.poll() is None:
        return {
            "pid": existing.pid,
            "status": "already_running",
            "port": existing.port,
            "message": "Gallery is already running",
        }

    try:
        gallery_dir = get_gallery_dir()
    except FileNotFoundError as e:
        raise RuntimeError(str(e)) from e

    main_file = gallery_dir / "main.py"
    if not main_file.exists():
        raise RuntimeError(f"Gallery main.py not found: {main_file}")

    # Build command
    env = os.environ.copy()
    cdp_port = port or 9222
    env["AURORAVIEW_CDP_PORT"] = str(cdp_port)

    if dev_mode:
        env["AURORAVIEW_DEV_MODE"] = "1"

    # Start process
    process = subprocess.Popen(
        [sys.executable, str(main_file)],
        cwd=str(gallery_dir),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    proc_info = ProcessInfo(
        pid=process.pid,
        name="gallery",
        process=process,
        port=cdp_port,
        is_gallery=True,
    )
    _process_manager.add(proc_info)

    # Wait for process to start
    await asyncio.sleep(1.0)

    # Check if process is still running
    if process.poll() is not None:
        stdout, stderr = process.communicate()
        _process_manager.remove(process.pid)
        error_msg = stderr.decode("utf-8", errors="replace")
        raise RuntimeError(f"Gallery failed to start: {error_msg}")

    return {
        "pid": process.pid,
        "status": "running",
        "port": cdp_port,
        "gallery_dir": str(gallery_dir),
    }


@mcp.tool()
async def stop_gallery() -> dict[str, Any]:
    """Stop the running Gallery application.

    Returns:
        Stop result:
        - status: "stopped" or "not_running"
        - pid: Stopped process ID (if was running)
    """
    proc_info = _process_manager.get_gallery()

    if not proc_info:
        return {"status": "not_running"}

    # Terminate process
    proc_info.process.terminate()
    try:
        proc_info.process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc_info.process.kill()

    _process_manager.remove(proc_info.pid)

    return {
        "status": "stopped",
        "pid": proc_info.pid,
    }


@mcp.tool()
async def get_gallery_status() -> dict[str, Any]:
    """Get the current Gallery application status.

    Returns:
        Gallery status:
        - running: Whether Gallery is running
        - pid: Process ID (if running)
        - port: CDP port (if running)
        - gallery_dir: Gallery directory path
        - dist_exists: Whether built dist exists
    """
    try:
        gallery_dir = get_gallery_dir()
        dist_exists = (gallery_dir / "dist" / "index.html").exists()
    except FileNotFoundError:
        return {
            "running": False,
            "error": "Gallery directory not found",
        }

    proc_info = _process_manager.get_gallery()

    if proc_info and proc_info.process.poll() is None:
        return {
            "running": True,
            "pid": proc_info.pid,
            "port": proc_info.port,
            "gallery_dir": str(gallery_dir),
            "dist_exists": dist_exists,
        }

    # Clean up if terminated
    if proc_info:
        _process_manager.remove(proc_info.pid)

    return {
        "running": False,
        "gallery_dir": str(gallery_dir),
        "dist_exists": dist_exists,
    }


# ==================== Sample Tools ====================


@mcp.tool()
async def get_samples(
    category: str | None = None,
    tags: list[str] | None = None,
) -> list[dict[str, Any]]:
    """Get list of available AuroraView samples.

    Returns information about all available sample applications.

    Args:
        category: Filter by category (e.g., "getting_started", "advanced").
        tags: Filter by tags.

    Returns:
        List of samples, each containing:
        - name: Sample identifier
        - title: Human-readable title
        - description: Sample description
        - category: Sample category
        - tags: List of tags
        - path: Full path to sample
        - source_file: Main Python file name
    """
    try:
        examples_dir = get_examples_dir()
    except FileNotFoundError:
        return []

    samples = scan_samples(examples_dir)

    # Apply filters
    if category:
        samples = [s for s in samples if s.get("category") == category]

    if tags:
        samples = [s for s in samples if any(t in s.get("tags", []) for t in tags)]

    return samples


@mcp.tool()
async def run_sample(
    name: str,
    port: int | None = None,
) -> dict[str, Any]:
    """Run an AuroraView sample application.

    Starts a sample application in a new process.

    Args:
        name: Sample name (file name without extension or directory name).
        port: Optional CDP port for the sample.

    Returns:
        Process information:
        - pid: Process ID
        - name: Sample name
        - status: "running"
        - port: CDP port (if specified)
    """
    try:
        examples_dir = get_examples_dir()
    except FileNotFoundError as e:
        raise RuntimeError(str(e)) from e

    # Find sample - try both file and directory
    sample_path = None
    main_file = None

    # Try as .py file
    for suffix in ["", "_demo", "_example"]:
        candidate = examples_dir / f"{name}{suffix}.py"
        if candidate.exists():
            sample_path = candidate
            main_file = candidate
            break

    # Try as directory
    if not sample_path:
        candidate = examples_dir / name
        if candidate.is_dir():
            sample_path = candidate
            main_file = candidate / "main.py"
            if not main_file.exists():
                py_files = list(candidate.glob("*.py"))
                if py_files:
                    main_file = py_files[0]

    if not sample_path or not main_file or not main_file.exists():
        raise RuntimeError(f"Sample not found: {name}")

    # Build environment
    env = os.environ.copy()
    if port:
        env["AURORAVIEW_CDP_PORT"] = str(port)

    # Start process
    cwd = str(sample_path.parent) if sample_path.is_file() else str(sample_path)
    process = subprocess.Popen(
        [sys.executable, str(main_file)],
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    proc_info = ProcessInfo(
        pid=process.pid,
        name=name,
        process=process,
        port=port,
    )
    _process_manager.add(proc_info)

    # Wait a bit for process to start
    await asyncio.sleep(0.5)

    # Check if process is still running
    if process.poll() is not None:
        stdout, stderr = process.communicate()
        _process_manager.remove(process.pid)
        error_msg = stderr.decode("utf-8", errors="replace")
        raise RuntimeError(f"Sample failed to start: {error_msg}")

    return {
        "pid": process.pid,
        "name": name,
        "status": "running",
        "port": port,
        "main_file": str(main_file),
    }


@mcp.tool()
async def stop_sample(
    pid: int | None = None,
    name: str | None = None,
) -> dict[str, Any]:
    """Stop a running sample application.

    Terminates a running sample process.

    Args:
        pid: Process ID to stop.
        name: Sample name to stop (if multiple, stops the first match).

    Returns:
        Stop result:
        - status: "stopped"
        - pid: Stopped process ID
        - name: Sample name
    """
    if not pid and not name:
        raise ValueError("Either pid or name must be provided")

    proc_info: ProcessInfo | None = None

    if pid:
        proc_info = _process_manager.get(pid)
    elif name:
        proc_info = _process_manager.get_by_name(name)

    if not proc_info:
        raise RuntimeError("Process not found")

    # Terminate process
    proc_info.process.terminate()
    try:
        proc_info.process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc_info.process.kill()

    _process_manager.remove(proc_info.pid)

    return {
        "status": "stopped",
        "pid": proc_info.pid,
        "name": proc_info.name,
    }


@mcp.tool()
async def get_sample_source(name: str) -> str:
    """Get the source code of a sample.

    Returns the main Python source file of a sample.

    Args:
        name: Sample name.

    Returns:
        Python source code as string.
    """
    try:
        examples_dir = get_examples_dir()
    except FileNotFoundError as e:
        raise RuntimeError(str(e)) from e

    # Find sample
    main_file = None

    # Try as .py file
    for suffix in ["", "_demo", "_example"]:
        candidate = examples_dir / f"{name}{suffix}.py"
        if candidate.exists():
            main_file = candidate
            break

    # Try as directory
    if not main_file:
        sample_dir = examples_dir / name
        if sample_dir.is_dir():
            main_file = sample_dir / "main.py"
            if not main_file.exists():
                py_files = list(sample_dir.glob("*.py"))
                if py_files:
                    main_file = py_files[0]

    if not main_file or not main_file.exists():
        raise RuntimeError(f"Sample not found: {name}")

    return main_file.read_text(encoding="utf-8")


@mcp.tool()
async def list_processes() -> list[dict[str, Any]]:
    """List all running sample processes.

    Returns information about all tracked sample processes.

    Returns:
        List of processes, each containing:
        - pid: Process ID
        - name: Sample name
        - status: "running" or "terminated"
        - port: CDP port (if specified)
        - is_gallery: Whether this is the Gallery process
    """
    _process_manager.cleanup()

    processes = []
    for info in _process_manager.list_all():
        status = "running" if info.process.poll() is None else "terminated"
        processes.append(
            {
                "pid": info.pid,
                "name": info.name,
                "status": status,
                "port": info.port,
                "is_gallery": info.is_gallery,
            }
        )

    return processes


@mcp.tool()
async def stop_all_samples() -> dict[str, Any]:
    """Stop all running sample processes.

    Terminates all tracked sample processes including Gallery.

    Returns:
        Result:
        - stopped: Number of processes stopped
        - pids: List of stopped PIDs
    """
    _process_manager.cleanup()

    stopped_pids = []
    for info in _process_manager.list_all():
        if info.process.poll() is None:
            info.process.terminate()
            try:
                info.process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                info.process.kill()
            stopped_pids.append(info.pid)

    # Clear all
    for pid in stopped_pids:
        _process_manager.remove(pid)

    return {
        "stopped": len(stopped_pids),
        "pids": stopped_pids,
    }


# ==================== Helper Tools ====================


@mcp.tool()
async def get_project_info() -> dict[str, Any]:
    """Get AuroraView project information.

    Returns paths and configuration for the AuroraView project.

    Returns:
        Project info:
        - project_root: Project root directory
        - gallery_dir: Gallery directory
        - examples_dir: Examples directory
        - gallery_built: Whether Gallery dist exists
        - sample_count: Number of available samples
    """
    try:
        project_root = get_project_root()
        gallery_dir = get_gallery_dir()
        examples_dir = get_examples_dir()
    except FileNotFoundError as e:
        return {"error": str(e)}

    gallery_built = (gallery_dir / "dist" / "index.html").exists()
    samples = scan_samples(examples_dir)

    return {
        "project_root": str(project_root),
        "gallery_dir": str(gallery_dir),
        "examples_dir": str(examples_dir),
        "gallery_built": gallery_built,
        "sample_count": len(samples),
    }
