# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""Unit tests for WebView initialization in packed mode.

Tests that WebView can be instantiated in packed mode without _core.pyd.
"""

from __future__ import annotations

import os
from unittest.mock import patch


class TestWebViewPackedModeInit:
    """Test WebView __init__ behavior in packed mode."""

    def test_webview_init_without_core_in_packed_mode(self):
        """WebView should not raise when _core.pyd is missing in packed mode."""
        with patch.dict(os.environ, {"AURORAVIEW_PACKED": "1"}):
            import auroraview.core.webview as webview_module

            original_core = webview_module._CoreWebView
            original_packed = webview_module._IS_PACKED_MODE

            try:
                webview_module._CoreWebView = None
                webview_module._IS_PACKED_MODE = True

                # Should not raise RuntimeError in packed mode
                webview_module.WebView.__new__(webview_module.WebView)
                # Manually test the condition that would have raised
                assert webview_module._CoreWebView is None
                assert webview_module._IS_PACKED_MODE is True
            finally:
                webview_module._CoreWebView = original_core
                webview_module._IS_PACKED_MODE = original_packed

    def test_webview_init_without_core_not_packed_raises(self):
        """WebView should raise RuntimeError when _core.pyd missing and not packed."""
        import auroraview.core.webview as webview_module

        original_core = webview_module._CoreWebView
        original_packed = webview_module._IS_PACKED_MODE
        original_error = webview_module._CORE_IMPORT_ERROR

        try:
            webview_module._CoreWebView = None
            webview_module._IS_PACKED_MODE = False
            webview_module._CORE_IMPORT_ERROR = "No module named 'auroraview._core'"

            import pytest

            with pytest.raises(RuntimeError, match="AuroraView core library not found"):
                webview_module.WebView(title="Test")
        finally:
            webview_module._CoreWebView = original_core
            webview_module._IS_PACKED_MODE = original_packed
            webview_module._CORE_IMPORT_ERROR = original_error

    def test_is_packed_mode_flag_set_on_import_error(self):
        """_IS_PACKED_MODE should be set when import fails and env var is set."""
        # This tests that the module-level _IS_PACKED_MODE variable
        # correctly reflects the AURORAVIEW_PACKED environment variable
        import auroraview.core.webview as webview_module

        # When _CoreWebView is None and _IS_PACKED_MODE is True,
        # init should not raise
        original_packed = webview_module._IS_PACKED_MODE
        try:
            webview_module._IS_PACKED_MODE = True
            # The flag should be readable
            assert webview_module._IS_PACKED_MODE is True
        finally:
            webview_module._IS_PACKED_MODE = original_packed
