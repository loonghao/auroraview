# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""MCP Sidecar Process Manager.

This module provides the McpSidecar class for managing the MCP sidecar process
that handles MCP protocol communication with external AI agents.

The sidecar process runs independently and communicates with the main process
via IPC (Named Pipe on Windows, Unix Socket on Unix).

Compatible with Python 3.7+.
"""

from __future__ import annotations

import logging
import os
import platform
import secrets
import subprocess
import sys
import threading
import time

logger = logging.getLogger(__name__)


def _generate_channel_name():
    # type: () -> str
    """Generate a unique IPC channel name."""
    pid = os.getpid()
    nonce = secrets.token_hex(8)
    return "auroraview_mcp_{}_{}".format(pid, nonce)


def _generate_auth_token():
    # type: () -> str
    """Generate a secure authentication token."""
    return secrets.token_urlsafe(32)


class McpSidecar:
    """Manager for the MCP Sidecar process.

    The sidecar process handles MCP protocol communication, allowing external
    AI agents to call tools registered in the main AuroraView process.

    This class manages:
    1. The Rust IPC Server (via SidecarBridge) that handles tool calls
    2. The sidecar process that exposes tools via MCP protocol

    Example:
        >>> sidecar = McpSidecar()
        >>> sidecar.register_tool("echo", "Echo back input", lambda args: args)
        >>> port = sidecar.start()
        >>> print(f"MCP server running on port {port}")
        >>> # ... use the MCP server ...
        >>> sidecar.stop()
    """

    def __init__(
        self,
        port=0,  # type: int
        log_level="info",  # type: str
        on_tool_call=None,  # type: Optional[Callable[[str, Dict], Any]]
    ):
        # type: (...) -> None
        """Initialize the sidecar manager.

        Args:
            port: MCP server port (0 for auto-assign).
            log_level: Log level for sidecar process (trace/debug/info/warn/error).
            on_tool_call: Callback for handling tool calls from the sidecar.
        """
        self._port = port
        self._log_level = log_level
        self._on_tool_call = on_tool_call

        self._process = None  # type: Optional[subprocess.Popen]
        self._actual_port = 0
        self._started = False
        self._lock = threading.Lock()

        # Tool registry (Python side)
        self._tools = {}  # type: Dict[str, Dict[str, Any]]

        # Rust IPC Server bridge (lazy initialization)
        self._bridge = None  # type: Optional[Any]
        self._bridge_available = self._check_bridge_available()

        # Log forwarding threads
        self._stdout_thread = None  # type: Optional[threading.Thread]
        self._stderr_thread = None  # type: Optional[threading.Thread]
        self._stop_log_threads = False

    @staticmethod
    def _check_bridge_available():
        # type: () -> bool
        """Check if Rust SidecarBridge is available."""
        try:
            from auroraview.core import SidecarBridge

            # SidecarBridge is set to None if the feature is not enabled
            return SidecarBridge is not None
        except ImportError:
            return False

    def _get_bridge(self):
        # type: () -> Any
        """Get or create the Rust SidecarBridge."""
        if self._bridge is None and self._bridge_available:
            from auroraview.core import SidecarBridge

            self._bridge = SidecarBridge()
            # Register all existing tools with the bridge
            for name, tool in self._tools.items():
                self._bridge.register_tool(name, tool["description"], tool["handler"])
        return self._bridge

    @property
    def channel_name(self):
        # type: () -> str
        """Get the IPC channel name."""
        if self._bridge:
            return self._bridge.channel_name
        return ""

    @property
    def auth_token(self):
        # type: () -> str
        """Get the IPC authentication token."""
        if self._bridge:
            return self._bridge.auth_token
        return ""

    @staticmethod
    def get_binary_path():
        # type: () -> Optional[str]
        """Locate the sidecar binary.

        Search order:
        1. Same directory as the auroraview package
        2. AURORAVIEW_MCP_SERVER environment variable
        3. System PATH

        Returns:
            Path to the binary, or None if not found.
        """
        binary_name = "auroraview-mcp-server"
        if platform.system() == "Windows":
            binary_name += ".exe"

        # 1. Check same directory as this module
        module_dir = os.path.dirname(os.path.abspath(__file__))
        local_path = os.path.join(module_dir, binary_name)
        if os.path.isfile(local_path):
            return local_path

        # Also check parent directory (auroraview/)
        parent_dir = os.path.dirname(module_dir)
        parent_path = os.path.join(parent_dir, binary_name)
        if os.path.isfile(parent_path):
            return parent_path

        # 2. Check environment variable
        env_path = os.environ.get("AURORAVIEW_MCP_SERVER")
        if env_path and os.path.isfile(env_path):
            return env_path

        # 3. Check cargo target directory (development)
        # Go up from python/auroraview/mcp to project root
        project_root = os.path.dirname(os.path.dirname(os.path.dirname(module_dir)))
        cargo_debug = os.path.join(project_root, "target", "debug", binary_name)
        if os.path.isfile(cargo_debug):
            return cargo_debug

        cargo_release = os.path.join(project_root, "target", "release", binary_name)
        if os.path.isfile(cargo_release):
            return cargo_release

        # 4. Try to find in PATH
        import shutil

        path_binary = shutil.which(binary_name)
        if path_binary:
            return path_binary

        return None

    def register_tool(
        self,
        name,  # type: str
        description,  # type: str
        handler,  # type: Callable
        input_schema=None,  # type: Optional[Dict]
    ):
        # type: (...) -> None
        """Register a tool that can be called via MCP.

        Args:
            name: Tool name (unique identifier).
            description: Human-readable description.
            handler: Function to call when tool is invoked.
            input_schema: JSON Schema for input parameters.
        """
        self._tools[name] = {
            "name": name,
            "description": description,
            "handler": handler,
            "input_schema": input_schema or {"type": "object", "properties": {}},
        }
        logger.debug("Registered tool: %s", name)

    def start(self):
        # type: () -> int
        """Start the sidecar process.

        This method:
        1. Starts the IPC Server (via SidecarBridge) to handle tool calls
        2. Starts the sidecar process that exposes tools via MCP protocol

        Returns:
            The port number the MCP server is listening on.

        Raises:
            RuntimeError: If sidecar binary not found or failed to start.
        """
        with self._lock:
            if self._started:
                logger.warning("Sidecar already started on port %d", self._actual_port)
                return self._actual_port

            binary_path = self.get_binary_path()
            if not binary_path:
                raise RuntimeError(
                    "MCP sidecar binary not found. Ensure auroraview-mcp-server "
                    "is built and available."
                )

            # Step 1: Start IPC Server via SidecarBridge
            # This creates the IPC channel that the sidecar will connect to
            bridge = self._get_bridge()
            if bridge is None:
                raise RuntimeError(
                    "SidecarBridge not available. Ensure auroraview is built "
                    "with mcp-sidecar feature."
                )

            # Register tools with the bridge before starting
            for name, tool in self._tools.items():
                try:
                    bridge.register_tool(name, tool["description"], tool["handler"])
                except Exception as e:
                    logger.warning("Failed to register tool %s: %s", name, e)

            # Start the IPC Server in a background thread
            bridge.start()
            logger.info("IPC Server started: channel=%s", bridge.channel_name)

            # Step 2: Start the sidecar process
            parent_pid = os.getpid()

            # Build command using bridge's credentials
            cmd = [
                binary_path,
                "--port",
                str(self._port),
                "--ipc",
                bridge.channel_name,
                "--token",
                bridge.auth_token,
                "--parent-pid",
                str(parent_pid),
                "--log-level",
                self._log_level,
            ]

            logger.info("Starting MCP sidecar: %s", " ".join(cmd))

            # Start process with explicit DEVNULL to avoid PIPE blocking
            # This is important for DCC environments
            try:
                self._process = subprocess.Popen(
                    cmd,
                    stdin=subprocess.DEVNULL,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    creationflags=self._get_creation_flags(),
                )
            except OSError as e:
                bridge.stop()  # Cleanup on failure
                raise RuntimeError("Failed to start sidecar: {}".format(e))

            # Wait for sidecar to report ready via stdout
            # The sidecar prints the port when ready
            try:
                self._actual_port = self._wait_for_ready()
            except RuntimeError:
                bridge.stop()  # Cleanup on failure
                raise

            self._started = True

            # Start log forwarding threads
            self._stop_log_threads = False
            self._start_log_forwarding()

            logger.info("MCP sidecar started on port %d", self._actual_port)
            return self._actual_port

    def _start_log_forwarding(self):
        # type: () -> None
        """Start background threads to forward sidecar logs to Python logger."""
        if self._process is None:
            return

        def forward_stream(stream, prefix, level):
            # type: (Any, str, int) -> None
            """Read from stream and forward to logger."""
            try:
                while not self._stop_log_threads and stream:
                    line = stream.readline()
                    if not line:
                        break
                    text = line.decode("utf-8", errors="replace").strip()
                    if text:
                        # Print to stderr for immediate visibility
                        print("[Sidecar:{}] {}".format(prefix, text), file=sys.stderr)
                        logger.log(level, "[Sidecar:%s] %s", prefix, text)
            except Exception as e:
                if not self._stop_log_threads:
                    logger.debug("Log forwarding ended: %s", e)

        # Forward stdout (usually INFO level logs)
        if self._process.stdout:
            self._stdout_thread = threading.Thread(
                target=forward_stream,
                args=(self._process.stdout, "OUT", logging.INFO),
                daemon=True,
                name="sidecar-stdout",
            )
            self._stdout_thread.start()

        # Forward stderr (usually ERROR/WARN level logs)
        if self._process.stderr:
            self._stderr_thread = threading.Thread(
                target=forward_stream,
                args=(self._process.stderr, "ERR", logging.WARNING),
                daemon=True,
                name="sidecar-stderr",
            )
            self._stderr_thread.start()

    def _get_creation_flags(self):
        # type: () -> int
        """Get platform-specific process creation flags."""
        if platform.system() == "Windows":
            # CREATE_NO_WINDOW - prevents console window from appearing
            return 0x08000000
        return 0

    def _wait_for_ready(self, timeout=10.0):
        # type: (float) -> int
        """Wait for the sidecar to report ready.

        Args:
            timeout: Maximum time to wait in seconds.

        Returns:
            The port number reported by the sidecar.

        Raises:
            RuntimeError: If timeout or sidecar exited.
        """
        if self._process is None:
            raise RuntimeError("Process not started")

        start_time = time.time()
        while time.time() - start_time < timeout:
            # Check if process is still alive
            if self._process.poll() is not None:
                stderr = self._process.stderr.read().decode("utf-8", errors="replace")
                raise RuntimeError(
                    "Sidecar exited unexpectedly (code {}): {}".format(
                        self._process.returncode, stderr
                    )
                )

            # Try to read a line from stdout
            if self._process.stdout:
                line = self._process.stdout.readline()
                if line:
                    text = line.decode("utf-8", errors="replace").strip()
                    # Print startup logs to stderr for visibility
                    print("[Sidecar:INIT] {}".format(text), file=sys.stderr)
                    logger.debug("Sidecar: %s", text)
                    # Look for port in log output
                    if "port" in text.lower():
                        # Try to extract port number
                        import re

                        match = re.search(r"port\s+(\d+)", text, re.IGNORECASE)
                        if match:
                            return int(match.group(1))

            time.sleep(0.1)

        raise RuntimeError("Timeout waiting for sidecar to start")

    def stop(self, timeout=5.0):
        # type: (float) -> None
        """Stop the sidecar process and IPC Server.

        First attempts graceful shutdown, then forcefully terminates if needed.

        Args:
            timeout: Maximum time to wait for graceful shutdown.
        """
        with self._lock:
            if not self._started:
                return

            logger.info("Stopping MCP sidecar...")

            # Signal log threads to stop
            self._stop_log_threads = True

            # Stop sidecar process first
            if self._process is not None:
                try:
                    self._process.terminate()
                    self._process.wait(timeout=timeout)
                    logger.debug("Sidecar terminated gracefully")
                except subprocess.TimeoutExpired:
                    logger.warning("Sidecar did not terminate gracefully, killing...")
                    self._process.kill()
                    self._process.wait(timeout=2.0)

            # Wait for log threads to finish
            if self._stdout_thread and self._stdout_thread.is_alive():
                self._stdout_thread.join(timeout=1.0)
            if self._stderr_thread and self._stderr_thread.is_alive():
                self._stderr_thread.join(timeout=1.0)

            # Stop IPC Server
            if self._bridge is not None:
                try:
                    self._bridge.stop()
                    logger.debug("IPC Server stopped")
                except Exception as e:
                    logger.warning("Failed to stop IPC Server: %s", e)

            self._started = False
            self._actual_port = 0
            self._process = None
            self._stdout_thread = None
            self._stderr_thread = None
            logger.info("MCP sidecar stopped")

    def is_alive(self):
        # type: () -> bool
        """Check if the sidecar process is running.

        Returns:
            True if the process is alive.
        """
        with self._lock:
            if not self._started or self._process is None:
                return False
            return self._process.poll() is None

    @property
    def port(self):
        # type: () -> int
        """Get the MCP server port.

        Returns:
            Port number, or 0 if not started.
        """
        return self._actual_port

    @property
    def tools(self):
        # type: () -> List[Dict[str, Any]]
        """Get registered tool definitions.

        Returns:
            List of tool definition dicts.
        """
        return [
            {
                "name": t["name"],
                "description": t["description"],
                "input_schema": t["input_schema"],
            }
            for t in self._tools.values()
        ]

    def __enter__(self):
        # type: () -> McpSidecar
        """Context manager entry."""
        self.start()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        # type: (Any, Any, Any) -> None
        """Context manager exit."""
        self.stop()

    def __del__(self):
        # type: () -> None
        """Destructor - ensure cleanup."""
        try:
            self.stop()
        except Exception:
            pass
