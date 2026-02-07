"""Cookie management for AuroraView.

This module provides a unified interface for managing cookies in WebView,
inspired by Qt WebView's cookie management pattern.

Example:
    >>> from auroraview.cookies import Cookie
    >>> cookie = Cookie(
    ...     name="session_id",
    ...     value="abc123",
    ...     domain="example.com",
    ...     path="/",
    ...     secure=True,
    ...     http_only=True,
    ... )
    >>> print(cookie.to_dict())
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from typing import Optional

from ..features.serializable import Serializable


@dataclass
class Cookie(Serializable):
    """Represents an HTTP cookie.

    Attributes:
        name: Cookie name (required)
        value: Cookie value (required)
        domain: Cookie domain (optional, defaults to current domain)
        path: Cookie path (default: "/")
        expires: Expiration datetime (optional, None = session cookie)
        secure: Only send over HTTPS (default: False)
        http_only: Not accessible via JavaScript (default: False)
        same_site: SameSite attribute ("Strict", "Lax", "None", or None)
    """

    _exclude_none_fields = frozenset({"domain", "expires", "same_site"})

    name: str
    value: str
    domain: Optional[str] = None
    path: str = "/"
    expires: Optional[datetime] = None
    secure: bool = False
    http_only: bool = False
    same_site: Optional[str] = None

    def __post_init__(self):
        """Validate cookie attributes."""
        if not self.name:
            raise ValueError("Cookie name cannot be empty")
        if self.same_site is not None and self.same_site not in ("Strict", "Lax", "None"):
            raise ValueError(f"Invalid SameSite value: {self.same_site}")

    def to_set_cookie_header(self) -> str:
        """Generate Set-Cookie header value.

        Returns:
            Set-Cookie header string
        """
        parts = [f"{self.name}={self.value}"]

        if self.domain:
            parts.append(f"Domain={self.domain}")
        parts.append(f"Path={self.path}")

        if self.expires:
            # Format as HTTP date
            parts.append(f"Expires={self.expires.strftime('%a, %d %b %Y %H:%M:%S GMT')}")

        if self.secure:
            parts.append("Secure")
        if self.http_only:
            parts.append("HttpOnly")
        if self.same_site:
            parts.append(f"SameSite={self.same_site}")

        return "; ".join(parts)

    def is_expired(self) -> bool:
        """Check if cookie is expired.

        Returns:
            True if cookie is expired, False otherwise
        """
        if self.expires is None:
            return False  # Session cookie, never expires
        return datetime.now() > self.expires

    def is_session_cookie(self) -> bool:
        """Check if this is a session cookie.

        Returns:
            True if no expiration is set
        """
        return self.expires is None


__all__ = [
    "Cookie",
]
