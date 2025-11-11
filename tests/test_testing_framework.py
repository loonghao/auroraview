"""Tests for our internal testing helpers under auroraview.testing

These tests validate the helper APIs themselves and do not require a real GUI.
"""

from typing import Any, Callable, Dict

import pytest

from auroraview.testing import assertions
from auroraview.testing.webview_bot import WebViewBot


class DummyWebView:
    """Minimal WebView stub to satisfy WebViewBot without opening a real window."""

    def __init__(self) -> None:
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Any]] = {}
        self._scripts = []

    def on(self, event_name: str):  # decorator-style API
        def decorator(func: Callable[[Dict[str, Any]], Any]):
            self._handlers[event_name] = func
            return func

        return decorator

    def eval_js(self, script: str) -> None:
        # Record script to simulate successful execution
        self._scripts.append(script)


@pytest.fixture
def dummy_bot() -> WebViewBot:
    return WebViewBot(DummyWebView())


def test_webview_bot_basic_apis(dummy_bot: WebViewBot) -> None:
    # Should not raise
    dummy_bot.inject_monitoring_script()
    dummy_bot.click("#btn")
    dummy_bot.type("#input", "hello")
    assert dummy_bot.element_exists("#anything") is True
    assert isinstance(dummy_bot.get_element_text("#anything"), str)


def test_assertions_helpers_do_not_raise(dummy_bot: WebViewBot) -> None:
    # All helpers should execute without raising exceptions when JS runs successfully
    assertions.assert_element_exists(dummy_bot, "#root")
    assertions.assert_element_text(dummy_bot, "#root", "Test Page")
    assertions.assert_element_visible(dummy_bot, "#root")
    assertions.assert_element_hidden(dummy_bot, "#root")
    assertions.assert_window_title(dummy_bot, "Title")

    # Event assertion (relies on JS path executing without errors)
    assertions.assert_event_emitted(dummy_bot, "webview_ready")


def test_webview_bot_more_apis(dummy_bot: WebViewBot) -> None:
    # Exercise additional helper methods
    dummy_bot.inject_monitoring_script()
    assert dummy_bot.wait_for_event("webview_ready", timeout=0.2) is True
    dummy_bot.drag("#draggable", (10, 5))
