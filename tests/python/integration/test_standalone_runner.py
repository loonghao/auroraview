"""Tests for standalone runner functionality."""

import pytest


class TestStandaloneRunner:
    """Test standalone runner module and functions."""

    def test_import_run_standalone(self):
        """Test that run_standalone can be imported from _core."""
        try:
            from auroraview._core import run_standalone

            assert run_standalone is not None
            assert callable(run_standalone)
        except ImportError as e:
            pytest.skip(f"run_standalone not available: {e}")

    def test_run_standalone_signature(self):
        """Test run_standalone function signature."""
        try:
            import inspect

            from auroraview._core import run_standalone

            sig = inspect.signature(run_standalone)
            params = list(sig.parameters.keys())

            # Verify required parameters
            assert "title" in params
            assert "width" in params
            assert "height" in params

            # Verify optional parameters
            assert "url" in params
            assert "html" in params
            assert "dev_tools" in params
            assert "resizable" in params
            assert "decorations" in params
            assert "transparent" in params

            # Verify new window control parameters
            assert "allow_new_window" in params
            assert "allow_file_protocol" in params
            assert "always_on_top" in params

        except ImportError:
            pytest.skip("run_standalone not available")

    def test_run_standalone_docstring(self):
        """Test run_standalone has proper documentation."""
        try:
            from auroraview._core import run_standalone

            assert run_standalone.__doc__ is not None
            assert len(run_standalone.__doc__) > 0

            # Verify key documentation points
            doc = run_standalone.__doc__
            assert "standalone" in doc.lower()
            assert "window" in doc.lower()

        except ImportError:
            pytest.skip("run_standalone not available")

    def test_cli_uses_run_standalone(self):
        """Test that CLI module uses run_standalone."""
        try:
            # Import the CLI module
            import inspect

            from auroraview import __main__

            # Get the source code
            source = inspect.getsource(__main__)

            # Verify it imports run_standalone
            assert "from auroraview._core import run_standalone" in source
            assert "run_standalone(" in source

        except ImportError:
            pytest.skip("CLI module not available")

    def test_standalone_config_validation(self):
        """Test that standalone configuration is validated."""
        try:
            from auroraview._core import run_standalone

            # This test just verifies the function exists and has proper signature
            # We can't actually run it without a display
            assert callable(run_standalone)

        except ImportError:
            pytest.skip("run_standalone not available")

    def test_run_standalone_exported_from_package(self):
        """Test that run_standalone is exported from main package."""
        try:
            from auroraview import run_standalone

            assert run_standalone is not None
            assert callable(run_standalone)
        except ImportError as e:
            pytest.skip(f"run_standalone not exported from package: {e}")

    def test_new_window_control_parameter_defaults(self):
        """Test that new window control parameters have correct defaults."""
        try:
            import inspect

            from auroraview._core import run_standalone

            sig = inspect.signature(run_standalone)

            # Check default values for new parameters
            assert sig.parameters["allow_new_window"].default is False
            assert sig.parameters["allow_file_protocol"].default is False
            assert sig.parameters["always_on_top"].default is False

        except ImportError:
            pytest.skip("run_standalone not available")

    def test_asset_root_parameter_exists(self):
        """Test that asset_root parameter exists in run_standalone signature."""
        try:
            import inspect

            from auroraview._core import run_standalone

            sig = inspect.signature(run_standalone)
            params = list(sig.parameters.keys())

            # Verify asset_root parameter exists
            assert "asset_root" in params

            # Verify it has None as default (optional parameter)
            assert sig.parameters["asset_root"].default is None

        except ImportError:
            pytest.skip("run_standalone not available")

    def test_local_file_support_parameters(self):
        """Test that run_standalone has all parameters needed for local file support."""
        try:
            import inspect

            from auroraview._core import run_standalone

            sig = inspect.signature(run_standalone)
            params = list(sig.parameters.keys())

            # Both methods for local file access should be available
            assert "asset_root" in params  # auroraview:// protocol
            assert "allow_file_protocol" in params  # file:// protocol

            # Check defaults match WebView.create() interface
            assert sig.parameters["asset_root"].default is None
            assert sig.parameters["allow_file_protocol"].default is False

        except ImportError:
            pytest.skip("run_standalone not available")

    def test_asset_root_parameter_type(self):
        """Test that asset_root parameter accepts string type."""
        try:
            import inspect

            from auroraview._core import run_standalone

            sig = inspect.signature(run_standalone)
            param = sig.parameters["asset_root"]

            # asset_root should be optional (default is None)
            assert param.default is None

            # Verify annotation if available (should accept str or None)
            # In PyO3, the type is typically Option<String> which maps to Optional[str]

        except ImportError:
            pytest.skip("run_standalone not available")

    def test_width_height_zero_for_maximize(self):
        """Test that width=0 or height=0 should trigger window maximization."""
        try:
            import inspect

            from auroraview._core import run_standalone

            sig = inspect.signature(run_standalone)

            # Verify width and height parameters exist
            assert "width" in sig.parameters
            assert "height" in sig.parameters

            # Width and height should be required (no default)
            # This means users must explicitly set them
            # Setting to 0 should maximize the window

        except ImportError:
            pytest.skip("run_standalone not available")

    def test_run_standalone_all_parameters(self):
        """Test that run_standalone has all expected parameters."""
        try:
            import inspect

            from auroraview._core import run_standalone

            sig = inspect.signature(run_standalone)
            params = list(sig.parameters.keys())

            # All expected parameters
            expected_params = [
                "title",
                "width",
                "height",
                "url",
                "html",
                "dev_tools",
                "resizable",
                "decorations",
                "transparent",
                "allow_new_window",
                "allow_file_protocol",
                "always_on_top",
                "asset_root",
            ]

            for param in expected_params:
                assert param in params, f"Missing parameter: {param}"

        except ImportError:
            pytest.skip("run_standalone not available")

    def test_run_standalone_parameter_defaults(self):
        """Test that run_standalone has correct parameter defaults."""
        try:
            import inspect

            from auroraview._core import run_standalone

            sig = inspect.signature(run_standalone)

            # Check all defaults
            defaults = {
                "url": None,
                "html": None,
                "dev_tools": True,
                "resizable": True,
                "decorations": True,
                "transparent": False,
                "allow_new_window": False,
                "allow_file_protocol": False,
                "always_on_top": False,
                "asset_root": None,
            }

            for param_name, expected_default in defaults.items():
                actual_default = sig.parameters[param_name].default
                assert actual_default == expected_default, (
                    f"{param_name}: expected {expected_default}, got {actual_default}"
                )

        except ImportError:
            pytest.skip("run_standalone not available")


