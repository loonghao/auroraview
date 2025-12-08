"""Tests for the Cookie management module."""

from datetime import datetime, timedelta

import pytest

from auroraview.core.cookies import Cookie


class TestCookie:
    """Tests for Cookie class."""

    def test_basic_cookie(self):
        """Test creating a basic cookie."""
        cookie = Cookie(name="test", value="value123")
        assert cookie.name == "test"
        assert cookie.value == "value123"
        assert cookie.path == "/"
        assert cookie.secure is False
        assert cookie.http_only is False

    def test_full_cookie(self):
        """Test creating a cookie with all attributes."""
        expires = datetime(2025, 12, 31, 23, 59, 59)
        cookie = Cookie(
            name="session",
            value="abc123",
            domain="example.com",
            path="/app",
            expires=expires,
            secure=True,
            http_only=True,
            same_site="Strict",
        )
        assert cookie.name == "session"
        assert cookie.value == "abc123"
        assert cookie.domain == "example.com"
        assert cookie.path == "/app"
        assert cookie.expires == expires
        assert cookie.secure is True
        assert cookie.http_only is True
        assert cookie.same_site == "Strict"

    def test_empty_name_raises(self):
        """Test that empty name raises ValueError."""
        with pytest.raises(ValueError, match="Cookie name cannot be empty"):
            Cookie(name="", value="test")

    def test_invalid_same_site_raises(self):
        """Test that invalid SameSite value raises ValueError."""
        with pytest.raises(ValueError, match="Invalid SameSite value"):
            Cookie(name="test", value="value", same_site="Invalid")

    def test_valid_same_site_values(self):
        """Test valid SameSite values."""
        for same_site in ("Strict", "Lax", "None"):
            cookie = Cookie(name="test", value="value", same_site=same_site)
            assert cookie.same_site == same_site


class TestCookieToDict:
    """Tests for Cookie.to_dict method."""

    def test_basic_to_dict(self):
        """Test converting basic cookie to dict."""
        cookie = Cookie(name="test", value="value")
        result = cookie.to_dict()
        assert result["name"] == "test"
        assert result["value"] == "value"
        assert result["path"] == "/"
        assert "domain" not in result  # None values excluded

    def test_full_to_dict(self):
        """Test converting full cookie to dict."""
        expires = datetime(2025, 12, 31, 23, 59, 59)
        cookie = Cookie(
            name="test",
            value="value",
            domain="example.com",
            expires=expires,
            same_site="Lax",
        )
        result = cookie.to_dict()
        assert result["domain"] == "example.com"
        assert result["expires"] == expires.isoformat()
        assert result["same_site"] == "Lax"


class TestCookieFromDict:
    """Tests for Cookie.from_dict method."""

    def test_basic_from_dict(self):
        """Test creating cookie from basic dict."""
        data = {"name": "test", "value": "value"}
        cookie = Cookie.from_dict(data)
        assert cookie.name == "test"
        assert cookie.value == "value"
        assert cookie.path == "/"

    def test_full_from_dict(self):
        """Test creating cookie from full dict."""
        expires = datetime(2025, 12, 31, 23, 59, 59)
        data = {
            "name": "test",
            "value": "value",
            "domain": "example.com",
            "expires": expires.isoformat(),
            "secure": True,
        }
        cookie = Cookie.from_dict(data)
        assert cookie.domain == "example.com"
        assert cookie.expires == expires
        assert cookie.secure is True


class TestCookieExpiration:
    """Tests for cookie expiration methods."""

    def test_session_cookie(self):
        """Test session cookie detection."""
        cookie = Cookie(name="test", value="value")
        assert cookie.is_session_cookie() is True
        assert cookie.is_expired() is False

    def test_future_expiration(self):
        """Test cookie with future expiration."""
        future = datetime.now() + timedelta(days=1)
        cookie = Cookie(name="test", value="value", expires=future)
        assert cookie.is_session_cookie() is False
        assert cookie.is_expired() is False

    def test_past_expiration(self):
        """Test cookie with past expiration."""
        past = datetime.now() - timedelta(days=1)
        cookie = Cookie(name="test", value="value", expires=past)
        assert cookie.is_expired() is True


class TestCookieHeader:
    """Tests for Set-Cookie header generation."""

    def test_basic_header(self):
        """Test basic Set-Cookie header."""
        cookie = Cookie(name="test", value="value")
        header = cookie.to_set_cookie_header()
        assert header.startswith("test=value")
        assert "Path=/" in header

    def test_secure_header(self):
        """Test Set-Cookie header with Secure flag."""
        cookie = Cookie(name="test", value="value", secure=True)
        header = cookie.to_set_cookie_header()
        assert "Secure" in header

    def test_http_only_header(self):
        """Test Set-Cookie header with HttpOnly flag."""
        cookie = Cookie(name="test", value="value", http_only=True)
        header = cookie.to_set_cookie_header()
        assert "HttpOnly" in header
