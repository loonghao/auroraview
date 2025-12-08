"""
Pytest fixtures for AuroraView testing.

Provides fixtures for headless WebView testing using the new unified API.
"""

import pytest

from .headless_webview import HeadlessWebView


@pytest.fixture
def headless_webview():
    """Create a headless WebView instance for testing.

    Uses Playwright by default for fast, reliable testing.

    Example:
        ```python
        def test_example(headless_webview):
            headless_webview.load_html("<h1>Hello</h1>")
            assert headless_webview.text("h1") == "Hello"
        ```
    """
    with HeadlessWebView.playwright() as webview:
        yield webview


@pytest.fixture
def playwright_webview():
    """Create a Playwright-based headless WebView.

    Same as headless_webview, explicit name for clarity.
    """
    with HeadlessWebView.playwright() as webview:
        yield webview


@pytest.fixture
def test_html():
    """Provide sample HTML for testing."""
    return """
    <!DOCTYPE html>
    <html>
        <head>
            <title>Test Page</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 20px; }
                button { padding: 10px 20px; cursor: pointer; }
                #output { margin-top: 10px; padding: 10px; border: 1px solid #ccc; }
            </style>
        </head>
        <body>
            <h1>Test Page</h1>
            <button id="testBtn" class="test-button">Test Button</button>
            <div id="output"></div>
            <script>
                document.getElementById('testBtn').addEventListener('click', function() {
                    document.getElementById('output').textContent = 'Button clicked!';
                    if (window.auroraview && window.auroraview.trigger) {
                        window.auroraview.trigger('button_clicked', {});
                    }
                });
            </script>
        </body>
    </html>
    """


@pytest.fixture
def form_html():
    """Provide HTML with a form for testing."""
    return """
    <!DOCTYPE html>
    <html>
        <head>
            <title>Form Test</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 20px; }
                .form-group { margin-bottom: 15px; }
                label { display: block; margin-bottom: 5px; }
                input, textarea { width: 100%; padding: 8px; box-sizing: border-box; }
                button { padding: 10px 20px; cursor: pointer; }
                #result { margin-top: 10px; padding: 10px; border: 1px solid #ccc; }
            </style>
        </head>
        <body>
            <h1>Form Test</h1>
            <form id="testForm">
                <div class="form-group">
                    <label for="name">Name:</label>
                    <input type="text" id="name" name="name" />
                </div>
                <div class="form-group">
                    <label for="email">Email:</label>
                    <input type="email" id="email" name="email" />
                </div>
                <div class="form-group">
                    <label for="message">Message:</label>
                    <textarea id="message" name="message"></textarea>
                </div>
                <button type="submit" id="submitBtn">Submit</button>
            </form>
            <div id="result"></div>
            <script>
                document.getElementById('testForm').addEventListener('submit', function(e) {
                    e.preventDefault();
                    const name = document.getElementById('name').value;
                    const email = document.getElementById('email').value;
                    const message = document.getElementById('message').value;
                    document.getElementById('result').textContent =
                        'Submitted: ' + name + ', ' + email;
                    if (window.auroraview && window.auroraview.trigger) {
                        window.auroraview.trigger('form_submitted', {name, email, message});
                    }
                });
            </script>
        </body>
    </html>
    """


@pytest.fixture
def draggable_window_html():
    """Provide HTML for testing window dragging."""
    return """
    <!DOCTYPE html>
    <html>
        <head>
            <title>Draggable Window</title>
            <style>
                body { margin: 0; font-family: Arial, sans-serif; }
                .title-bar {
                    background: #333;
                    color: white;
                    padding: 10px 15px;
                    cursor: move;
                    user-select: none;
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                }
                .title-bar .title { font-weight: bold; }
                .title-bar .controls button {
                    background: none;
                    border: none;
                    color: white;
                    cursor: pointer;
                    padding: 5px 10px;
                }
                #content { padding: 20px; }
            </style>
        </head>
        <body>
            <div class="title-bar" id="titleBar">
                <span class="title">Draggable Window</span>
                <div class="controls">
                    <button id="minimizeBtn">_</button>
                    <button id="closeBtn">X</button>
                </div>
            </div>
            <div id="content">
                <p>This window can be dragged by the title bar.</p>
            </div>
            <script>
                const titleBar = document.getElementById('titleBar');
                titleBar.addEventListener('mousedown', function(e) {
                    if (e.target.tagName !== 'BUTTON') {
                        if (window.auroraview && window.auroraview.trigger) {
                            window.auroraview.trigger('drag_start', {x: e.clientX, y: e.clientY});
                        }
                    }
                });
            </script>
        </body>
    </html>
    """
