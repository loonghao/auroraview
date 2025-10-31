"""
Custom assertions for AuroraView testing

Provides specialized assertion functions for WebView testing.
"""

from typing import Optional


def assert_event_emitted(webview_bot, event_name: str):
    """
    Assert that a specific event was emitted.
    
    Args:
        webview_bot: WebViewBot instance
        event_name: Name of the event to check
        
    Raises:
        AssertionError: If event was not emitted
    """
    webview_bot.assert_event_emitted(event_name)


def assert_element_exists(webview_bot, selector: str):
    """
    Assert that an element exists in the DOM.
    
    Args:
        webview_bot: WebViewBot instance
        selector: CSS selector for the element
        
    Raises:
        AssertionError: If element does not exist
    """
    assert webview_bot.element_exists(selector), \
        f"Element with selector '{selector}' does not exist"


def assert_element_text(webview_bot, selector: str, expected_text: str):
    """
    Assert that an element has specific text content.
    
    Args:
        webview_bot: WebViewBot instance
        selector: CSS selector for the element
        expected_text: Expected text content
        
    Raises:
        AssertionError: If text does not match
    """
    actual_text = webview_bot.get_element_text(selector)
    assert actual_text == expected_text, \
        f"Element text mismatch. Expected: '{expected_text}', Got: '{actual_text}'"


def assert_window_title(webview_bot, expected_title: str):
    """
    Assert that the window has a specific title.
    
    Args:
        webview_bot: WebViewBot instance
        expected_title: Expected window title
        
    Raises:
        AssertionError: If title does not match
    """
    script = "document.title"
    actual_title = webview_bot.webview.eval_js(script)
    assert actual_title == expected_title, \
        f"Window title mismatch. Expected: '{expected_title}', Got: '{actual_title}'"


def assert_element_visible(webview_bot, selector: str):
    """
    Assert that an element is visible.
    
    Args:
        webview_bot: WebViewBot instance
        selector: CSS selector for the element
        
    Raises:
        AssertionError: If element is not visible
    """
    script = f"""
    (function() {{
        const element = document.querySelector('{selector}');
        if (!element) return false;
        const style = window.getComputedStyle(element);
        return style.display !== 'none' && style.visibility !== 'hidden';
    }})()
    """
    try:
        is_visible = webview_bot.webview.eval_js(script)
        assert is_visible, f"Element with selector '{selector}' is not visible"
    except Exception as e:
        raise AssertionError(f"Failed to check visibility: {e}")


def assert_element_hidden(webview_bot, selector: str):
    """
    Assert that an element is hidden.
    
    Args:
        webview_bot: WebViewBot instance
        selector: CSS selector for the element
        
    Raises:
        AssertionError: If element is visible
    """
    script = f"""
    (function() {{
        const element = document.querySelector('{selector}');
        if (!element) return true;
        const style = window.getComputedStyle(element);
        return style.display === 'none' || style.visibility === 'hidden';
    }})()
    """
    try:
        is_hidden = webview_bot.webview.eval_js(script)
        assert is_hidden, f"Element with selector '{selector}' is not hidden"
    except Exception as e:
        raise AssertionError(f"Failed to check visibility: {e}")

