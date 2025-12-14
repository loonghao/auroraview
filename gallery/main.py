#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""AuroraView Gallery - Interactive showcase of all features and components.

This gallery uses a React frontend built with Vite and displays via AuroraView.
It demonstrates the full capabilities of AuroraView including:
- Rust-powered plugin system with IPC
- Process management with stdout/stderr streaming
- Native desktop integration

Usage:
    python gallery/main.py
    # or via just:
    just gallery

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import ast
import os
import sys
from pathlib import Path
from typing import Optional

# Add parent directory to path for imports
PROJECT_ROOT = Path(__file__).parent.parent
GALLERY_DIR = Path(__file__).parent
DIST_DIR = GALLERY_DIR / "dist"

# Check if running in packed mode (set by Rust CLI)
PACKED_MODE = os.environ.get("AURORAVIEW_PACKED", "0") == "1"

# Determine examples directory based on mode
# In packed mode, examples are extracted to AURORAVIEW_EXAMPLES_DIR
# In development mode, examples are in the project root
if PACKED_MODE:
    _examples_env = os.environ.get("AURORAVIEW_EXAMPLES_DIR")
    if _examples_env:
        EXAMPLES_DIR = Path(_examples_env)
    else:
        # Fallback: check resources directory relative to script
        _resources_dir = os.environ.get("AURORAVIEW_RESOURCES_DIR")
        if _resources_dir:
            EXAMPLES_DIR = Path(_resources_dir) / "examples"
        else:
            # Last resort: use project root (won't work in packed mode)
            EXAMPLES_DIR = PROJECT_ROOT / "examples"
else:
    EXAMPLES_DIR = PROJECT_ROOT / "examples"

sys.path.insert(0, str(PROJECT_ROOT))
sys.path.insert(0, str(PROJECT_ROOT / "python"))

from auroraview import PluginManager, WebView, json_dumps, json_loads


# Category definitions
CATEGORIES = {
    "getting_started": {
        "title": "Getting Started",
        "icon": "rocket",
        "description": "Quick start examples and basic usage patterns",
    },
    "api_patterns": {
        "title": "API Patterns",
        "icon": "code",
        "description": "Different ways to use the AuroraView API",
    },
    "window_features": {
        "title": "Window Features",
        "icon": "layout",
        "description": "Window styles, events, and customization",
    },
    "desktop_features": {
        "title": "Desktop Features",
        "icon": "monitor",
        "description": "File dialogs, shell commands, and system integration",
    },
    "dcc_integration": {
        "title": "DCC Integration",
        "icon": "box",
        "description": "Maya, Houdini, Blender, and other DCC apps",
    },
}

# Icon mapping based on keywords in filename or docstring
ICON_MAPPING = {
    "decorator": "wand-2",
    "binding": "link",
    "event": "bell",
    "floating": "layers",
    "panel": "layers",
    "button": "circle",
    "logo": "circle",
    "tray": "inbox",
    "menu": "menu",
    "context": "menu",
    "desktop": "folder",
    "app": "folder",
    "asset": "image",
    "local": "image",
    "dcc": "box",
    "maya": "layers",
    "qt": "palette",
    "style": "palette",
    "window": "layout",
    "monitor": "monitor",
}

# Category mapping based on keywords
CATEGORY_MAPPING = {
    # Getting Started
    "simple": "getting_started",
    "decorator": "getting_started",
    "binding": "getting_started",
    "dynamic": "getting_started",
    # API Patterns
    "event": "api_patterns",
    "callback": "api_patterns",
    # Window Features
    "floating": "window_features",
    "panel": "window_features",
    "button": "window_features",
    "logo": "window_features",
    "tray": "window_features",
    "menu": "window_features",
    "context": "window_features",
    "window": "window_features",
    # Desktop Features
    "desktop": "desktop_features",
    "file": "desktop_features",
    "dialog": "desktop_features",
    "asset": "desktop_features",
    "local": "desktop_features",
    # DCC Integration
    "dcc": "dcc_integration",
    "maya": "dcc_integration",
    "houdini": "dcc_integration",
    "blender": "dcc_integration",
    "qt": "dcc_integration",
    "integration": "dcc_integration",
}

