"""
Tests for window dragging functionality

Tests the window dragging and movement features.
"""

import time

import pytest


@pytest.mark.ui
def test_window_dragging_setup(webview, webview_bot):
    """Test setup for window dragging tests"""
    html = """
    <html>
        <head>
            <style>
                .title-bar {
                    background: #333;
                    color: white;
                    padding: 10px;
                    cursor: move;
                }
            </style>
        </head>
        <body>
            <div class="title-bar">Draggable Window</div>
            <div id="content">Content</div>
        </body>
    </html>
    """

    webview.load_html(html)
    webview_bot.inject_monitoring_script()
    webview_bot.wait_for_event("webview_ready", timeout=5)

    # Verify elements exist
    assert webview_bot.element_exists(".title-bar")
    assert webview_bot.element_exists("#content")


@pytest.mark.ui
def test_window_drag_event(webview, webview_bot):
    """Test window drag event emission"""
    html = """
    <html>
        <head>
            <style>
                .title-bar {
                    background: #333;
                    color: white;
                    padding: 10px;
                    cursor: move;
                }
            </style>
        </head>
        <body>
            <div class="title-bar" id="titleBar">Draggable Window</div>
            <script>
                document.getElementById('titleBar').addEventListener('mousedown', function() {
                    window.dispatchEvent(new CustomEvent('move_window'));
                });
            </script>
        </body>
    </html>
    """

    webview.load_html(html)
    webview_bot.inject_monitoring_script()
    webview_bot.wait_for_event("webview_ready", timeout=5)

    # Simulate drag
    webview_bot.drag(".title-bar", offset=(100, 50))
    time.sleep(0.5)

    # Check if event was emitted
    webview_bot.assert_event_emitted("move_window")
    print("[OK] Window drag event test passed")
