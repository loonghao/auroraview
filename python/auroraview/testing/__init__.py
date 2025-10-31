"""
AuroraView Testing Framework

A pytest-qt inspired testing framework for AuroraView WebView applications.

This module provides fixtures, utilities, and helpers for testing WebView-based
applications with a focus on UI automation and event verification.

Example:
    ```python
    from auroraview.testing import WebViewBot, webview, webview_bot
    
    def test_window_dragging(webview, webview_bot):
        webview.load_html(test_html)
        webview_bot.wait_for_event('webview_ready', timeout=5)
        webview_bot.drag('.title-bar', offset=(100, 50))
        webview_bot.assert_event_emitted('move_window')
    ```
"""

from .webview_bot import WebViewBot, EventRecord
from .fixtures import (
    webview,
    webview_bot,
    webview_with_html,
    headless_webview,
    test_html,
    draggable_window_html,
)
from .assertions import (
    assert_event_emitted,
    assert_element_exists,
    assert_element_text,
    assert_window_title,
)

__all__ = [
    'WebViewBot',
    'EventRecord',
    'webview',
    'webview_bot',
    'webview_with_html',
    'headless_webview',
    'test_html',
    'draggable_window_html',
    'assert_event_emitted',
    'assert_element_exists',
    'assert_element_text',
    'assert_window_title',
]

