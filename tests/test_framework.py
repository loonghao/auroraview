"""Tests for auroraview.framework module."""

from unittest.mock import MagicMock, patch

import pytest


class TestAuroraViewCreation:
    """Tests for AuroraView class creation."""

    def test_auroraview_import(self):
        """Test that AuroraView can be imported."""
        from auroraview.framework import AuroraView

        assert AuroraView is not None

    def test_auroraview_creation_with_url(self):
        """Test AuroraView creation with URL."""
        from auroraview.framework import AuroraView

        with patch("auroraview.framework.WebView") as mock_webview:
            mock_instance = MagicMock()
            mock_webview.return_value = mock_instance

            tool = AuroraView(url="http://localhost:3000", title="Test Tool")

            assert tool._url == "http://localhost:3000"
            assert tool._title == "Test Tool"
            mock_webview.assert_called_once()

    def test_auroraview_creation_with_html(self):
        """Test AuroraView creation with HTML content."""
        from auroraview.framework import AuroraView

        with patch("auroraview.framework.WebView") as mock_webview:
            mock_instance = MagicMock()
            mock_webview.return_value = mock_instance

            html_content = "<html><body>Hello</body></html>"
            tool = AuroraView(html=html_content)

            assert tool._html == html_content

    def test_auroraview_creation_with_custom_view(self):
        """Test AuroraView creation with custom view."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock()
        tool = AuroraView(_view=mock_view)

        assert tool._view == mock_view

    def test_auroraview_default_dimensions(self):
        """Test AuroraView default dimensions."""
        from auroraview.framework import AuroraView

        with patch("auroraview.framework.WebView"):
            tool = AuroraView()

            assert tool._width == 800
            assert tool._height == 600

    def test_auroraview_custom_dimensions(self):
        """Test AuroraView with custom dimensions."""
        from auroraview.framework import AuroraView

        with patch("auroraview.framework.WebView"):
            tool = AuroraView(width=1024, height=768)

            assert tool._width == 1024
            assert tool._height == 768


class TestAuroraViewKeepAlive:
    """Tests for AuroraView keep-alive registry."""

    def test_instance_registered(self):
        """Test that instance is registered in keep-alive registry."""
        from auroraview.framework import AuroraView

        with patch("auroraview.framework.WebView"):
            initial_count = len(AuroraView._instances)
            tool = AuroraView()

            assert tool in AuroraView._instances
            assert len(AuroraView._instances) == initial_count + 1

            # Cleanup
            tool.close()

    def test_instance_unregistered_on_close(self):
        """Test that instance is unregistered on close."""
        from auroraview.framework import AuroraView

        with patch("auroraview.framework.WebView"):
            tool = AuroraView()
            assert tool in AuroraView._instances

            tool.close()
            assert tool not in AuroraView._instances


class TestAuroraViewDelegation:
    """Tests for AuroraView delegation methods."""

    def test_view_property(self):
        """Test view property returns underlying view."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock()
        tool = AuroraView(_view=mock_view)

        assert tool.view == mock_view
        tool.close()

    def test_emit_delegates_to_view(self):
        """Test emit delegates to underlying view."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock()
        mock_view.emit = MagicMock()
        tool = AuroraView(_view=mock_view)

        tool.emit("test_event", {"data": "value"})

        mock_view.emit.assert_called_once_with("test_event", {"data": "value"})
        tool.close()

    def test_bind_call_delegates_to_view(self):
        """Test bind_call delegates to underlying view."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock()
        mock_view.bind_call = MagicMock(return_value=None)
        tool = AuroraView(_view=mock_view)

        def my_func():
            pass

        tool.bind_call("my_method", my_func)

        mock_view.bind_call.assert_called_once_with("my_method", my_func)
        tool.close()

    def test_bind_call_raises_without_support(self):
        """Test bind_call raises if view doesn't support it."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No bind_call
        tool = AuroraView(_view=mock_view)

        with pytest.raises(RuntimeError, match="does not support bind_call"):
            tool.bind_call("method", lambda: None)

        tool.close()

    def test_bind_api_delegates_to_view(self):
        """Test bind_api delegates to underlying view."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock()
        mock_view.bind_api = MagicMock()
        tool = AuroraView(_view=mock_view)

        # Reset mock after __init__ auto-binding
        mock_view.bind_api.reset_mock()

        api_obj = MagicMock()
        tool.bind_api(api_obj, namespace="myapi")

        mock_view.bind_api.assert_called_once_with(api_obj, namespace="myapi")
        tool.close()

    def test_bind_api_raises_without_support(self):
        """Test bind_api raises if view doesn't support it."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No bind_api
        tool = AuroraView(_view=mock_view)

        with pytest.raises(RuntimeError, match="does not support bind_api"):
            tool.bind_api(MagicMock())

        tool.close()


class TestAuroraViewShow:
    """Tests for AuroraView show method."""

    def test_show_delegates_to_view(self):
        """Test show delegates to underlying view."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock()
        mock_view.show = MagicMock()
        tool = AuroraView(_view=mock_view)

        tool.show()

        mock_view.show.assert_called_once()
        tool.close()

    def test_show_with_args(self):
        """Test show passes arguments to underlying view."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock()
        mock_view.show = MagicMock()
        tool = AuroraView(_view=mock_view)

        tool.show(wait=False)

        mock_view.show.assert_called_once_with(wait=False)
        tool.close()


class TestAuroraViewClose:
    """Tests for AuroraView close method."""

    def test_close_with_different_keep_alive_root(self):
        """Test close when keep_alive_root differs from view."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock()
        mock_view.close = MagicMock()
        mock_root = MagicMock()
        mock_root.close = MagicMock()

        tool = AuroraView(_view=mock_view, _keep_alive_root=mock_root)
        tool.close()

        mock_root.close.assert_called_once()
        mock_view.close.assert_called_once()

    def test_close_idempotent(self):
        """Test that close can be called multiple times safely."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock()
        tool = AuroraView(_view=mock_view)

        tool.close()
        tool.close()  # Should not raise

        assert tool not in AuroraView._instances


class TestAuroraViewAutoShow:
    """Tests for AuroraView auto_show parameter."""

    def test_auto_show_true(self):
        """Test that _auto_show=True calls show automatically."""
        from auroraview.framework import AuroraView

        mock_view = MagicMock()
        mock_view.show = MagicMock()

        tool = AuroraView(_view=mock_view, _auto_show=True)

        mock_view.show.assert_called_once()
        tool.close()


class TestAuroraViewFullscreen:
    """Tests for AuroraView fullscreen parameter."""

    def test_fullscreen_parameter(self):
        """Test fullscreen parameter is stored."""
        from auroraview.framework import AuroraView

        with patch("auroraview.framework.WebView"):
            tool = AuroraView(fullscreen=True)

            assert tool._fullscreen is True
            tool.close()


class TestAuroraViewDebug:
    """Tests for AuroraView debug parameter."""

    def test_debug_parameter(self):
        """Test debug parameter is stored."""
        from auroraview.framework import AuroraView

        with patch("auroraview.framework.WebView"):
            tool = AuroraView(debug=True)

            assert tool._debug is True
            tool.close()
