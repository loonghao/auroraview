"""
Pytest fixtures for AuroraView testing

Provides fixtures for creating and managing WebView instances in tests.
"""

import pytest

from auroraview import WebView

from .webview_bot import WebViewBot


@pytest.fixture
def webview():
    """Create a basic WebView instance for testing"""
    view = WebView(title="Test WebView", width=800, height=600)
    yield view
    # Cleanup
    try:
        view.close()
    except:  # noqa: E722
        pass


@pytest.fixture
def webview_bot(webview):
    """Create a WebViewBot instance for automation"""
    return WebViewBot(webview)


@pytest.fixture
def test_html():
    """Provide sample HTML for testing"""
    return """
    <html>
        <head>
            <title>Test Page</title>
            <style>
                body { font-family: Arial, sans-serif; }
                button { padding: 10px 20px; }
            </style>
        </head>
        <body>
            <h1>Test Page</h1>
            <button id="testBtn" class="test-button">Test Button</button>
            <div id="output"></div>
            <script>
                document.getElementById('testBtn').addEventListener('click', function() {
                    window.dispatchEvent(new CustomEvent('button_clicked'));
                    document.getElementById('output').textContent = 'Button clicked!';
                });
            </script>
        </body>
    </html>
    """


@pytest.fixture
def webview_with_html(webview, test_html):
    """Create a WebView pre-loaded with test HTML"""
    webview.load_html(test_html)
    return webview


@pytest.fixture
def headless_webview():
    """Create a headless WebView instance"""
    view = WebView(title="Headless WebView", width=800, height=600, decorations=False)
    yield view
    try:
        view.close()
    except:  # noqa: E722
        pass


@pytest.fixture
def draggable_window_html():
    """Provide HTML for testing window dragging"""
    return """
    <html>
        <head>
            <style>
                .title-bar {
                    background: #333;
                    color: white;
                    padding: 10px;
                    cursor: move;
                    user-select: none;
                }
                body {
                    margin: 0;
                    font-family: Arial, sans-serif;
                }
                #content {
                    padding: 20px;
                }
            </style>
        </head>
        <body>
            <div class="title-bar" id="titleBar">Draggable Window</div>
            <div id="content">
                <p>This window can be dragged by the title bar.</p>
            </div>
            <script>
                const titleBar = document.getElementById('titleBar');
                titleBar.addEventListener('mousedown', function() {
                    window.dispatchEvent(new CustomEvent('move_window'));
                });
            </script>
        </body>
    </html>
    """
