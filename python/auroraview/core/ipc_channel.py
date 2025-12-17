"""IPC Channel Client for AuroraView subprocess communication.

This module provides a client for connecting to AuroraView's IPC channel
when running as a subprocess spawned via `spawn_ipc_channel`.

The IPC channel uses ipckit LocalSocket for high-performance bidirectional
JSON messaging between the parent process (AuroraView/Gallery) and child
Python scripts.

Example:
    >>> from auroraview.core.ipc_channel import IpcChannel
    >>>
    >>> # Connect to parent process (reads AURORAVIEW_IPC_CHANNEL env var)
    >>> with IpcChannel.connect() as channel:
    ...     # Send structured messages to parent
    ...     channel.send({"type": "progress", "value": 50})
    ...
    ...     # Receive messages from parent
    ...     msg = channel.receive()
    ...     if msg:
    ...         print(f"Received: {msg}")
    ...
    ...     # Send final result
    ...     channel.send({"type": "result", "data": "completed"})
"""

from __future__ import annotations

import logging
import os
import socket
import sys
import threading
from typing import Any, Callable, Dict, Optional

# Use Rust-powered JSON for better performance
try:
    from auroraview._core import json_dumps, json_loads
except ImportError:
    # Fallback to Python json if Rust core not available
    import json

    def json_dumps(obj: Any) -> str:
        return json.dumps(obj, ensure_ascii=False)

    def json_loads(s: str) -> Any:
        return json.loads(s)


logger = logging.getLogger(__name__)

# Windows Named Pipe prefix (ipckit uses this format)
WINDOWS_PIPE_PREFIX = r"\\.\pipe\ipckit_"


class IpcChannelError(Exception):
    """Exception raised for IPC channel errors."""

    pass


