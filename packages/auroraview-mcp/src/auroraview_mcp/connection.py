"""CDP connection management module."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import Any

import httpx
import websockets
from websockets import ClientConnection


@dataclass
class Page:
    """Represents a CDP page/target."""

    id: str
    url: str
    title: str
    ws_url: str
    type: str = "page"

    def to_dict(self) -> dict[str, Any]:
        """Convert page to dictionary."""
        return {
            "id": self.id,
            "url": self.url,
            "title": self.title,
            "ws_url": self.ws_url,
            "type": self.type,
        }


@dataclass
class CDPConnection:
    """Chrome DevTools Protocol connection."""

    port: int
    ws_url: str
    _ws: ClientConnection | None = field(default=None, repr=False)
    _message_id: int = field(default=0, repr=False)

    async def connect(self) -> None:
        """Establish WebSocket connection."""
        self._ws = await websockets.connect(self.ws_url)

    async def disconnect(self) -> None:
        """Close WebSocket connection."""
        if self._ws:
            await self._ws.close()
            self._ws = None

    async def send(self, method: str, params: dict[str, Any] | None = None) -> dict[str, Any]:
        """Send CDP command and wait for response.

        Args:
            method: CDP method name.
            params: Method parameters.

        Returns:
            CDP response.
        """
        if not self._ws:
            raise RuntimeError("Not connected")

        self._message_id += 1
        message = {
            "id": self._message_id,
            "method": method,
            "params": params or {},
        }

        await self._ws.send(json.dumps(message))

        # Wait for response with matching id
        while True:
            response_text = await self._ws.recv()
            response = json.loads(response_text)
            if response.get("id") == self._message_id:
                if "error" in response:
                    raise CDPError(response["error"])
                return response.get("result", {})

    @property
    def is_connected(self) -> bool:
        """Check if connection is active."""
        return self._ws is not None and self._ws.state.name == "OPEN"


class CDPError(Exception):
    """CDP protocol error."""

    def __init__(self, error: dict[str, Any]) -> None:
        self.code = error.get("code", -1)
        self.message = error.get("message", "Unknown error")
        super().__init__(f"CDP Error {self.code}: {self.message}")


@dataclass
class PageConnection:
    """Connection to a specific CDP page."""

    page: Page
    _ws: ClientConnection | None = field(default=None, repr=False)
    _message_id: int = field(default=0, repr=False)

    async def connect(self) -> None:
        """Establish WebSocket connection to page."""
        self._ws = await websockets.connect(self.page.ws_url)

    async def disconnect(self) -> None:
        """Close WebSocket connection."""
        if self._ws:
            await self._ws.close()
            self._ws = None

    async def send(self, method: str, params: dict[str, Any] | None = None) -> dict[str, Any]:
        """Send CDP command to page.

        Args:
            method: CDP method name.
            params: Method parameters.

        Returns:
            CDP response.
        """
        if not self._ws:
            raise RuntimeError("Not connected to page")

        self._message_id += 1
        message = {
            "id": self._message_id,
            "method": method,
            "params": params or {},
        }

        await self._ws.send(json.dumps(message))

        while True:
            response_text = await self._ws.recv()
            response = json.loads(response_text)
            if response.get("id") == self._message_id:
                if "error" in response:
                    raise CDPError(response["error"])
                return response.get("result", {})

    async def evaluate(self, expression: str) -> Any:
        """Evaluate JavaScript expression in page context.

        Args:
            expression: JavaScript expression to evaluate.

        Returns:
            Evaluation result.
        """
        result = await self.send(
            "Runtime.evaluate",
            {
                "expression": expression,
                "returnByValue": True,
                "awaitPromise": True,
            },
        )

        if "exceptionDetails" in result:
            exception = result["exceptionDetails"]
            raise JavaScriptError(exception)

        return result.get("result", {}).get("value")

    @property
    def is_connected(self) -> bool:
        """Check if connection is active."""
        return self._ws is not None and self._ws.state.name == "OPEN"


class JavaScriptError(Exception):
    """JavaScript evaluation error."""

    def __init__(self, exception_details: dict[str, Any]) -> None:
        self.details = exception_details
        text = exception_details.get("text", "")
        exception = exception_details.get("exception", {})
        description = exception.get("description", text)
        super().__init__(f"JavaScript Error: {description}")


@dataclass
class ConnectionManager:
    """Manages CDP connections to AuroraView instances."""

    _connections: dict[int, CDPConnection] = field(default_factory=dict)
    _page_connections: dict[str, PageConnection] = field(default_factory=dict)
    _current_port: int | None = field(default=None)
    _current_page: Page | None = field(default=None)
    timeout: float = 5.0

    async def connect(self, port: int) -> CDPConnection:
        """Connect to an AuroraView instance.

        Args:
            port: CDP port number.

        Returns:
            CDP connection.
        """
        if port in self._connections and self._connections[port].is_connected:
            self._current_port = port
            return self._connections[port]

        # Get WebSocket URL
        async with httpx.AsyncClient() as client:
            resp = await client.get(
                f"http://127.0.0.1:{port}/json/version",
                timeout=self.timeout,
            )
            data = resp.json()
            ws_url = data.get("webSocketDebuggerUrl", "")

        if not ws_url:
            raise ConnectionError(f"No WebSocket URL found for port {port}")

        conn = CDPConnection(port=port, ws_url=ws_url)
        await conn.connect()

        self._connections[port] = conn
        self._current_port = port
        return conn

    async def disconnect(self, port: int | None = None) -> None:
        """Disconnect from an AuroraView instance.

        Args:
            port: Port to disconnect from. If None, disconnects current.
        """
        target_port = port or self._current_port
        if target_port is None:
            return

        if target_port in self._connections:
            await self._connections[target_port].disconnect()
            del self._connections[target_port]

        if self._current_port == target_port:
            self._current_port = None

    async def disconnect_all(self) -> None:
        """Disconnect from all instances."""
        for port in list(self._connections.keys()):
            await self.disconnect(port)

        for page_id in list(self._page_connections.keys()):
            await self._page_connections[page_id].disconnect()
            del self._page_connections[page_id]

        self._current_page = None

    async def get_pages(self, port: int | None = None) -> list[Page]:
        """Get all pages from an instance.

        Args:
            port: Port to query. If None, uses current.

        Returns:
            List of pages.
        """
        target_port = port or self._current_port
        if target_port is None:
            raise RuntimeError("Not connected to any instance")

        async with httpx.AsyncClient() as client:
            resp = await client.get(
                f"http://127.0.0.1:{target_port}/json/list",
                timeout=self.timeout,
            )
            pages_data = resp.json()

        pages = []
        for data in pages_data:
            # Filter out about:blank and non-page targets
            url = data.get("url", "")
            target_type = data.get("type", "page")
            if url != "about:blank" and target_type == "page":
                pages.append(
                    Page(
                        id=data["id"],
                        url=url,
                        title=data.get("title", ""),
                        ws_url=data.get("webSocketDebuggerUrl", ""),
                        type=target_type,
                    )
                )

        return pages

    async def select_page(
        self, page_id: str | None = None, url_pattern: str | None = None
    ) -> Page | None:
        """Select a page to operate on.

        Args:
            page_id: Page ID to select.
            url_pattern: URL pattern to match.

        Returns:
            Selected page or None.
        """
        pages = await self.get_pages()

        if page_id:
            for page in pages:
                if page.id == page_id:
                    self._current_page = page
                    return page
        elif url_pattern:
            import fnmatch

            for page in pages:
                if fnmatch.fnmatch(page.url, url_pattern):
                    self._current_page = page
                    return page
        elif pages:
            # Select first page if no criteria specified
            self._current_page = pages[0]
            return pages[0]

        return None

    async def get_page_connection(self, page: Page | None = None) -> PageConnection:
        """Get or create a connection to a page.

        Args:
            page: Page to connect to. If None, uses current.

        Returns:
            Page connection.
        """
        target_page = page or self._current_page
        if target_page is None:
            raise RuntimeError("No page selected")

        if target_page.id in self._page_connections:
            conn = self._page_connections[target_page.id]
            if conn.is_connected:
                return conn

        conn = PageConnection(page=target_page)
        await conn.connect()
        self._page_connections[target_page.id] = conn
        return conn

    @property
    def current_port(self) -> int | None:
        """Get current connected port."""
        return self._current_port

    @property
    def current_page(self) -> Page | None:
        """Get current selected page."""
        return self._current_page

    @property
    def is_connected(self) -> bool:
        """Check if connected to any instance."""
        return self._current_port is not None
