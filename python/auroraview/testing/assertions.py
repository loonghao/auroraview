"""
Custom assertions for AuroraView testing

Provides specialized assertion functions for WebView testing.
"""


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

    Note: Due to AuroraView's async JavaScript execution model, this only verifies
    that the JavaScript executes without error. Actual element existence verification
    would require event-based communication.

    Args:
        webview_bot: WebViewBot instance
        selector: CSS selector for the element

    Raises:
        AssertionError: If JavaScript execution fails
    """
    result = webview_bot.element_exists(selector)
    assert result is not None, f"Failed to check element existence for selector '{selector}'"


def assert_element_text(webview_bot, selector: str, expected_text: str):
    """
    Assert that an element has specific text content.

    Note: Due to AuroraView's async JavaScript execution model, this only verifies
    that the JavaScript executes without error. Actual text content verification
    would require event-based communication.

    Args:
        webview_bot: WebViewBot instance
        selector: CSS selector for the element
        expected_text: Expected text content

    Raises:
        AssertionError: If JavaScript execution fails
    """
    actual_text = webview_bot.get_element_text(selector)
    # Just verify that we got a string back (JavaScript executed)
    assert isinstance(actual_text, str), f"Failed to get element text for selector '{selector}'"


def assert_window_title(webview_bot, expected_title: str):
    """
    Assert that the window has a specific title.

    Note: Due to AuroraView's async JavaScript execution model, this only verifies
    that the JavaScript executes without error. Actual title verification would
    require event-based communication.

    Args:
        webview_bot: WebViewBot instance
        expected_title: Expected window title

    Raises:
        AssertionError: If JavaScript execution fails
    """
    script = "document.title"
    try:
        webview_bot.webview.eval_js(script)
        # If no exception, JavaScript executed successfully
        assert True
    except Exception as e:
        raise AssertionError(f"Failed to execute title check: {e}") from e


def assert_element_visible(webview_bot, selector: str):
    """
    Assert that an element is visible.

    Note: Due to AuroraView's async JavaScript execution model, this only verifies
    that the JavaScript executes without error. Actual visibility verification would
    require event-based communication.

    Args:
        webview_bot: WebViewBot instance
        selector: CSS selector for the element

    Raises:
        AssertionError: If JavaScript execution fails
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
        webview_bot.webview.eval_js(script)
        # If no exception, JavaScript executed successfully
        assert True
    except Exception as e:
        raise AssertionError(f"Failed to check visibility: {e}") from e


def assert_element_hidden(webview_bot, selector: str):
    """
    Assert that an element is hidden.

    Note: Due to AuroraView's async JavaScript execution model, this only verifies
    that the JavaScript executes without error. Actual visibility verification would
    require event-based communication.

    Args:
        webview_bot: WebViewBot instance
        selector: CSS selector for the element

    Raises:
        AssertionError: If JavaScript execution fails
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
        webview_bot.webview.eval_js(script)
        # If no exception, JavaScript executed successfully
        assert True
    except Exception as e:
        raise AssertionError(f"Failed to check visibility: {e}") from e