# Tag mapping based on keywords
TAG_MAPPING = {
    "beginner": ["simple", "basic", "getting started", "quick"],
    "advanced": ["advanced", "complex", "plugin", "floating", "tray"],
    "window": ["window", "panel", "frame", "transparent"],
    "events": ["event", "callback", "lifecycle"],
    "qt": ["qt", "pyside", "maya", "houdini", "nuke"],
    "standalone": ["standalone", "desktop", "run_desktop"],
    "ui": ["ui", "style", "menu", "button", "panel"],
    "api": ["api", "decorator", "binding", "call"],
}


def extract_docstring(file_path: Path) -> Optional[str]:
    """Extract module docstring from a Python file."""
    try:
        source = file_path.read_text(encoding="utf-8")
        tree = ast.parse(source)
        return ast.get_docstring(tree)
    except (SyntaxError, UnicodeDecodeError):
        return None


def parse_docstring(docstring: str) -> dict:
    """Parse docstring to extract title, description, and features."""
    lines = docstring.strip().split("\n")
    result = {
        "title": "",
        "description": "",
        "features": [],
        "use_cases": [],
    }

    if not lines:
        return result

    # First line is usually the title
    first_line = lines[0].strip()
    # Remove trailing " - " suffix if present
    if " - " in first_line:
        result["title"] = first_line.split(" - ")[0].strip()
        # Rest of first line can be part of description
        rest = first_line.split(" - ", 1)[1].strip()
        if rest:
            result["description"] = rest
    else:
        result["title"] = first_line.rstrip(".")

    # Parse remaining content
    current_section = "description"
    description_lines = []

    for line in lines[1:]:
        stripped = line.strip()
        lower = stripped.lower()

        # Detect section headers
        if lower.startswith("features") or lower.startswith("key features"):
            current_section = "features"
            continue
        elif lower.startswith("use cases") or lower.startswith("use case"):
            current_section = "use_cases"
            continue
        elif lower.startswith("usage:") or lower.startswith("note:"):
            current_section = "skip"
            continue
        elif lower.startswith("recommended") or lower.startswith("supported"):
            current_section = "skip"
            continue

        if current_section == "skip":
            continue

        # Handle list items
        if stripped.startswith("- "):
            item = stripped[2:].strip()
            if current_section == "features":
                result["features"].append(item)
            elif current_section == "use_cases":
                result["use_cases"].append(item)
        elif stripped and current_section == "description":
            description_lines.append(stripped)

    # Build description if not already set
    if description_lines and not result["description"]:
        # Take first meaningful paragraph
        result["description"] = " ".join(description_lines[:2])

    return result


def infer_category(filename: str, docstring: str) -> str:
    """Infer category based on filename and docstring content."""
    text = (filename + " " + docstring).lower()

    for keyword, category in CATEGORY_MAPPING.items():
        if keyword in text:
            return category

    return "getting_started"  # Default


def infer_icon(filename: str, docstring: str) -> str:
    """Infer icon based on filename and docstring content."""
    text = (filename + " " + docstring).lower()

    for keyword, icon in ICON_MAPPING.items():
        if keyword in text:
            return icon

    return "code"  # Default


def infer_tags(filename: str, docstring: str) -> list:
    """Infer tags based on filename and docstring content."""
    text = (filename + " " + docstring).lower()
    tags = set()

    for tag, keywords in TAG_MAPPING.items():
        for keyword in keywords:
            if keyword in text:
                tags.add(tag)
                break

    return sorted(tags)


def filename_to_title(filename: str) -> str:
    """Convert filename to a readable title."""
    # Remove extension and common suffixes
    name = filename.replace(".py", "")
    for suffix in ["_demo", "_example", "_test"]:
        name = name.replace(suffix, "")

    # Convert to title case
    words = name.replace("_", " ").split()
    return " ".join(word.capitalize() for word in words)


def filename_to_id(filename: str) -> str:
    """Convert filename to a sample ID."""
    name = filename.replace(".py", "")
    for suffix in ["_demo", "_example", "_test"]:
        name = name.replace(suffix, "")
    return name


