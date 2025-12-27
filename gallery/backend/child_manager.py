"""Child Window Manager for Gallery.

This module manages child windows (examples running as sub-windows of Gallery).
It provides:
- IPC server for child communication
- Child window lifecycle management
- Event forwarding between parent and children

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import threading
import time
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional

if TYPE_CHECKING:
    from auroraview import WebView

# Import environment variable names from child module
from auroraview.child import (
    ENV_CHILD_ID,
    ENV_EXAMPLE_NAME,
    ENV_PARENT_ID,
    ENV_PARENT_PORT,
)


@dataclass
class ChildWindow:
    """Information about a child window."""

    child_id: str
    example_name: str
    process: Optional[subprocess.Popen] = None
    socket: Optional[socket.socket] = None
    connected: bool = False
    ready: bool = False
    created_at: float = field(default_factory=time.time)


class ChildWindowManager:
    """Manages child windows for Gallery.

    This class handles:
    - Starting examples as child windows
    - IPC communication with children
    - Event forwarding
    - Child lifecycle management
    """

    def __init__(self, parent_webview: "WebView"):
        """Initialize the child window manager.

        Args:
            parent_webview: The parent Gallery WebView for event emission.
        """
        self._parent = parent_webview
        self._children: Dict[str, ChildWindow] = {}
        self._server_socket: Optional[socket.socket] = None
        self._server_port: int = 0
        self._running = False
        self._accept_thread: Optional[threading.Thread] = None
        self._lock = threading.Lock()
        self._child_counter = 0
        self._event_handlers: Dict[str, List[Callable]] = {}

    def start(self) -> int:
        """Start the IPC server.

        Returns:
            The port number the server is listening on.
        """
        if self._running:
            return self._server_port

        # Create server socket
        self._server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self._server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self._server_socket.bind(("127.0.0.1", 0))  # Let OS choose port
        self._server_socket.listen(10)
        self._server_port = self._server_socket.getsockname()[1]

        # Start accept thread
        self._running = True
        self._accept_thread = threading.Thread(target=self._accept_loop, daemon=True)
        self._accept_thread.start()

        print(f"[ChildManager] IPC server started on port {self._server_port}", file=sys.stderr)
        return self._server_port

    def stop(self) -> None:
        """Stop the IPC server and close all children."""
        self._running = False

        # Close all children
        with self._lock:
            for child_id in list(self._children.keys()):
                self._close_child(child_id)

        # Close server socket
        if self._server_socket:
            try:
                self._server_socket.close()
            except Exception:
                pass
            self._server_socket = None

        print("[ChildManager] Stopped", file=sys.stderr)

    def launch_example(
        self,
        example_path: str,
        example_name: str,
        *,
        python_executable: Optional[str] = None,
        extra_env: Optional[Dict[str, str]] = None,
    ) -> str:
        """Launch an example as a child window.

        Args:
            example_path: Path to the example Python file.
            example_name: Display name for the example.
            python_executable: Python executable to use (default: sys.executable).
            extra_env: Extra environment variables to set.

        Returns:
            Child window ID.
        """
        # Ensure server is running
        if not self._running:
            self.start()

        # Generate child ID
        with self._lock:
            self._child_counter += 1
            child_id = f"child_{self._child_counter}_{int(time.time() * 1000)}"

        # Build environment
        env = os.environ.copy()
        env[ENV_PARENT_ID] = "gallery"
        env[ENV_PARENT_PORT] = str(self._server_port)
        env[ENV_CHILD_ID] = child_id
        env[ENV_EXAMPLE_NAME] = example_name

        if extra_env:
            env.update(extra_env)

        # Start process
        python = python_executable or sys.executable
        try:
            process = subprocess.Popen(
                [python, example_path],
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

            # Create child record
            child = ChildWindow(
                child_id=child_id,
                example_name=example_name,
                process=process,
            )

            with self._lock:
                self._children[child_id] = child

            # Start output monitoring threads
            threading.Thread(
                target=self._monitor_output,
                args=(child_id, process.stdout, "stdout"),
                daemon=True,
            ).start()
            threading.Thread(
                target=self._monitor_output,
                args=(child_id, process.stderr, "stderr"),
                daemon=True,
            ).start()

            # Emit event to frontend
            self._emit_to_parent(
                "child:launched",
                {
                    "child_id": child_id,
                    "example_name": example_name,
                },
            )

            print(f"[ChildManager] Launched {example_name} as {child_id}", file=sys.stderr)
            return child_id

        except Exception as e:
            print(f"[ChildManager] Failed to launch {example_name}: {e}", file=sys.stderr)
            raise

    def close_child(self, child_id: str) -> bool:
        """Close a child window.

        Args:
            child_id: Child window ID.

        Returns:
            True if closed successfully.
        """
        with self._lock:
            return self._close_child(child_id)

    def _close_child(self, child_id: str) -> bool:
        """Internal: Close a child window (must hold lock)."""
        child = self._children.pop(child_id, None)
        if not child:
            return False

        # Send close command
        if child.socket and child.connected:
            try:
                self._send_to_child(
                    child,
                    "parent:command",
                    {
                        "command": "close",
                    },
                )
            except Exception:
                pass

            try:
                child.socket.close()
            except Exception:
                pass

        # Terminate process
        if child.process:
            try:
                child.process.terminate()
                child.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                child.process.kill()
            except Exception:
                pass

        # Emit event
        self._emit_to_parent(
            "child:closed",
            {
                "child_id": child_id,
                "example_name": child.example_name,
            },
        )

        print(f"[ChildManager] Closed {child_id}", file=sys.stderr)
        return True

    def send_to_child(self, child_id: str, event: str, data: Any = None) -> bool:
        """Send an event to a child window.

        Args:
            child_id: Child window ID.
            event: Event name.
            data: Event data.

        Returns:
            True if sent successfully.
        """
        with self._lock:
            child = self._children.get(child_id)
            if not child or not child.connected:
                return False
            return self._send_to_child(child, event, data)

    def broadcast_to_children(self, event: str, data: Any = None) -> int:
        """Broadcast an event to all children.

        Args:
            event: Event name.
            data: Event data.

        Returns:
            Number of children that received the event.
        """
        count = 0
        with self._lock:
            for child in self._children.values():
                if child.connected:
                    if self._send_to_child(child, event, data):
                        count += 1
        return count

    def get_children(self) -> List[Dict]:
        """Get list of all child windows.

        Returns:
            List of child window info dicts.
        """
        with self._lock:
            return [
                {
                    "child_id": c.child_id,
                    "example_name": c.example_name,
                    "connected": c.connected,
                    "ready": c.ready,
                }
                for c in self._children.values()
            ]

    def on(self, event: str, handler: Callable) -> Callable[[], None]:
        """Register an event handler for child events.

        Args:
            event: Event name.
            handler: Callback function(child_id, data).

        Returns:
            Unsubscribe function.
        """
        with self._lock:
            if event not in self._event_handlers:
                self._event_handlers[event] = []
            self._event_handlers[event].append(handler)

        def unsubscribe():
            with self._lock:
                if event in self._event_handlers:
                    try:
                        self._event_handlers[event].remove(handler)
                    except ValueError:
                        pass

        return unsubscribe

    def _accept_loop(self) -> None:
        """Accept incoming connections from children."""
        while self._running and self._server_socket:
            try:
                self._server_socket.settimeout(1.0)
                try:
                    client_socket, addr = self._server_socket.accept()
                except socket.timeout:
                    continue

                # Start handler thread for this connection
                threading.Thread(
                    target=self._handle_child_connection,
                    args=(client_socket,),
                    daemon=True,
                ).start()

            except Exception as e:
                if self._running:
                    print(f"[ChildManager] Accept error: {e}", file=sys.stderr)

    def _handle_child_connection(self, client_socket: socket.socket) -> None:
        """Handle a child connection."""
        buffer = ""
        child_id = None

        while self._running:
            try:
                data = client_socket.recv(4096)
                if not data:
                    break

                buffer += data.decode("utf-8")

                # Process complete messages
                while "\n" in buffer:
                    line, buffer = buffer.split("\n", 1)
                    if line.strip():
                        msg = json.loads(line)
                        child_id = msg.get("child_id")

                        # Associate socket with child
                        if child_id:
                            with self._lock:
                                child = self._children.get(child_id)
                                if child:
                                    child.socket = client_socket
                                    child.connected = True

                        # Handle message
                        self._handle_child_message(child_id, msg)

            except Exception as e:
                print(f"[ChildManager] Connection error: {e}", file=sys.stderr)
                break

        # Cleanup
        if child_id:
            with self._lock:
                child = self._children.get(child_id)
                if child:
                    child.connected = False
                    child.socket = None

    def _handle_child_message(self, child_id: str, msg: Dict) -> None:
        """Handle a message from a child."""
        event = msg.get("event")
        data = msg.get("data")

        if not event:
            return

        # Handle special events
        if event == "child:ready":
            with self._lock:
                child = self._children.get(child_id)
                if child:
                    child.ready = True
            print(f"[ChildManager] Child {child_id} is ready", file=sys.stderr)

        elif event == "child:closing":
            print(f"[ChildManager] Child {child_id} is closing", file=sys.stderr)

        # Forward to parent WebView
        self._emit_to_parent(f"child:{child_id}:{event}", data)

        # Also emit generic event with child_id in data
        self._emit_to_parent(
            "child:event",
            {
                "child_id": child_id,
                "event": event,
                "data": data,
            },
        )

        # Call registered handlers
        with self._lock:
            handlers = self._event_handlers.get(event, [])[:]

        for handler in handlers:
            try:
                handler(child_id, data)
            except Exception as e:
                print(f"[ChildManager] Handler error: {e}", file=sys.stderr)

    def _send_to_child(self, child: ChildWindow, event: str, data: Any) -> bool:
        """Send a message to a child (must hold lock for child access)."""
        if not child.socket:
            return False

        try:
            message = json.dumps(
                {
                    "event": event,
                    "data": data,
                }
            )
            child.socket.sendall((message + "\n").encode("utf-8"))
            return True
        except Exception as e:
            print(f"[ChildManager] Send error to {child.child_id}: {e}", file=sys.stderr)
            return False

    def _emit_to_parent(self, event: str, data: Any) -> None:
        """Emit an event to the parent WebView."""
        try:
            emitter = self._parent.create_emitter()
            emitter.emit(event, data)
        except Exception as e:
            print(f"[ChildManager] Emit error: {e}", file=sys.stderr)

    def _monitor_output(self, child_id: str, stream, stream_name: str) -> None:
        """Monitor child process output."""
        try:
            for line in stream:
                if isinstance(line, bytes):
                    line = line.decode("utf-8", errors="replace")
                line = line.rstrip()
                if line:
                    print(f"[Child:{child_id}:{stream_name}] {line}", file=sys.stderr)
                    self._emit_to_parent(
                        "child:output",
                        {
                            "child_id": child_id,
                            "stream": stream_name,
                            "line": line,
                        },
                    )
        except Exception:
            pass


# Global instance
_manager: Optional[ChildWindowManager] = None


def get_manager() -> Optional[ChildWindowManager]:
    """Get the global child window manager."""
    return _manager


def init_manager(parent_webview: "WebView") -> ChildWindowManager:
    """Initialize the global child window manager.

    Args:
        parent_webview: Parent Gallery WebView.

    Returns:
        The initialized manager.
    """
    global _manager
    if _manager is None:
        _manager = ChildWindowManager(parent_webview)
        _manager.start()
    return _manager


def cleanup_manager() -> None:
    """Cleanup the global child window manager."""
    global _manager
    if _manager:
        _manager.stop()
        _manager = None


__all__ = [
    "ChildWindow",
    "ChildWindowManager",
    "get_manager",
    "init_manager",
    "cleanup_manager",
]
