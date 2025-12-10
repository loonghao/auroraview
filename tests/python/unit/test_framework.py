"""Tests for auroraview.framework module."""

from unittest.mock import MagicMock, patch

import pytest


class TestAuroraViewCreation:
    """Tests for AuroraView class creation."""

    def test_auroraview_import(self):
        """Test that AuroraView can be imported."""
        from auroraview.integration.framework import AuroraView

        assert AuroraView is not None

    def test_auroraview_creation_with_url(self):
        """Test AuroraView creation with URL."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView") as mock_webview:
            mock_instance = MagicMock()
            mock_webview.return_value = mock_instance

            tool = AuroraView(url="http://localhost:3000", title="Test Tool")

            assert tool._url == "http://localhost:3000"
            assert tool._title == "Test Tool"
            mock_webview.assert_called_once()

    def test_auroraview_creation_with_html(self):
        """Test AuroraView creation with HTML content."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView") as mock_webview:
            mock_instance = MagicMock()
            mock_webview.return_value = mock_instance

            html_content = "<html><body>Hello</body></html>"
            tool = AuroraView(html=html_content)

            assert tool._html == html_content

    def test_auroraview_creation_with_custom_view(self):
        """Test AuroraView creation with custom view."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        tool = AuroraView(_view=mock_view)

        assert tool._view == mock_view

    def test_auroraview_default_dimensions(self):
        """Test AuroraView default dimensions."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView"):
            tool = AuroraView()

            assert tool._width == 800
            assert tool._height == 600

    def test_auroraview_custom_dimensions(self):
        """Test AuroraView with custom dimensions."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView"):
            tool = AuroraView(width=1024, height=768)

            assert tool._width == 1024
            assert tool._height == 768


class TestAuroraViewKeepAlive:
    """Tests for AuroraView keep-alive registry."""

    def test_instance_registered(self):
        """Test that instance is registered in keep-alive registry."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView"):
            initial_count = len(AuroraView._instances)
            tool = AuroraView()

            assert tool in AuroraView._instances
            assert len(AuroraView._instances) == initial_count + 1

            # Cleanup
            tool.close()

    def test_instance_unregistered_on_close(self):
        """Test that instance is unregistered on close."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView"):
            tool = AuroraView()
            assert tool in AuroraView._instances

            tool.close()
            assert tool not in AuroraView._instances


class TestAuroraViewDelegation:
    """Tests for AuroraView delegation methods."""

    def test_view_property(self):
        """Test view property returns underlying view."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        tool = AuroraView(_view=mock_view)

        assert tool.view == mock_view
        tool.close()

    def test_emit_delegates_to_view(self):
        """Test emit delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.emit = MagicMock()
        tool = AuroraView(_view=mock_view)

        tool.emit("test_event", {"data": "value"})

        mock_view.emit.assert_called_once_with("test_event", {"data": "value"})
        tool.close()

    def test_bind_call_delegates_to_view(self):
        """Test bind_call delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

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
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No bind_call
        tool = AuroraView(_view=mock_view)

        with pytest.raises(RuntimeError, match="does not support bind_call"):
            tool.bind_call("method", lambda: None)

        tool.close()

    def test_bind_api_delegates_to_view(self):
        """Test bind_api delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

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
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No bind_api
        tool = AuroraView(_view=mock_view)

        with pytest.raises(RuntimeError, match="does not support bind_api"):
            tool.bind_api(MagicMock())

        tool.close()


class TestAuroraViewShow:
    """Tests for AuroraView show method."""

    def test_show_delegates_to_view(self):
        """Test show delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.show = MagicMock()
        tool = AuroraView(_view=mock_view)

        tool.show()

        mock_view.show.assert_called_once()
        tool.close()

    def test_show_with_args(self):
        """Test show passes arguments to underlying view."""
        from auroraview.integration.framework import AuroraView

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
        from auroraview.integration.framework import AuroraView

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
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        tool = AuroraView(_view=mock_view)

        tool.close()
        tool.close()  # Should not raise

        assert tool not in AuroraView._instances