def scan_examples() -> list:
    """Scan examples directory and build sample list from docstrings."""
    samples = []

    for py_file in sorted(EXAMPLES_DIR.glob("*.py")):
        # Skip __init__.py and non-demo files
        if py_file.name.startswith("__"):
            continue

        docstring = extract_docstring(py_file)
        if not docstring:
            continue

        # Parse docstring
        parsed = parse_docstring(docstring)

        # Build sample entry
        sample_id = filename_to_id(py_file.name)
        title = parsed["title"] or filename_to_title(py_file.name)
        description = parsed["description"] or f"Demo: {title}"
        category = infer_category(py_file.name, docstring)
        icon = infer_icon(py_file.name, docstring)
        tags = infer_tags(py_file.name, docstring)

        # Truncate description if too long
        if len(description) > 100:
            description = description[:97] + "..."

        samples.append({
            "id": sample_id,
            "title": title,
            "category": category,
            "description": description,
            "icon": icon,
            "source_file": py_file.name,
            "tags": tags,
        })

    return samples


# Scan examples on module load
SAMPLES = scan_examples()


def get_sample_by_id(sample_id: str) -> dict | None:
    """Get a sample by its ID."""
    for sample in SAMPLES:
        if sample["id"] == sample_id:
            return sample
    return None


def get_source_code(source_file: str) -> str:
    """Read source code from a sample file."""
    file_path = EXAMPLES_DIR / source_file
    if file_path.exists():
        return file_path.read_text(encoding="utf-8")
    return f"# Source file not found: {source_file}"


