"""Tests for new CLI features: window control, file protocol, always-on-top."""

import subprocess
import sys
from pathlib import Path

import pytest


class TestCLINewFeatures:
    """Test new CLI features added in this PR."""

    @pytest.fixture
    def cli_module_path(self):
        """Get path to CLI module."""
        project_root = Path(__file__).parent.parent
        return project_root / "python" / "auroraview" / "__main__.py"

    def test_cli_has_allow_new_window_argument(self, cli_module_path):
        """Test that CLI has --allow-new-window argument."""
        if not cli_module_path.exists():
            pytest.skip("CLI module not found")

        content = cli_module_path.read_text(encoding="utf-8")
        assert "--allow-new-window" in content
        assert "Allow opening new windows" in content

    def test_cli_has_allow_file_protocol_argument(self, cli_module_path):
        """Test that CLI has --allow-file-protocol argument."""
        if not cli_module_path.exists():
            pytest.skip("CLI module not found")

        content = cli_module_path.read_text(encoding="utf-8")
        assert "--allow-file-protocol" in content
        assert "file://" in content

    def test_cli_has_always_on_top_argument(self, cli_module_path):
        """Test that CLI has --always-on-top argument."""
        if not cli_module_path.exists():
            pytest.skip("CLI module not found")

        content = cli_module_path.read_text(encoding="utf-8")
        assert "--always-on-top" in content
        assert "always on top" in content.lower()

    def test_cli_help_shows_new_arguments(self, cli_module_path):
        """Test that CLI help shows new arguments."""
        if not cli_module_path.exists():
            pytest.skip("CLI module not found")

        try:
            result = subprocess.run(
                [sys.executable, "-m", "auroraview", "--help"],
                capture_output=True,
                text=True,
                timeout=5,
            )

            help_text = result.stdout + result.stderr

            # Check for new arguments in help
            assert "--allow-new-window" in help_text
            assert "--allow-file-protocol" in help_text
            assert "--always-on-top" in help_text

        except (subprocess.TimeoutExpired, FileNotFoundError):
            pytest.skip("Could not run CLI help")

    def test_cli_width_height_zero_documentation(self, cli_module_path):
        """Test that CLI documents width/height=0 for maximization."""
        if not cli_module_path.exists():
            pytest.skip("CLI module not found")

        content = cli_module_path.read_text(encoding="utf-8")

        # Check that help text mentions setting to 0 for maximize
        assert "set to 0 to maximize" in content.lower()


class TestRustCLINewFeatures:
    """Test Rust CLI new features."""

    @pytest.fixture
    def rust_cli_path(self):
        """Get path to Rust CLI source."""
        project_root = Path(__file__).parent.parent
        return project_root / "src" / "bin" / "cli.rs"

    def test_rust_cli_has_allow_new_window_flag(self, rust_cli_path):
        """Test that Rust CLI has allow_new_window flag."""
        if not rust_cli_path.exists():
            pytest.skip("Rust CLI source not found")

        content = rust_cli_path.read_text(encoding="utf-8")
        assert "allow_new_window" in content

    def test_rust_cli_has_allow_file_protocol_flag(self, rust_cli_path):
        """Test that Rust CLI has allow_file_protocol flag."""
        if not rust_cli_path.exists():
            pytest.skip("Rust CLI source not found")

        content = rust_cli_path.read_text(encoding="utf-8")
        assert "allow_file_protocol" in content

    def test_rust_cli_has_always_on_top_flag(self, rust_cli_path):
        """Test that Rust CLI has always_on_top flag."""
        if not rust_cli_path.exists():
            pytest.skip("Rust CLI source not found")

        content = rust_cli_path.read_text(encoding="utf-8")
        assert "always_on_top" in content

    def test_rust_cli_implements_window_maximization(self, rust_cli_path):
        """Test that Rust CLI implements window maximization logic."""
        if not rust_cli_path.exists():
            pytest.skip("Rust CLI source not found")

        content = rust_cli_path.read_text(encoding="utf-8")

        # Check for maximization logic
        assert "with_maximized" in content
        assert (
            "width == 0 || height == 0" in content
            or "args.width == 0 || args.height == 0" in content
        )


class TestConfigNewFields:
    """Test WebViewConfig new fields."""

    @pytest.fixture
    def config_rs_path(self):
        """Get path to config.rs."""
        project_root = Path(__file__).parent.parent
        return project_root / "src" / "webview" / "config.rs"

    def test_config_has_allow_new_window_field(self, config_rs_path):
        """Test that WebViewConfig has allow_new_window field."""
        if not config_rs_path.exists():
            pytest.skip("config.rs not found")

        content = config_rs_path.read_text(encoding="utf-8")
        assert "pub allow_new_window: bool" in content

    def test_config_has_allow_file_protocol_field(self, config_rs_path):
        """Test that WebViewConfig has allow_file_protocol field."""
        if not config_rs_path.exists():
            pytest.skip("config.rs not found")

        content = config_rs_path.read_text(encoding="utf-8")
        assert "pub allow_file_protocol: bool" in content
