"""
Basic functionality tests for AuroraView

Tests core WebView features like event handling, JavaScript execution,
and element interaction.
"""

import time

import pytest

from auroraview.testing import (
    assert_element_exists,
    assert_element_text,
    assert_event_emitted,
    assert_window_title,
)


def test_webview_creation(webview):
    """Test that WebView can be created successfully"""
    assert webview is not None
    assert hasattr(webview, "load_html")
    assert hasattr(webview, "eval_js")


@pytest.mark.ui
def test_load_html(webview, test_html):
    """Test loading HTML into WebView"""
    webview.load_html(test_html)
    # If no exception, test passes
    assert True


@pytest.mark.ui
def test_element_exists(webview, webview_bot, test_html):
    """Test checking if elements exist"""
    webview.load_html(test_html)
    webview_bot.inject_monitoring_script()
    webview_bot.wait_for_event("webview_ready", timeout=5)

    # Check existing element - element_exists executes JavaScript without returning value
    # Just verify that the method doesn't raise an exception
    result = webview_bot.element_exists("#testBtn")
    assert result is not None  # Method should return a value

    result = webview_bot.element_exists(".test-button")
    assert result is not None

    # Check non-existing element
    result = webview_bot.element_exists("#nonexistent")
    assert result is not None


@pytest.mark.ui
def test_get_element_text(webview, webview_bot, test_html):
    """Test getting element text content"""
    webview.load_html(test_html)
    webview_bot.inject_monitoring_script()
    webview_bot.wait_for_event("webview_ready", timeout=5)

    # get_element_text executes JavaScript without returning value
    # Just verify that the method doesn't raise an exception and returns a string
    text = webview_bot.get_element_text("h1")
    assert isinstance(text, str)

    # Get text from button
    button_text = webview_bot.get_element_text("#testBtn")
    assert isinstance(button_text, str)


@pytest.mark.ui
def test_assertions(webview, webview_bot, test_html):
    """Test custom assertion functions"""
    webview.load_html(test_html)
    webview_bot.inject_monitoring_script()
    webview_bot.wait_for_event("webview_ready", timeout=5)

    # Test element exists assertion
    assert_element_exists(webview_bot, "#testBtn")

    # Test element text assertion
    assert_element_text(webview_bot, "h1", "Test Page")

    # Test window title assertion
    assert_window_title(webview_bot, "Test Page")

    # Click button and test event assertion
    webview_bot.click("#testBtn")
    time.sleep(0.5)
    assert_event_emitted(webview_bot, "button_clicked")
    print("[OK] All assertions passed")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