class TestAuroraViewAutoShow:
    """Tests for AuroraView auto_show parameter."""

    def test_auto_show_true(self):
        """Test that _auto_show=True calls show automatically."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.show = MagicMock()

        tool = AuroraView(_view=mock_view, _auto_show=True)

        mock_view.show.assert_called_once()
        tool.close()


class TestAuroraViewFullscreen:
    """Tests for AuroraView fullscreen parameter."""

    def test_fullscreen_parameter(self):
        """Test fullscreen parameter is stored."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView"):
            tool = AuroraView(fullscreen=True)

            assert tool._fullscreen is True
            tool.close()


class TestAuroraViewDebug:
    """Tests for AuroraView debug parameter."""

    def test_debug_parameter(self):
        """Test debug parameter is stored."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView"):
            tool = AuroraView(debug=True)

            assert tool._debug is True
            tool.close()


class TestAuroraViewHWND:
    """Tests for AuroraView HWND integration methods."""

    def test_get_hwnd_delegates_to_view(self):
        """Test get_hwnd delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.get_hwnd = MagicMock(return_value=12345)
        tool = AuroraView(_view=mock_view)

        hwnd = tool.get_hwnd()

        assert hwnd == 12345
        mock_view.get_hwnd.assert_called_once()
        tool.close()

    def test_get_hwnd_returns_none_without_support(self):
        """Test get_hwnd returns None if view doesn't support it."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No get_hwnd
        tool = AuroraView(_view=mock_view)

        hwnd = tool.get_hwnd()

        assert hwnd is None
        tool.close()

    def test_set_visible_delegates_to_view(self):
        """Test set_visible delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.set_visible = MagicMock()
        tool = AuroraView(_view=mock_view)

        tool.set_visible(True)
        mock_view.set_visible.assert_called_once_with(True)

        mock_view.set_visible.reset_mock()
        tool.set_visible(False)
        mock_view.set_visible.assert_called_once_with(False)
        tool.close()

    def test_set_visible_without_support(self):
        """Test set_visible does nothing if view doesn't support it."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No set_visible
        tool = AuroraView(_view=mock_view)

        # Should not raise
        tool.set_visible(True)
        tool.close()

    def test_set_position_delegates_to_view(self):
        """Test set_position delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.set_position = MagicMock()
        tool = AuroraView(_view=mock_view)

        tool.set_position(100, 200)

        mock_view.set_position.assert_called_once_with(100, 200)
        tool.close()

    def test_set_position_without_support(self):
        """Test set_position does nothing if view doesn't support it."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No set_position
        tool = AuroraView(_view=mock_view)

        # Should not raise
        tool.set_position(100, 200)
        tool.close()

    def test_set_size_delegates_to_view(self):
        """Test set_size delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.set_size = MagicMock()
        tool = AuroraView(_view=mock_view)

        tool.set_size(1024, 768)

        mock_view.set_size.assert_called_once_with(1024, 768)
        tool.close()

    def test_set_size_without_support(self):
        """Test set_size does nothing if view doesn't support it."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No set_size
        tool = AuroraView(_view=mock_view)

        # Should not raise
        tool.set_size(1024, 768)
        tool.close()


class TestAuroraViewProtocol:
    """Tests for AuroraView custom protocol registration."""

    def test_register_protocol_delegates_to_view(self):
        """Test register_protocol delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.register_protocol = MagicMock()
        tool = AuroraView(_view=mock_view)

        def handler(uri):
            return {"data": b"test", "mime_type": "text/plain", "status": 200}

        tool.register_protocol("myscheme", handler)

        mock_view.register_protocol.assert_called_once_with("myscheme", handler)
        tool.close()

    def test_register_protocol_without_support(self):
        """Test register_protocol logs warning if view doesn't support it."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No register_protocol
        tool = AuroraView(_view=mock_view)

        # Should not raise, just log warning
        tool.register_protocol("myscheme", lambda uri: {})
        tool.close()


class TestAuroraViewState:
    """Tests for AuroraView state property."""

    def test_state_delegates_to_view(self):
        """Test state property delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

        mock_state = MagicMock()
        mock_view = MagicMock()
        mock_view.state = mock_state
        tool = AuroraView(_view=mock_view)

        state = tool.state

        assert state == mock_state
        tool.close()

    def test_state_raises_without_support(self):
        """Test state raises if view doesn't support it."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.state = None
        tool = AuroraView(_view=mock_view)

        with pytest.raises(RuntimeError, match="does not support state"):
            _ = tool.state

        tool.close()


class TestAuroraViewEventDecorators:
    """Tests for AuroraView event decorator methods."""

    def test_on_decorator_delegates_to_view(self):
        """Test on() decorator delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

        mock_decorator = MagicMock(return_value=lambda f: f)
        mock_view = MagicMock()
        mock_view.on = MagicMock(return_value=mock_decorator)
        tool = AuroraView(_view=mock_view)

        _ = tool.on("test_event")

        mock_view.on.assert_called_once_with("test_event")
        tool.close()

    def test_on_raises_without_support(self):
        """Test on() raises if view doesn't support it."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No on method
        tool = AuroraView(_view=mock_view)

        with pytest.raises(RuntimeError, match="does not support on"):
            tool.on("test_event")

        tool.close()

    def test_command_decorator_delegates_to_view(self):
        """Test command() decorator delegates to underlying view."""
        from auroraview.integration.framework import AuroraView

        mock_decorator = MagicMock(return_value=lambda f: f)
        mock_view = MagicMock()
        mock_view.command = MagicMock(return_value=mock_decorator)
        tool = AuroraView(_view=mock_view)

        _ = tool.command("my_command")

        mock_view.command.assert_called_once_with("my_command")
        tool.close()

    def test_command_decorator_without_name(self):
        """Test command() decorator without name."""
        from auroraview.integration.framework import AuroraView

        mock_decorator = MagicMock(return_value=lambda f: f)
        mock_view = MagicMock()
        mock_view.command = MagicMock(return_value=mock_decorator)
        tool = AuroraView(_view=mock_view)

        _ = tool.command()

        mock_view.command.assert_called_once_with()
        tool.close()

    def test_command_raises_without_support(self):
        """Test command() raises if view doesn't support it."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No command method
        tool = AuroraView(_view=mock_view)

        with pytest.raises(RuntimeError, match="does not support command"):
            tool.command("my_command")

        tool.close()


