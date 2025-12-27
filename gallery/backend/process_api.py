"""Process management API handlers for AuroraView Gallery.

This module provides API handlers for:
- Running sample demos with IPC support
- Killing running processes
- Sending data to processes
- Listing running processes
"""

from __future__ import annotations

import os
import sys
from typing import TYPE_CHECKING

from auroraview import json_dumps, json_loads

from .config import EXAMPLES_DIR
from .samples import get_sample_by_id

if TYPE_CHECKING:
    from auroraview import PluginManager, WebView


def register_process_apis(view: WebView, plugins: PluginManager):
    """Register all process management API handlers.

    Args:
        view: The WebView instance to register handlers on
        plugins: The PluginManager instance for plugin commands

    Returns:
        dict: A dictionary containing references to key API functions
    """
    # Store reference to run_sample for extension bridge
    api_refs = {}

    @view.bind_call("api.get_source")
    def get_source(sample_id: str = "") -> str:
        """Get source code for a sample."""
        from .samples import get_source_code

        sample = get_sample_by_id(sample_id)
        if sample:
            return get_source_code(sample["source_file"])
        return f"# Sample not found: {sample_id}"

    @view.bind_call("api.prepare_run_sample")
    def prepare_run_sample(
        sample_id: str = "", show_console: bool = False, use_channel: bool = False
    ) -> dict:
        """Prepare arguments for running a sample demo.

        Returns spawn arguments that JavaScript should use to call ProcessPlugin directly.
        This ensures ProcessPlugin runs in Rust CLI where events are properly forwarded.

        Args:
            sample_id: The ID of the sample to run
            show_console: If True, show console window (for debugging)
            use_channel: If True, use ipckit LocalSocket for IPC (more efficient)

        Returns:
            dict with:
            - ok: True if preparation succeeded
            - spawn_args: Arguments to pass to plugin:process|spawn_ipc
            - ipc_command: The plugin command to use (spawn_ipc or spawn_ipc_channel)
            - title: Sample title for display
        """
        print(
            f"[Python:prepare_run_sample] sample_id={sample_id}, show_console={show_console}, use_channel={use_channel}",
            file=sys.stderr,
        )

        sample = get_sample_by_id(sample_id)
        if not sample:
            error_msg = f"Sample not found: {sample_id}"
            print(f"[Python:prepare_run_sample] ERROR: {error_msg}", file=sys.stderr)
            return {"ok": False, "error": error_msg}

        sample_path = EXAMPLES_DIR / sample["source_file"]
        print(f"[Python:prepare_run_sample] sample_path={sample_path}", file=sys.stderr)

        if not sample_path.exists():
            error_msg = f"File not found: {sample['source_file']} (full path: {sample_path})"
            print(f"[Python:prepare_run_sample] ERROR: {error_msg}", file=sys.stderr)
            return {"ok": False, "error": error_msg}

        # Get Python executable - in packed mode, prefer AURORAVIEW_PYTHON_EXE if set
        python_exe = os.environ.get("AURORAVIEW_PYTHON_EXE", sys.executable)
        print(f"[Python:prepare_run_sample] Python executable: {python_exe}", file=sys.stderr)
        print(f"[Python:prepare_run_sample] Working directory: {EXAMPLES_DIR}", file=sys.stderr)

        # Prepare spawn arguments
        spawn_args = {
            "command": python_exe,
            "args": [str(sample_path)],
            "cwd": str(EXAMPLES_DIR),
            "showConsole": show_console,
        }

        # Choose IPC mode: channel (ipckit LocalSocket) or pipe (stdout/stderr)
        ipc_command = (
            "plugin:process|spawn_ipc_channel" if use_channel else "plugin:process|spawn_ipc"
        )

        print(f"[Python:prepare_run_sample] Prepared spawn_args: {spawn_args}", file=sys.stderr)
        print(f"[Python:prepare_run_sample] IPC command: {ipc_command}", file=sys.stderr)

        return {
            "ok": True,
            "spawn_args": spawn_args,
            "ipc_command": ipc_command,
            "title": sample["title"],
        }

    @view.bind_call("api.run_sample")
    def run_sample(
        sample_id: str = "", show_console: bool = False, use_channel: bool = False
    ) -> dict:
        """Run a sample demo with IPC support.

        NOTE: In packed mode, this runs ProcessPlugin in the Python process,
        which requires event forwarding via stdout. For better performance,
        use prepare_run_sample + JavaScript plugin call instead.

        Uses the Rust ProcessPlugin for efficient process management.
        The process output will be streamed via events:
        - process:stdout - { pid, data }
        - process:stderr - { pid, data }
        - process:exit - { pid, code }
        - process:message - { pid, data } (only with use_channel=True)

        Args:
            sample_id: The ID of the sample to run
            show_console: If True, show console window (for debugging)
            use_channel: If True, use ipckit LocalSocket for IPC (more efficient)
        """
        print(
            f"[Python:run_sample] sample_id={sample_id}, show_console={show_console}, use_channel={use_channel}",
            file=sys.stderr,
        )

        sample = get_sample_by_id(sample_id)
        if not sample:
            error_msg = f"Sample not found: {sample_id}"
            print(f"[Python:run_sample] ERROR: {error_msg}", file=sys.stderr)
            return {"ok": False, "error": error_msg}

        sample_path = EXAMPLES_DIR / sample["source_file"]
        print(f"[Python:run_sample] sample_path={sample_path}", file=sys.stderr)

        if not sample_path.exists():
            error_msg = f"File not found: {sample['source_file']} (full path: {sample_path})"
            print(f"[Python:run_sample] ERROR: {error_msg}", file=sys.stderr)
            return {"ok": False, "error": error_msg}

        # Log Python executable being used
        # In packed mode, prefer AURORAVIEW_PYTHON_EXE if set
        python_exe = os.environ.get("AURORAVIEW_PYTHON_EXE", sys.executable)
        print(f"[Python:run_sample] Python executable: {python_exe}", file=sys.stderr)
        print(f"[Python:run_sample] Working directory: {EXAMPLES_DIR}", file=sys.stderr)
        print(
            f"[Python:run_sample] AURORAVIEW_PYTHON_PATH: {os.environ.get('AURORAVIEW_PYTHON_PATH', 'not set')}",
            file=sys.stderr,
        )

        try:
            # Use Rust ProcessPlugin for IPC-enabled spawn
            args_json = json_dumps(
                {
                    "command": python_exe,
                    "args": [str(sample_path)],
                    "cwd": str(EXAMPLES_DIR),
                    "showConsole": show_console,
                }
            )
            print(f"[Python:run_sample] Spawning with args: {args_json}", file=sys.stderr)

            # Choose IPC mode: channel (ipckit LocalSocket) or pipe (stdout/stderr)
            ipc_command = (
                "plugin:process|spawn_ipc_channel" if use_channel else "plugin:process|spawn_ipc"
            )
            print(f"[Python:run_sample] Using IPC command: {ipc_command}", file=sys.stderr)

            result_json = plugins.handle_command(ipc_command, args_json)
            result = json_loads(result_json)
            print(f"[Python:run_sample] Result: {result}", file=sys.stderr)

            if result.get("success"):
                # Extract data from PluginResponse structure
                data = result.get("data", {})
                mode = data.get("mode", "pipe")
                pid = data.get("pid")
                channel_name = data.get("channel")
                print(
                    f"[Python:run_sample] SUCCESS: Started with PID {pid}, mode={mode}",
                    file=sys.stderr,
                )
                response = {
                    "ok": True,
                    "pid": pid,
                    "mode": mode,
                    "message": f"Started {sample['title']} (mode: {mode})",
                }
                if channel_name:
                    response["channel"] = channel_name
                return response
            else:
                error_msg = result.get("error", "Unknown error from ProcessPlugin")
                print(f"[Python:run_sample] ERROR from plugin: {error_msg}", file=sys.stderr)
                return {"ok": False, "error": error_msg}
        except Exception as e:
            error_msg = f"Exception while spawning: {e}"
            print(f"[Python:run_sample] EXCEPTION: {error_msg}", file=sys.stderr)
            import traceback

            traceback.print_exc(file=sys.stderr)
            return {"ok": False, "error": error_msg}

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

    @view.bind_call("api.send_to_process")
    def send_to_process(pid: int = 0, data: str = "") -> dict:
        """Send data to a process's stdin (pipe mode only)."""
        if not pid:
            return {"ok": False, "error": "No PID provided"}

        args_json = json_dumps({"pid": pid, "data": data})
        result_json = plugins.handle_command("plugin:process|send", args_json)
        result = json_loads(result_json)

        if result.get("success"):
            return {"ok": True}
        else:
            return {"ok": False, "error": result.get("error", "Unknown error")}

    @view.bind_call("api.send_json_to_process")
    def send_json_to_process(pid: int = 0, data: dict = None) -> dict:
        """Send JSON data to a process via IPC channel.

        This only works for processes spawned with use_channel=True.
        For pipe mode processes, use send_to_process instead.

        Args:
            pid: Process ID
            data: JSON-serializable data to send
        """
        if not pid:
            return {"ok": False, "error": "No PID provided"}
        if data is None:
            data = {}

        args_json = json_dumps({"pid": pid, "data": data})
        result_json = plugins.handle_command("plugin:process|send_json", args_json)
        result = json_loads(result_json)

        if result.get("success"):
            return {"ok": True}
        else:
            return {"ok": False, "error": result.get("error", "Unknown error")}

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

    @view.bind_call("api.open_url")
    def open_url(url: str = "") -> dict:
        """Open a URL in the default browser."""
        print(f"[Python:open_url] url={url}", file=sys.stderr)

        if not url:
            error_msg = "No URL provided"
            print(f"[Python:open_url] ERROR: {error_msg}", file=sys.stderr)
            return {"ok": False, "error": error_msg}

        try:
            args_json = json_dumps({"path": url})
            result_json = plugins.handle_command("plugin:shell|open", args_json)
            result = json_loads(result_json)
            print(f"[Python:open_url] Result: {result}", file=sys.stderr)

            if result.get("success"):
                print(f"[Python:open_url] SUCCESS: Opened {url}", file=sys.stderr)
                return {"ok": True}
            else:
                error_msg = result.get("error", "Failed to open URL")
                print(f"[Python:open_url] ERROR: {error_msg}", file=sys.stderr)
                return {"ok": False, "error": error_msg}
        except Exception as e:
            error_msg = f"Exception while opening URL: {e}"
            print(f"[Python:open_url] EXCEPTION: {error_msg}", file=sys.stderr)
            return {"ok": False, "error": error_msg}

    # Store run_sample reference for extension bridge
    api_refs["run_sample"] = run_sample
    return api_refs
