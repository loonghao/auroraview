"""
Network interception classes for AuroraTest.

Provides request/response interception and mocking capabilities.
"""

from __future__ import annotations

import json
import logging
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any, Dict, Optional, Union

if TYPE_CHECKING:
    from .page import Page

logger = logging.getLogger(__name__)


@dataclass
class Request:
    """
    Represents an intercepted network request.

    Example:
        ```python
        async def handle_route(route):
            request = route.request
            print(f"URL: {request.url}")
            print(f"Method: {request.method}")
            print(f"Headers: {request.headers}")
        ```
    """

    url: str
    """Request URL."""

    method: str = "GET"
    """HTTP method."""

    headers: Dict[str, str] = field(default_factory=dict)
    """Request headers."""

    post_data: Optional[str] = None
    """POST data if any."""

    resource_type: str = "document"
    """Resource type: document, stylesheet, script, image, etc."""

    is_navigation_request: bool = False
    """Whether this is a navigation request."""

    frame: Optional[Any] = None
    """Frame that initiated the request."""

    def post_data_json(self) -> Optional[Any]:
        """Parse POST data as JSON."""
        if self.post_data:
            try:
                return json.loads(self.post_data)
            except json.JSONDecodeError:
                return None
        return None

    def all_headers(self) -> Dict[str, str]:
        """Get all headers."""
        return self.headers.copy()


@dataclass
class Response:
    """
    Represents a network response.

    Example:
        ```python
        response = await page.goto("https://example.com")
        print(f"Status: {response.status}")
        print(f"Headers: {response.headers}")
        body = await response.body()
        ```
    """

    url: str
    """Response URL."""

    status: int = 200
    """HTTP status code."""

    status_text: str = "OK"
    """HTTP status text."""

    headers: Dict[str, str] = field(default_factory=dict)
    """Response headers."""

    _body: bytes = field(default=b"", repr=False)
    """Response body."""

    request: Optional[Request] = None
    """Associated request."""

    @property
    def ok(self) -> bool:
        """Check if response was successful (2xx status)."""
        return 200 <= self.status < 300

    async def body(self) -> bytes:
        """Get response body as bytes."""
        return self._body

    async def text(self) -> str:
        """Get response body as text."""
        return self._body.decode("utf-8")

    async def json(self) -> Any:
        """Parse response body as JSON."""
        return json.loads(self._body)

    def all_headers(self) -> Dict[str, str]:
        """Get all headers."""
        return self.headers.copy()


class Route:
    """
    Route for intercepting and handling network requests.

    Example:
        ```python
        async def mock_api(route):
            if "api/users" in route.request.url:
                await route.fulfill(
                    status=200,
                    content_type="application/json",
                    body='[{"id": 1, "name": "Test"}]'
                )
            else:
                await route.continue_()

        await page.route("**/api/**", mock_api)
        ```
    """

    def __init__(
        self,
        request: Request,
        page: "Page",
    ):
        """Initialize route."""
        self._request = request
        self._page = page
        self._handled = False

    @property
    def request(self) -> Request:
        """Get the intercepted request."""
        return self._request

    async def abort(self, error_code: str = "failed"):
        """
        Abort the request.

        Args:
            error_code: Error code:
                - "aborted": Request was aborted
                - "accessdenied": Access denied
                - "addressunreachable": Address unreachable
                - "blockedbyclient": Blocked by client
                - "blockedbyresponse": Blocked by response
                - "connectionaborted": Connection aborted
                - "connectionclosed": Connection closed
                - "connectionfailed": Connection failed
                - "connectionrefused": Connection refused
                - "connectionreset": Connection reset
                - "internetdisconnected": Internet disconnected
                - "namenotresolved": Name not resolved
                - "timedout": Timed out
                - "failed": Generic failure
        """
        if self._handled:
            raise RuntimeError("Route already handled")

        self._handled = True
        logger.info(f"Aborting request: {self._request.url} ({error_code})")
        # TODO: Implement actual abort via WebView2

    async def continue_(
        self,
        url: Optional[str] = None,
        method: Optional[str] = None,
        headers: Optional[Dict[str, str]] = None,
        post_data: Optional[Union[str, bytes]] = None,
    ):
        """
        Continue the request with optional modifications.

        Args:
            url: Override URL.
            method: Override method.
            headers: Override headers.
            post_data: Override POST data.

        Example:
            ```python
            async def add_auth(route):
                headers = route.request.headers.copy()
                headers["Authorization"] = "Bearer token"
                await route.continue_(headers=headers)
            ```
        """
        if self._handled:
            raise RuntimeError("Route already handled")

        self._handled = True
        logger.info(f"Continuing request: {self._request.url}")
        # TODO: Implement actual continue via WebView2

    async def fulfill(
        self,
        status: int = 200,
        headers: Optional[Dict[str, str]] = None,
        content_type: Optional[str] = None,
        body: Optional[Union[str, bytes]] = None,
        json: Optional[Any] = None,
        path: Optional[str] = None,
        response: Optional[Response] = None,
    ):
        """
        Fulfill the request with a custom response.

        Args:
            status: HTTP status code.
            headers: Response headers.
            content_type: Content-Type header.
            body: Response body (string or bytes).
            json: Response body as JSON (will be serialized).
            path: Path to file to use as response body.
            response: Use another Response object.

        Example:
            ```python
            # Mock JSON API
            await route.fulfill(
                status=200,
                content_type="application/json",
                body='{"success": true}'
            )

            # Mock with JSON helper
            await route.fulfill(
                json={"users": [{"id": 1}]}
            )

            # Mock from file
            await route.fulfill(path="fixtures/response.json")
            ```
        """
        if self._handled:
            raise RuntimeError("Route already handled")

        self._handled = True

        # Build response headers
        response_headers = headers or {}
        if content_type:
            response_headers["Content-Type"] = content_type

        # Build response body
        if json is not None:
            import json as json_module

            json_module.dumps(json)
            if "Content-Type" not in response_headers:
                response_headers["Content-Type"] = "application/json"
        elif path:
            with open(path, "rb") as f:
                f.read()
        elif response:
            await response.body()
            status = response.status
            response_headers = response.headers

        logger.info(f"Fulfilling request: {self._request.url} with status {status}")
        # TODO: Implement actual fulfill via WebView2

    async def fallback(
        self,
        url: Optional[str] = None,
        method: Optional[str] = None,
        headers: Optional[Dict[str, str]] = None,
        post_data: Optional[Union[str, bytes]] = None,
    ):
        """
        Fall through to the next route handler.

        Similar to continue_() but allows other handlers to process.
        """
        if self._handled:
            raise RuntimeError("Route already handled")

        # Don't mark as handled - allow fallback to next handler
        logger.info(f"Falling back request: {self._request.url}")
        # TODO: Implement fallback mechanism