class TestAuroraViewParentHwnd:
    """Tests for AuroraView parent_hwnd parameter."""

    def test_parent_hwnd_alias(self):
        """Test parent_hwnd is alias for parent."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView") as mock_webview:
            mock_instance = MagicMock()
            mock_webview.return_value = mock_instance

            tool = AuroraView(parent_hwnd=12345)

            assert tool._parent == 12345
            # Verify WebView was called with parent=12345
            call_kwargs = mock_webview.call_args[1]
            assert call_kwargs["parent"] == 12345
            tool.close()

    def test_parent_takes_precedence_over_parent_hwnd(self):
        """Test parent parameter takes precedence over parent_hwnd."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView") as mock_webview:
            mock_instance = MagicMock()
            mock_webview.return_value = mock_instance

            # parent is set, parent_hwnd should be ignored
            tool = AuroraView(parent=11111, parent_hwnd=22222)

            assert tool._parent == 11111
            tool.close()


class TestAuroraViewEmbedMode:
    """Tests for AuroraView embed_mode parameter."""

    def test_embed_mode_default(self):
        """Test embed_mode defaults to 'none'."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView") as mock_webview:
            mock_instance = MagicMock()
            mock_webview.return_value = mock_instance

            tool = AuroraView()

            assert tool._embed_mode == "none"
            # Verify WebView was called with mode="none"
            call_kwargs = mock_webview.call_args[1]
            assert call_kwargs["mode"] == "none"
            tool.close()

    def test_embed_mode_owner(self):
        """Test embed_mode='owner' is passed to WebView."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView") as mock_webview:
            mock_instance = MagicMock()
            mock_webview.return_value = mock_instance

            tool = AuroraView(embed_mode="owner")

            assert tool._embed_mode == "owner"
            call_kwargs = mock_webview.call_args[1]
            assert call_kwargs["mode"] == "owner"
            tool.close()

    def test_embed_mode_child(self):
        """Test embed_mode='child' is passed to WebView."""
        from auroraview.integration.framework import AuroraView

        with patch("auroraview.integration.framework.WebView") as mock_webview:
            mock_instance = MagicMock()
            mock_webview.return_value = mock_instance

            tool = AuroraView(embed_mode="child")

            assert tool._embed_mode == "child"
            call_kwargs = mock_webview.call_args[1]
            assert call_kwargs["mode"] == "child"
            tool.close()