def run_gallery():
    """Run the AuroraView Gallery application."""
    # Log to stderr to avoid interfering with JSON-RPC on stdout in packed mode
    print("Starting AuroraView Gallery...", file=sys.stderr)
    print("=" * 50, file=sys.stderr)
    print("Interactive showcase of all features and components", file=sys.stderr)
    print("=" * 50, file=sys.stderr)

    # In packed mode, run as API server (headless)
    if PACKED_MODE:
        print("Running in packed mode (API server)", file=sys.stderr)
        run_api_server()
        return

    # Check if dist exists
    index_html = DIST_DIR / "index.html"
    if not index_html.exists():
        print("Error: Gallery not built. Run 'just gallery-build' first.")
        print(f"Expected: {index_html}")
        sys.exit(1)

    # Use local file path - normalize_url will convert to auroraview protocol
    url = str(index_html)
    print(f"Loading: {url}")

    view = WebView(
        title="AuroraView Gallery",
        url=url,
        width=1200,
        height=800,
        debug=True,
        allow_new_window=True,  # Allow window.open() to create new browser windows
    )

    # Create plugin manager with permissive scope for demo
    plugins = PluginManager.permissive()

    # Create a thread-safe event emitter for plugin callbacks
    # ProcessPlugin runs in background threads, so we need a thread-safe emitter
    emitter = view.create_emitter()

    # This enables ProcessPlugin to emit events directly to the frontend
    plugins.set_emit_callback(emitter.emit)

    # API: Get source code
    @view.bind_call("api.get_source")
    def get_source(sample_id: str = "") -> str:
        """Get source code for a sample."""
        sample = get_sample_by_id(sample_id)
        if sample:
            return get_source_code(sample["source_file"])
        return f"# Sample not found: {sample_id}"

    # API: Run sample with IPC support (using Rust ProcessPlugin)
    @view.bind_call("api.run_sample")
    def run_sample(sample_id: str = "", show_console: bool = False) -> dict:
        """Run a sample demo with IPC support.

        Uses the Rust ProcessPlugin for efficient process management.
        The process output will be streamed via events:
        - process:stdout - { pid, data }
        - process:stderr - { pid, data }
        - process:exit - { pid, code }

        Args:
            sample_id: The ID of the sample to run
            show_console: If True, show console window (for debugging)
        """
        sample = get_sample_by_id(sample_id)
        if not sample:
            return {"ok": False, "error": f"Sample not found: {sample_id}"}

        sample_path = EXAMPLES_DIR / sample["source_file"]
        if not sample_path.exists():
            return {"ok": False, "error": f"File not found: {sample['source_file']}"}

        # Use Rust ProcessPlugin for IPC-enabled spawn
        args_json = json_dumps({
            "command": sys.executable,
            "args": [str(sample_path)],
            "cwd": str(EXAMPLES_DIR),
            "showConsole": show_console,
        })
        result_json = plugins.handle_command("plugin:process|spawn_ipc", args_json)
        result = json_loads(result_json)

        if result.get("success"):
            # Extract data from PluginResponse structure
            data = result.get("data", {})
            mode = "with console" if show_console else "with IPC"
            return {
                "ok": True,
                "pid": data.get("pid"),
                "message": f"Started {sample['title']} ({mode})",
            }
        else:
            return {"ok": False, "error": result.get("error", "Unknown error")}

    # API: Kill a running process
    @view.bind_call("api.kill_process")
    def kill_process(pid: int = 0) -> dict:
        """Kill a running process by PID."""
        if not pid:
            return {"ok": False, "error": "No PID provided"}

        args_json = json_dumps({"pid": pid})
        result_json = plugins.handle_command("plugin:process|kill", args_json)
        result = json_loads(result_json)

        if result.get("success"):
            return {"ok": True}
        else:
            return {"ok": False, "error": result.get("error", "Unknown error")}

    # API: Send data to a process
    @view.bind_call("api.send_to_process")
    def send_to_process(pid: int = 0, data: str = "") -> dict:
        """Send data to a process's stdin."""
        if not pid:
            return {"ok": False, "error": "No PID provided"}

        args_json = json_dumps({"pid": pid, "data": data})
        result_json = plugins.handle_command("plugin:process|send", args_json)
        result = json_loads(result_json)

        if result.get("success"):
            return {"ok": True}
        else:
            return {"ok": False, "error": result.get("error", "Unknown error")}

    # API: List running processes
    @view.bind_call("api.list_processes")
    def list_processes() -> dict:
        """List all running processes."""
        result_json = plugins.handle_command("plugin:process|list", "{}")
        result = json_loads(result_json)

        if result.get("success"):
            data = result.get("data", {})
            return {"ok": True, "processes": data.get("processes", [])}
        else:
            return {"ok": False, "error": result.get("error", "Unknown error")}

    # API: Open URL in default browser
    @view.bind_call("api.open_url")
    def open_url(url: str = "") -> dict:
        """Open a URL in the default browser."""
        if not url:
            return {"ok": False, "error": "No URL provided"}

        try:
            args_json = json_dumps({"path": url})
            result_json = plugins.handle_command("plugin:shell|open", args_json)
            result = json_loads(result_json)
            return {"ok": result.get("success", False)}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # API: Get samples list
    @view.bind_call("api.get_samples")
    def get_samples() -> list:
        """Get all samples."""
        return SAMPLES

    # API: Get categories
    @view.bind_call("api.get_categories")
    def get_categories() -> dict:
        """Get all categories."""
        return CATEGORIES


    # Cleanup on close - kill all managed processes
    @view.on("close")
    def on_close():
        # Use kill_all to terminate all managed processes at once
        plugins.handle_command("plugin:process|kill_all", "{}")

    # Show the gallery
    view.show()


