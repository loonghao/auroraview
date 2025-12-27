"""Child Window Support for AuroraView.

This module provides utilities for running examples either standalone or as
child windows of a parent application (like Gallery).

The mode is determined by environment variables:
- AURORAVIEW_PARENT_ID: If set, running as child window
- AURORAVIEW_PARENT_PORT: IPC port for parent communication

Usage in examples:
    from auroraview.child import ChildContext, is_child_mode

    # Check if running as child
    if is_child_mode():
        print("Running as child window")

    # Or use context manager for automatic setup
    with ChildContext() as ctx:
        webview = ctx.create_webview(
            title="My Example",
            width=800,
            height=600,
            html="<h1>Hello</h1>"
        )
        webview.show()

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import json
import os
import socket
import sys
import threading
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional

if TYPE_CHECKING:
    from auroraview import WebView

# Environment variable names
ENV_PARENT_ID = "AURORAVIEW_PARENT_ID"
ENV_PARENT_PORT = "AURORAVIEW_PARENT_PORT"
ENV_CHILD_ID = "AURORAVIEW_CHILD_ID"
ENV_EXAMPLE_NAME = "AURORAVIEW_EXAMPLE_NAME"


def is_child_mode() -> bool:
    """Check if running as a child window.

    Returns:
        True if running as child window of a parent application.
    """
    return ENV_PARENT_ID in os.environ


def get_parent_id() -> Optional[str]:
    """Get the parent window ID if running as child.

    Returns:
        Parent window ID or None if not in child mode.
    """
    return os.environ.get(ENV_PARENT_ID)


def get_child_id() -> Optional[str]:
    """Get this child window's ID.

    Returns:
        Child window ID or None if not in child mode.
    """
    return os.environ.get(ENV_CHILD_ID)


def get_example_name() -> Optional[str]:
    """Get the example name if running as child.

    Returns:
        Example name or None if not in child mode.
    """
    return os.environ.get(ENV_EXAMPLE_NAME)


@dataclass
class ChildInfo:
    """Information about the child window context."""

    is_child: bool = False
    parent_id: Optional[str] = None
    child_id: Optional[str] = None
    example_name: Optional[str] = None
    parent_port: Optional[int] = None

    @classmethod
    def from_env(cls) -> "ChildInfo":
        """Create ChildInfo from environment variables."""
        parent_id = os.environ.get(ENV_PARENT_ID)
        port_str = os.environ.get(ENV_PARENT_PORT)

        return cls(
            is_child=parent_id is not None,
            parent_id=parent_id,
            child_id=os.environ.get(ENV_CHILD_ID),
            example_name=os.environ.get(ENV_EXAMPLE_NAME),
            parent_port=int(port_str) if port_str else None,
        )


class ParentBridge:
    """Bridge for communicating with parent window.

    This class handles IPC communication with the parent application
    when running as a child window.
    """

    def __init__(self, port: int):
        """Initialize the parent bridge.

        Args:
            port: IPC port for parent communication.
        """
        self._port = port
        self._socket: Optional[socket.socket] = None
        self._connected = False
        self._handlers: Dict[str, List[Callable]] = {}
        self._lock = threading.Lock()
        self._recv_thread: Optional[threading.Thread] = None
        self._running = False

    def connect(self) -> bool:
        """Connect to the parent application.

        Returns:
            True if connection successful.
        """
        try:
            self._socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self._socket.connect(("127.0.0.1", self._port))
            self._connected = True

            # Start receive thread
            self._running = True
            self._recv_thread = threading.Thread(target=self._receive_loop, daemon=True)
            self._recv_thread.start()

            print(f"[ChildBridge] Connected to parent on port {self._port}", file=sys.stderr)
            return True
        except Exception as e:
            print(f"[ChildBridge] Failed to connect: {e}", file=sys.stderr)
            return False

    def disconnect(self) -> None:
        """Disconnect from parent."""
        self._running = False
        if self._socket:
            try:
                self._socket.close()
            except Exception:
                pass
        self._socket = None
        self._connected = False

    def send(self, event: str, data: Any = None) -> bool:
        """Send an event to the parent.

        Args:
            event: Event name.
            data: Event data (JSON-serializable).

        Returns:
            True if sent successfully.
        """
        if not self._connected or not self._socket:
            return False

        try:
            message = json.dumps(
                {
                    "type": "event",
                    "event": event,
                    "data": data,
                    "child_id": get_child_id(),
                }
            )
            # Send with newline delimiter
            self._socket.sendall((message + "\n").encode("utf-8"))
            return True
        except Exception as e:
            print(f"[ChildBridge] Send error: {e}", file=sys.stderr)
            return False

    def on(self, event: str, handler: Callable) -> Callable[[], None]:
        """Register an event handler.

        Args:
            event: Event name to listen for.
            handler: Callback function.

        Returns:
            Unsubscribe function.
        """
        with self._lock:
            if event not in self._handlers:
                self._handlers[event] = []
            self._handlers[event].append(handler)

        def unsubscribe():
            with self._lock:
                if event in self._handlers:
                    try:
                        self._handlers[event].remove(handler)
                    except ValueError:
                        pass

        return unsubscribe

    def _receive_loop(self) -> None:
        """Background thread for receiving messages from parent."""
        buffer = ""
        while self._running and self._socket:
            try:
                data = self._socket.recv(4096)
                if not data:
                    break

                buffer += data.decode("utf-8")

                # Process complete messages (newline-delimited)
                while "\n" in buffer:
                    line, buffer = buffer.split("\n", 1)
                    if line.strip():
                        self._handle_message(line)

            except Exception as e:
                if self._running:
                    print(f"[ChildBridge] Receive error: {e}", file=sys.stderr)
                break

    def _handle_message(self, message: str) -> None:
        """Handle a received message from parent."""
        try:
            msg = json.loads(message)
            event = msg.get("event")
            data = msg.get("data")

            if event:
                with self._lock:
                    handlers = self._handlers.get(event, [])[:]

                for handler in handlers:
                    try:
                        handler(data)
                    except Exception as e:
                        print(f"[ChildBridge] Handler error: {e}", file=sys.stderr)

        except json.JSONDecodeError as e:
            print(f"[ChildBridge] Invalid message: {e}", file=sys.stderr)


@dataclass
class ChildContext:
    """Context manager for child window mode.

    Provides automatic setup and teardown for child windows,
    with fallback to standalone mode when not running as child.

    Example:
        with ChildContext() as ctx:
            # ctx.is_child tells you the mode
            # ctx.bridge is available for parent communication (if child)

            webview = ctx.create_webview(...)
            webview.show()
    """

    info: ChildInfo = field(default_factory=ChildInfo.from_env)
    bridge: Optional[ParentBridge] = None
    _webview: Optional["WebView"] = None

    @property
    def is_child(self) -> bool:
        """Check if running as child window."""
        return self.info.is_child

    @property
    def parent_id(self) -> Optional[str]:
        """Get parent window ID."""
        return self.info.parent_id

    @property
    def child_id(self) -> Optional[str]:
        """Get child window ID."""
        return self.info.child_id

    @property
    def example_name(self) -> Optional[str]:
        """Get example name."""
        return self.info.example_name

    def __enter__(self) -> "ChildContext":
        """Enter the context, setting up parent bridge if needed."""
        if self.info.is_child and self.info.parent_port:
            self.bridge = ParentBridge(self.info.parent_port)
            self.bridge.connect()

            # Notify parent that child is ready
            self.bridge.send(
                "child:ready",
                {
                    "child_id": self.info.child_id,
                    "example_name": self.info.example_name,
                },
            )

        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """Exit the context, cleaning up resources."""
        if self.bridge:
            # Notify parent that child is closing
            self.bridge.send(
                "child:closing",
                {
                    "child_id": self.info.child_id,
                },
            )
            self.bridge.disconnect()

    def create_webview(
        self,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        **kwargs,
    ) -> "WebView":
        """Create a WebView with child-aware configuration.

        If running as child, the WebView title will be prefixed
        and certain behaviors may be adjusted.

        Args:
            title: Window title.
            width: Window width.
            height: Window height.
            **kwargs: Additional WebView arguments.

        Returns:
            Configured WebView instance.
        """
        from auroraview import WebView

        # Adjust title for child mode
        if self.is_child and self.example_name:
            title = f"{self.example_name} - {title}"

        # Create WebView
        webview = WebView(
            title=title,
            width=width,
            height=height,
            **kwargs,
        )

        self._webview = webview

        # Set up bridge integration if in child mode
        if self.bridge:
            self._setup_bridge_integration(webview)

        return webview

    def _setup_bridge_integration(self, webview: "WebView") -> None:
        """Set up bridge integration for parent communication."""
        if not self.bridge:
            return

        # Forward events from webview to parent
        @webview.on("*")
        def forward_event(event_name: str, data: Any):
            self.bridge.send(
                f"child:event:{event_name}",
                {
                    "child_id": self.info.child_id,
                    "event": event_name,
                    "data": data,
                },
            )

        # Listen for parent commands
        self.bridge.on("parent:command", self._handle_parent_command)

    def _handle_parent_command(self, data: Dict) -> None:
        """Handle commands from parent."""
        if not self._webview:
            return

        command = data.get("command")
        args = data.get("args", {})

        if command == "close":
            self._webview.close()
        elif command == "eval":
            js = args.get("js", "")
            self._webview.eval(js)
        elif command == "emit":
            event = args.get("event")
            payload = args.get("data")
            if event:
                self._webview.emit(event, payload)

    def emit_to_parent(self, event: str, data: Any = None) -> bool:
        """Emit an event to the parent window.

        Args:
            event: Event name.
            data: Event data.

        Returns:
            True if sent (or not in child mode).
        """
        if self.bridge:
            return self.bridge.send(event, data)
        return True  # Not in child mode, consider success

    def on_parent_event(self, event: str, handler: Callable) -> Callable[[], None]:
        """Listen for events from parent.

        Args:
            event: Event name.
            handler: Callback function.

        Returns:
            Unsubscribe function.
        """
        if self.bridge:
            return self.bridge.on(event, handler)

        # Return no-op unsubscribe if not in child mode
        return lambda: None


# Convenience function for simple usage
def run_example(
    create_webview_func: Callable[["ChildContext"], "WebView"],
    *,
    on_ready: Optional[Callable[["ChildContext"], None]] = None,
) -> None:
    """Run an example with automatic child mode handling.

    This is a convenience function that handles both standalone and
    child window modes automatically.

    Args:
        create_webview_func: Function that creates and returns a WebView.
            Receives ChildContext as argument.
        on_ready: Optional callback when ready.

    Example:
        def create_my_webview(ctx):
            webview = ctx.create_webview(
                title="My Example",
                html="<h1>Hello</h1>"
            )
            return webview

        if __name__ == "__main__":
            run_example(create_my_webview)
    """
    with ChildContext() as ctx:
        if ctx.is_child:
            print(f"[Example] Running as child window (parent={ctx.parent_id})", file=sys.stderr)
        else:
            print("[Example] Running standalone", file=sys.stderr)

        webview = create_webview_func(ctx)

        if on_ready:
            on_ready(ctx)

        webview.show()


__all__ = [
    "is_child_mode",
    "get_parent_id",
    "get_child_id",
    "get_example_name",
    "ChildInfo",
    "ChildContext",
    "ParentBridge",
    "run_example",
    "ENV_PARENT_ID",
    "ENV_PARENT_PORT",
    "ENV_CHILD_ID",
    "ENV_EXAMPLE_NAME",
]