class APIRequestContext:
    """
    Context for making API requests independent of browser.

    Example:
        ```python
        api = await browser.new_api_context(base_url="https://api.example.com")
        response = await api.get("/users")
        data = await response.json()
        ```
    """

    def __init__(
        self,
        base_url: Optional[str] = None,
        extra_http_headers: Optional[Dict[str, str]] = None,
        timeout: float = 30000,
    ):
        """Initialize API context."""
        self._base_url = base_url or ""
        self._headers = extra_http_headers or {}
        self._timeout = timeout

    def _build_url(self, url: str) -> str:
        """Build full URL from relative path."""
        if url.startswith("http://") or url.startswith("https://"):
            return url
        return f"{self._base_url.rstrip('/')}/{url.lstrip('/')}"

    async def get(
        self,
        url: str,
        params: Optional[Dict[str, str]] = None,
        headers: Optional[Dict[str, str]] = None,
        timeout: Optional[float] = None,
    ) -> Response:
        """Make GET request."""
        # TODO: Implement with httpx or aiohttp
        return Response(url=self._build_url(url))

    async def post(
        self,
        url: str,
        data: Optional[Union[str, bytes, Dict]] = None,
        json: Optional[Any] = None,
        headers: Optional[Dict[str, str]] = None,
        timeout: Optional[float] = None,
    ) -> Response:
        """Make POST request."""
        # TODO: Implement
        return Response(url=self._build_url(url))

    async def put(
        self,
        url: str,
        data: Optional[Union[str, bytes, Dict]] = None,
        json: Optional[Any] = None,
        headers: Optional[Dict[str, str]] = None,
        timeout: Optional[float] = None,
    ) -> Response:
        """Make PUT request."""
        # TODO: Implement
        return Response(url=self._build_url(url))

    async def patch(
        self,
        url: str,
        data: Optional[Union[str, bytes, Dict]] = None,
        json: Optional[Any] = None,
        headers: Optional[Dict[str, str]] = None,
        timeout: Optional[float] = None,
    ) -> Response:
        """Make PATCH request."""
        # TODO: Implement
        return Response(url=self._build_url(url))

    async def delete(
        self,
        url: str,
        headers: Optional[Dict[str, str]] = None,
        timeout: Optional[float] = None,
    ) -> Response:
        """Make DELETE request."""
        # TODO: Implement
        return Response(url=self._build_url(url))

    async def head(
        self,
        url: str,
        headers: Optional[Dict[str, str]] = None,
        timeout: Optional[float] = None,
    ) -> Response:
        """Make HEAD request."""
        # TODO: Implement
        return Response(url=self._build_url(url))

    async def dispose(self):
        """Dispose of the API context."""
        pass
