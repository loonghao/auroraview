"""
Tests for Maya Outliner example

Tests the Maya outliner WebView integration example.
"""

import pytest


@pytest.mark.maya
def test_maya_outliner_import():
    """Test that Maya outliner example can be imported"""
    try:
        from examples.maya import outliner_example

        assert outliner_example is not None
    except ImportError:
        pytest.skip("Maya not available")


@pytest.mark.maya
def test_maya_outliner_webview_creation():
    """Test creating WebView for Maya outliner"""
    try:
        from auroraview import WebView

        webview = WebView(title="Maya Outliner", width=400, height=600, decorations=False)
        assert webview is not None
        assert webview.title == "Maya Outliner"
    except ImportError:
        pytest.skip("Package not built yet")


@pytest.mark.maya
def test_maya_outliner_html_loading():
    """Test loading HTML for Maya outliner"""
    try:
        from auroraview import WebView

        webview = WebView(title="Maya Outliner")

        html = """
        <html>
            <body>
                <div id="outliner">
                    <ul>
                        <li>pCube1</li>
                        <li>pSphere1</li>
                    </ul>
                </div>
            </body>
        </html>
        """

        webview.load_html(html)
        assert True
    except ImportError:
        pytest.skip("Package not built yet")