class IpcChannel:
    """IPC Channel client for subprocess communication.

    This class connects to the parent process's LocalSocket channel
    for bidirectional JSON messaging.

    Attributes:
        channel_name: The channel name (from AURORAVIEW_IPC_CHANNEL)
        _socket: The underlying socket connection
        _connected: Whether the channel is connected
        _lock: Thread lock for socket operations
    """

    def __init__(self, channel_name: str):
        """Initialize IPC Channel.

        Args:
            channel_name: The channel name to connect to
        """
        self.channel_name = channel_name
        self._socket: Optional[socket.socket] = None
        self._file: Any = None  # File-like wrapper for line reading
        self._connected = False
        self._lock = threading.Lock()
        self._receive_handlers: list[Callable[[Dict[str, Any]], None]] = []
        self._receive_thread: Optional[threading.Thread] = None
        self._running = False

    @classmethod
    def connect(cls, channel_name: Optional[str] = None) -> "IpcChannel":
        """Connect to the IPC channel.

        If channel_name is not provided, reads from AURORAVIEW_IPC_CHANNEL
        environment variable.

        Args:
            channel_name: Optional channel name override

        Returns:
            Connected IpcChannel instance

        Raises:
            IpcChannelError: If connection fails or channel not available
        """
        if channel_name is None:
            channel_name = os.environ.get("AURORAVIEW_IPC_CHANNEL")
            if not channel_name:
                raise IpcChannelError(
                    "AURORAVIEW_IPC_CHANNEL environment variable not set. "
                    "This script must be spawned via spawn_ipc_channel."
                )

        channel = cls(channel_name)
        channel._connect()
        return channel

    @classmethod
    def is_available(cls) -> bool:
        """Check if IPC channel is available.

        Returns:
            True if AURORAVIEW_IPC_CHANNEL is set and mode is 'channel'
        """
        channel = os.environ.get("AURORAVIEW_IPC_CHANNEL")
        mode = os.environ.get("AURORAVIEW_IPC_MODE")
        return bool(channel) and mode == "channel"

    def _connect(self) -> None:
        """Establish connection to the LocalSocket.

        On Windows, ipckit uses Named Pipes with format:
        \\.\\pipe\\ipckit_{channel_name}
        """
        if self._connected:
            return

        try:
            if sys.platform == "win32":
                # Windows: Connect to Named Pipe
                pipe_path = f"{WINDOWS_PIPE_PREFIX}{self.channel_name}"
                logger.debug(f"[IpcChannel] Connecting to Windows pipe: {pipe_path}")

                # Use socket-like interface for Named Pipes via file operations
                # Python's socket module doesn't support Named Pipes directly,
                # so we use open() with the pipe path

                # Open the named pipe as a file
                self._file = open(pipe_path, "r+b", buffering=0)
                self._connected = True
                logger.info(f"[IpcChannel] Connected to pipe: {pipe_path}")
            else:
                # Unix: Connect to Unix Domain Socket
                socket_path = f"/tmp/ipckit_{self.channel_name}"
                logger.debug(f"[IpcChannel] Connecting to Unix socket: {socket_path}")

                self._socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                self._socket.connect(socket_path)
                self._file = self._socket.makefile("rw", buffering=1)
                self._connected = True
                logger.info(f"[IpcChannel] Connected to socket: {socket_path}")

        except FileNotFoundError as e:
            raise IpcChannelError(
                f"IPC channel not found: {self.channel_name}. "
                f"Ensure parent process created the channel. Error: {e}"
            )
        except Exception as e:
            raise IpcChannelError(f"Failed to connect to IPC channel: {e}")

    def send(self, data: Dict[str, Any]) -> bool:
        """Send a JSON message to the parent process.

        Args:
            data: Dictionary to send (will be JSON serialized)

        Returns:
            True if sent successfully

        Raises:
            IpcChannelError: If send fails
        """
        if not self._connected:
            raise IpcChannelError("Channel not connected")

        try:
            with self._lock:
                json_str = json_dumps(data) + "\n"
                json_bytes = json_str.encode("utf-8")

                if sys.platform == "win32":
                    self._file.write(json_bytes)
                    self._file.flush()
                else:
                    self._file.write(json_str)
                    self._file.flush()

                logger.debug(f"[IpcChannel] Sent: {data}")
                return True

        except Exception as e:
            logger.error(f"[IpcChannel] Send error: {e}")
            raise IpcChannelError(f"Failed to send message: {e}")

    def receive(self, timeout: Optional[float] = None) -> Optional[Dict[str, Any]]:
        """Receive a JSON message from the parent process.

        Args:
            timeout: Optional timeout in seconds (None for blocking)

        Returns:
            Parsed JSON dictionary, or None if no message available

        Raises:
            IpcChannelError: If receive fails
        """
        if not self._connected:
            raise IpcChannelError("Channel not connected")

        try:
            with self._lock:
                if sys.platform == "win32":
                    # Windows: Read line from pipe
                    line = b""
                    while True:
                        char = self._file.read(1)
                        if not char:
                            return None
                        if char == b"\n":
                            break
                        line += char
                    json_str = line.decode("utf-8")
                else:
                    # Unix: Read line from socket file
                    json_str = self._file.readline()
                    if not json_str:
                        return None
                    json_str = json_str.strip()

                if not json_str:
                    return None

                data = json_loads(json_str)
                logger.debug(f"[IpcChannel] Received: {data}")
                return data

        except (ValueError, TypeError) as e:
            logger.warning(f"[IpcChannel] Invalid JSON received: {e}")
            return None
        except Exception as e:
            logger.error(f"[IpcChannel] Receive error: {e}")
            raise IpcChannelError(f"Failed to receive message: {e}")

    def on_message(self, handler: Callable[[Dict[str, Any]], None]) -> None:
        """Register a handler for incoming messages.

        Args:
            handler: Function to call with received messages
        """
        self._receive_handlers.append(handler)

    def start_receiving(self) -> None:
        """Start background thread to receive messages.

        Messages will be dispatched to registered handlers.
        """
        if self._running:
            return

        self._running = True
        self._receive_thread = threading.Thread(
            target=self._receive_loop, daemon=True, name="IpcChannel-Receiver"
        )
        self._receive_thread.start()
        logger.debug("[IpcChannel] Started receive thread")

    def stop_receiving(self) -> None:
        """Stop the background receive thread."""
        self._running = False
        if self._receive_thread:
            self._receive_thread.join(timeout=1.0)
            self._receive_thread = None

    def _receive_loop(self) -> None:
        """Background receive loop."""
        while self._running and self._connected:
            try:
                msg = self.receive(timeout=0.1)
                if msg and self._receive_handlers:
                    for handler in self._receive_handlers:
                        try:
                            handler(msg)
                        except Exception as e:
                            logger.error(f"[IpcChannel] Handler error: {e}")
            except IpcChannelError:
                break
            except Exception as e:
                logger.debug(f"[IpcChannel] Receive loop error: {e}")
                break

    def close(self) -> None:
        """Close the IPC channel."""
        self._running = False
        self._connected = False

        if self._receive_thread:
            self._receive_thread.join(timeout=1.0)
            self._receive_thread = None

        if self._file:
            try:
                self._file.close()
            except Exception:
                pass
            self._file = None

        if self._socket:
            try:
                self._socket.close()
            except Exception:
                pass
            self._socket = None

        logger.debug("[IpcChannel] Closed")

    def __enter__(self) -> "IpcChannel":
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """Context manager exit."""
        self.close()

    def __repr__(self) -> str:
        """String representation."""
        status = "connected" if self._connected else "disconnected"
        return f"IpcChannel({self.channel_name}, {status})"


# Convenience functions for simple use cases


def send_to_parent(data: Dict[str, Any]) -> bool:
    """Send a message to the parent process.

    This is a convenience function for one-shot message sending.
    For multiple messages, use IpcChannel.connect() directly.

    Args:
        data: Dictionary to send

    Returns:
        True if sent successfully, False if channel not available
    """
    if not IpcChannel.is_available():
        logger.warning("[IpcChannel] Channel not available, message not sent")
        return False

    try:
        with IpcChannel.connect() as channel:
            return channel.send(data)
    except IpcChannelError as e:
        logger.error(f"[IpcChannel] Failed to send: {e}")
        return False


def emit_event(event: str, data: Optional[Dict[str, Any]] = None) -> bool:
    """Emit an event to the parent process.

    This sends a structured event message that the parent can handle.

    Args:
        event: Event name
        data: Optional event data

    Returns:
        True if sent successfully
    """
    return send_to_parent({"type": "event", "event": event, "data": data or {}})


def report_progress(value: int, message: Optional[str] = None) -> bool:
    """Report progress to the parent process.

    Args:
        value: Progress value (0-100)
        message: Optional progress message

    Returns:
        True if sent successfully
    """
    payload = {"type": "progress", "value": value}
    if message:
        payload["message"] = message
    return send_to_parent(payload)


def report_result(success: bool, data: Any = None, error: Optional[str] = None) -> bool:
    """Report final result to the parent process.

    Args:
        success: Whether operation succeeded
        data: Result data (if success)
        error: Error message (if failure)

    Returns:
        True if sent successfully
    """
    payload = {"type": "result", "success": success}
    if success:
        payload["data"] = data
    else:
        payload["error"] = error
    return send_to_parent(payload)