def run_api_server():
    """Run the Gallery as a headless API server for packed mode.

    In packed mode, the Rust CLI creates the WebView and loads the frontend.
    This function provides the Python API backend via JSON-RPC over stdin/stdout.
    """
    import json
    import signal
    import subprocess

    # Log to stderr to avoid interfering with JSON-RPC on stdout
    print("Gallery API Server started", file=sys.stderr)
    print("Waiting for JSON-RPC requests on stdin...", file=sys.stderr)

    # Track running processes for cleanup
    running_processes: dict[int, subprocess.Popen] = {}

    def run_sample(sample_id: str = "", show_console: bool = False) -> dict:
        """Run a sample demo."""
        sample = get_sample_by_id(sample_id)
        if not sample:
            return {"ok": False, "error": f"Sample not found: {sample_id}"}

        sample_path = EXAMPLES_DIR / sample["source_file"]
        if not sample_path.exists():
            return {"ok": False, "error": f"File not found: {sample['source_file']}"}

        try:
            # Build command
            cmd = [sys.executable, str(sample_path)]

            # Configure process creation flags
            creation_flags = 0
            if sys.platform == "win32":
                if show_console:
                    creation_flags = subprocess.CREATE_NEW_CONSOLE
                else:
                    creation_flags = subprocess.CREATE_NO_WINDOW

            # Start process
            process = subprocess.Popen(
                cmd,
                cwd=str(EXAMPLES_DIR),
                creationflags=creation_flags if sys.platform == "win32" else 0,
                stdout=subprocess.PIPE if not show_console else None,
                stderr=subprocess.PIPE if not show_console else None,
            )

            running_processes[process.pid] = process
            mode = "with console" if show_console else "background"
            print(f"Started sample {sample_id} (PID: {process.pid}, {mode})", file=sys.stderr)

            return {
                "ok": True,
                "pid": process.pid,
                "message": f"Started {sample['title']} ({mode})",
            }
        except Exception as e:
            return {"ok": False, "error": str(e)}

    def kill_process(pid: int = 0) -> dict:
        """Kill a running process by PID."""
        if not pid:
            return {"ok": False, "error": "No PID provided"}

        if pid in running_processes:
            try:
                running_processes[pid].terminate()
                del running_processes[pid]
                return {"ok": True}
            except Exception as e:
                return {"ok": False, "error": str(e)}
        else:
            # Try to kill by PID directly
            try:
                import os
                os.kill(pid, signal.SIGTERM)
                return {"ok": True}
            except Exception as e:
                return {"ok": False, "error": str(e)}


    def list_processes() -> dict:
        """List all running processes."""
        # Clean up finished processes
        finished = []
        for pid, proc in running_processes.items():
            if proc.poll() is not None:
                finished.append(pid)
        for pid in finished:
            del running_processes[pid]

        processes = [{"pid": pid} for pid in running_processes.keys()]
        return {"ok": True, "processes": processes}

    def open_url(url: str = "") -> dict:
        """Open a URL in the default browser."""
        if not url:
            return {"ok": False, "error": "No URL provided"}
        try:
            import webbrowser
            webbrowser.open(url)
            return {"ok": True}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    # API handlers
    api_handlers = {
        "api.get_samples": lambda: SAMPLES,
        "api.get_categories": lambda: CATEGORIES,
        "api.get_source": lambda sample_id="": get_source_code(
            get_sample_by_id(sample_id)["source_file"]
        ) if get_sample_by_id(sample_id) else f"# Sample not found: {sample_id}",
        "api.run_sample": run_sample,
        "api.kill_process": kill_process,
        "api.list_processes": list_processes,
        "api.open_url": open_url,
    }

    def handle_request(request: dict) -> dict:
        """Handle a JSON-RPC request."""
        req_id = request.get("id")
        method = request.get("method", "")
        params = request.get("params")

        if method in api_handlers:
            try:
                handler = api_handlers[method]
                # Handle different param types
                # Note: JSON null becomes Python None
                if params is None or params == {}:
                    result = handler()
                elif isinstance(params, dict):
                    result = handler(**params)
                elif isinstance(params, list):
                    result = handler(*params)
                else:
                    # Single value parameter - but our handlers don't expect this
                    result = handler()
                return {"id": req_id, "ok": True, "result": result}
            except Exception as e:
                return {"id": req_id, "ok": False, "error": str(e)}
        else:
            return {"id": req_id, "ok": False, "error": f"Unknown method: {method}"}

    # Handle SIGTERM gracefully
    running = True

    def signal_handler(signum, frame):
        nonlocal running
        print("Received shutdown signal", file=sys.stderr)
        running = False

    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGINT, signal_handler)

    # Main loop: read JSON-RPC requests from stdin
    while running:
        try:
            line = sys.stdin.readline()
            if not line:
                break

            line = line.strip()
            if not line:
                continue

            request = json.loads(line)
            response = handle_request(request)
            print(json.dumps(response), flush=True)

        except json.JSONDecodeError as e:
            print(json.dumps({"ok": False, "error": f"Invalid JSON: {e}"}), flush=True)
        except Exception as e:
            print(json.dumps({"ok": False, "error": str(e)}), flush=True)

    # Cleanup: kill all running processes
    for pid, proc in running_processes.items():
        try:
            proc.terminate()
            print(f"Terminated process {pid}", file=sys.stderr)
        except Exception:
            pass

    print("Gallery API Server stopped", file=sys.stderr)


if __name__ == "__main__":
    run_gallery()
