"""Gallery tools for AuroraView MCP Server."""

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


def get_examples_dir() -> Path:
    """Get the examples directory path."""
    # Try environment variable first
    env_dir = os.environ.get("AURORAVIEW_EXAMPLES_DIR")
    if env_dir:
        return Path(env_dir)

    # Try to find relative to this package
    # Assuming structure: packages/auroraview-mcp/src/auroraview_mcp/tools/gallery.py
    # Examples at: examples/
    current = Path(__file__).resolve()
    for _ in range(6):  # Go up to project root
        current = current.parent
        examples = current / "examples"
        if examples.exists():
            return examples

    raise FileNotFoundError("Could not find examples directory")


def get_sample_info(sample_dir: Path) -> dict[str, Any] | None:
    """Get sample information from a directory."""
    # Look for main Python file
    main_file = sample_dir / "main.py"
    if not main_file.exists():
        # Try sample_name.py
        py_files = list(sample_dir.glob("*.py"))
        if not py_files:
            return None
        main_file = py_files[0]

    # Extract metadata from docstring or comments
    content = main_file.read_text(encoding="utf-8")

    # Simple metadata extraction
    name = sample_dir.name
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
                title = lines[0].strip()
                if len(lines) > 1:
                    description = "\n".join(lines[1:]).strip()

    return {
        "name": name,
        "title": title,
        "description": description,
        "category": category,
        "tags": tags,
        "path": str(sample_dir),
        "main_file": str(main_file),
    }


@mcp.tool()
async def get_samples(
    category: str | None = None, tags: list[str] | None = None
) -> list[dict[str, Any]]:
    """Get list of available AuroraView samples.

    Returns information about all available sample applications.

    Args:
        category: Filter by category (e.g., "getting-started", "advanced").
        tags: Filter by tags.

    Returns:
        List of samples, each containing:
        - name: Sample directory name
        - title: Human-readable title
        - description: Sample description
        - category: Sample category
        - tags: List of tags
        - path: Full path to sample directory
    """
    try:
        examples_dir = get_examples_dir()
    except FileNotFoundError:
        return []

    samples = []
    for item in examples_dir.iterdir():
        if item.is_dir() and not item.name.startswith(("_", ".")):
            info = get_sample_info(item)
            if info:
                # Apply filters
                if category and info.get("category") != category:
                    continue
                if tags:
                    sample_tags = info.get("tags", [])
                    if not any(t in sample_tags for t in tags):
                        continue
                samples.append(info)

    return sorted(samples, key=lambda x: x["name"])


@mcp.tool()
async def run_sample(name: str, port: int | None = None) -> dict[str, Any]:
    """Run an AuroraView sample application.

    Starts a sample application in a new process.

    Args:
        name: Sample name (directory name in examples/).
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

    sample_dir = examples_dir / name
    if not sample_dir.exists():
        raise RuntimeError(f"Sample not found: {name}")

    info = get_sample_info(sample_dir)
    if not info:
        raise RuntimeError(f"Invalid sample: {name}")

    main_file = info["main_file"]

    # Build command
    env = os.environ.copy()
    if port:
        env["AURORAVIEW_CDP_PORT"] = str(port)

    # Start process
    process = subprocess.Popen(
        [sys.executable, main_file],
        cwd=str(sample_dir),
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
        # Process terminated
        stdout, stderr = process.communicate()
        _process_manager.remove(process.pid)
        raise RuntimeError(
            f"Sample failed to start: {stderr.decode('utf-8', errors='replace')}"
        )

    return {
        "pid": process.pid,
        "name": name,
        "status": "running",
        "port": port,
    }


@mcp.tool()
async def stop_sample(pid: int | None = None, name: str | None = None) -> dict[str, Any]:
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

    sample_dir = examples_dir / name
    if not sample_dir.exists():
        raise RuntimeError(f"Sample not found: {name}")

    info = get_sample_info(sample_dir)
    if not info:
        raise RuntimeError(f"Invalid sample: {name}")

    main_file = Path(info["main_file"])
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
    """
    _process_manager.cleanup()

    processes = []
    for info in _process_manager.list_all():
        status = "running" if info.process.poll() is None else "terminated"
        processes.append({
            "pid": info.pid,
            "name": info.name,
            "status": status,
            "port": info.port,
        })

    return processes