class TestAuroraViewApiBinding:
    """Tests for AuroraView API binding."""

    def test_api_defaults_to_self(self):
        """Test api defaults to self when not provided."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.bind_api = MagicMock()
        tool = AuroraView(_view=mock_view)

        assert tool._api is tool
        tool.close()

    def test_api_custom_object(self):
        """Test api can be set to custom object."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.bind_api = MagicMock()

        class MyApi:
            def method(self):
                pass

        api = MyApi()
        tool = AuroraView(_view=mock_view, api=api)

        assert tool._api is api
        # bind_api should be called with the custom api
        mock_view.bind_api.assert_called_once_with(api)
        tool.close()

    def test_api_binding_skipped_without_support(self):
        """Test api binding is skipped if view doesn't support bind_api."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=["close"])  # No bind_api

        class MyApi:
            def method(self):
                pass

        api = MyApi()
        # Should not raise
        tool = AuroraView(_view=mock_view, api=api)

        assert tool._api is api
        tool.close()


class TestAuroraViewLifecycleHooks:
    """Tests for AuroraView lifecycle hooks."""

    def test_on_show_called_after_show(self):
        """Test on_show is called after show()."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_view.show = MagicMock()

        class MyTool(AuroraView):
            def __init__(self, **kwargs):
                self.on_show_called = False
                super().__init__(**kwargs)

            def on_show(self):
                self.on_show_called = True

        tool = MyTool(_view=mock_view)
        assert not tool.on_show_called

        tool.show()
        assert tool.on_show_called
        tool.close()

    def test_on_close_called_after_close(self):
        """Test on_close is called after close()."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()

        class MyTool(AuroraView):
            def __init__(self, **kwargs):
                self.on_close_called = False
                super().__init__(**kwargs)

            def on_close(self):
                self.on_close_called = True

        tool = MyTool(_view=mock_view)
        assert not tool.on_close_called

        tool.close()
        assert tool.on_close_called


class TestAuroraViewKeepAliveRoot:
    """Tests for AuroraView keep_alive_root parameter."""

    def test_keep_alive_root_defaults_to_view(self):
        """Test _keep_alive_root defaults to view."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        tool = AuroraView(_view=mock_view)

        assert tool._keep_alive_root is mock_view
        tool.close()

    def test_keep_alive_root_custom(self):
        """Test _keep_alive_root can be set to custom object."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock()
        mock_root = MagicMock()
        tool = AuroraView(_view=mock_view, _keep_alive_root=mock_root)

        assert tool._keep_alive_root is mock_root
        tool.close()


class TestAuroraViewEmitWithoutSupport:
    """Tests for AuroraView emit without support."""

    def test_emit_does_nothing_without_support(self):
        """Test emit does nothing if view doesn't support it."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No emit
        tool = AuroraView(_view=mock_view)

        # Should not raise
        tool.emit("test_event", {"data": "value"})
        tool.close()


class TestAuroraViewShowWithoutSupport:
    """Tests for AuroraView show without support."""

    def test_show_still_calls_on_show_without_view_support(self):
        """Test show still calls on_show even if view doesn't support show."""
        from auroraview.integration.framework import AuroraView

        mock_view = MagicMock(spec=[])  # No show

        class MyTool(AuroraView):
            def __init__(self, **kwargs):
                self.on_show_called = False
                super().__init__(**kwargs)

            def on_show(self):
                self.on_show_called = True

        tool = MyTool(_view=mock_view)
        tool.show()

        assert tool.on_show_called
        tool.close()
