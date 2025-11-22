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


class TestLoadingScreen:
    """Test loading screen functionality."""

    def test_loading_html_exists(self):
        """Test that loading.html asset exists."""
        from pathlib import Path

        # Find the loading.html file
        project_root = Path(__file__).parent.parent
        loading_html = project_root / "src" / "assets" / "html" / "loading.html"

        assert loading_html.exists(), f"loading.html not found at {loading_html}"

    def test_loading_html_content(self):
        """Test loading.html has proper content."""
        from pathlib import Path

        project_root = Path(__file__).parent.parent
        loading_html = project_root / "src" / "assets" / "html" / "loading.html"

        if not loading_html.exists():
            pytest.skip("loading.html not found")

        content = loading_html.read_text(encoding="utf-8")

        # Verify HTML structure
        assert "<!DOCTYPE html>" in content
        assert "<html" in content
        assert "</html>" in content

        # Verify loading elements
        assert "Loading" in content
        assert "spinner" in content

        # Verify styling
        assert "background" in content
        assert "gradient" in content

    def test_loading_html_is_valid_html(self):
        """Test loading.html is valid HTML."""
        from pathlib import Path

        project_root = Path(__file__).parent.parent
        loading_html = project_root / "src" / "assets" / "html" / "loading.html"

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
        # This is tested indirectly through the loading screen
        # The module should be accessible from Rust code
        from pathlib import Path

        project_root = Path(__file__).parent.parent
        js_assets_rs = project_root / "src" / "webview" / "js_assets.rs"

        assert js_assets_rs.exists(), f"js_assets.rs not found at {js_assets_rs}"

    def test_js_assets_has_html_registry(self):
        """Test js_assets.rs has HTML registry function."""
        from pathlib import Path

        project_root = Path(__file__).parent.parent
        js_assets_rs = project_root / "src" / "webview" / "js_assets.rs"

        if not js_assets_rs.exists():
            pytest.skip("js_assets.rs not found")

        content = js_assets_rs.read_text(encoding="utf-8")

        # Verify HTML registry exists
        assert "get_html_registry" in content
        assert "get_loading_html" in content
        assert "loading.html" in content