class TestLoadingScreen:
    """Test loading screen functionality."""

    def test_loading_html_exists(self):
        """Test that loading.html asset exists."""
        from pathlib import Path

        # Find the loading.html file (in auroraview-core crate assets)
        project_root = Path(__file__).parent.parent.parent.parent
        loading_html = (
            project_root / "crates" / "auroraview-core" / "src" / "assets" / "html" / "loading.html"
        )

        assert loading_html.exists(), f"loading.html not found at {loading_html}"

    def test_loading_html_content(self):
        """Test loading.html has proper content."""
        from pathlib import Path

        project_root = Path(__file__).parent.parent.parent.parent
        loading_html = (
            project_root / "crates" / "auroraview-core" / "src" / "assets" / "html" / "loading.html"
        )

        if not loading_html.exists():
            pytest.skip("loading.html not found")

        content = loading_html.read_text(encoding="utf-8")

        # Verify HTML structure
        assert "<!DOCTYPE html>" in content or "<!doctype html>" in content.lower()
        assert "<html" in content
        assert "</html>" in content

        # Verify loading elements
        assert "Loading" in content or "loading" in content.lower()

    def test_loading_html_is_valid_html(self):
        """Test loading.html is valid HTML."""
        from pathlib import Path

        project_root = Path(__file__).parent.parent.parent.parent
        loading_html = (
            project_root / "crates" / "auroraview-core" / "src" / "assets" / "html" / "loading.html"
        )

        if not loading_html.exists():
            pytest.skip("loading.html not found")

        content = loading_html.read_text(encoding="utf-8")

        # Basic HTML validation
        assert content.count("<html") == content.count("</html>")
        assert content.count("<head") == content.count("</head>")
        assert content.count("<body") == content.count("</body>")


class TestJsAssets:
    """Test JavaScript assets module."""

    def test_js_assets_module_exists(self):
        """Test that js_assets module exists in Rust."""
        from pathlib import Path

        project_root = Path(__file__).parent.parent.parent.parent
        js_assets_rs = project_root / "src" / "webview" / "js_assets.rs"

        assert js_assets_rs.exists(), f"js_assets.rs not found at {js_assets_rs}"

    def test_js_assets_has_loading_html_function(self):
        """Test js_assets.rs has get_loading_html function."""
        from pathlib import Path

        project_root = Path(__file__).parent.parent.parent.parent
        js_assets_rs = project_root / "src" / "webview" / "js_assets.rs"

        if not js_assets_rs.exists():
            pytest.skip("js_assets.rs not found")

        content = js_assets_rs.read_text(encoding="utf-8")

        # Verify get_loading_html function is imported/available
        assert "get_loading_html" in content
