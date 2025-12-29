"""Tests for the AuroraView testing framework.

These tests verify that the testing utilities work correctly,
including the Midscene bridge script injection.
"""

from __future__ import annotations

import pytest


class TestMidsceneBridge:
    """Tests for Midscene bridge script management."""

    def test_get_bridge_script_returns_string(self):
        """Test that get_midscene_bridge_script returns a non-empty string."""
        from auroraview.testing.midscene import get_midscene_bridge_script

        script = get_midscene_bridge_script()
        assert isinstance(script, str)
        assert len(script) > 0

    def test_bridge_script_contains_expected_functions(self):
        """Test that bridge script contains expected API functions."""
        from auroraview.testing.midscene import get_midscene_bridge_script

        script = get_midscene_bridge_script()

        # Core functions
        assert "__midscene_bridge__" in script
        assert "getSimplifiedDOM" in script
        assert "getPageInfo" in script

        # Interaction functions
        assert "clickAt" in script or "clickSelector" in script
        assert "typeText" in script

    def test_bridge_script_is_iife(self):
        """Test that bridge script is wrapped in an IIFE."""
        from auroraview.testing.midscene import get_midscene_bridge_script

        script = get_midscene_bridge_script()
        # Should start with IIFE pattern
        assert script.strip().startswith("(function")

    def test_bridge_script_caching(self):
        """Test that bridge script is cached after first load."""
        from auroraview.testing.midscene import get_midscene_bridge_script

        script1 = get_midscene_bridge_script()
        script2 = get_midscene_bridge_script()

        # Should be the same object (cached)
        assert script1 is script2

    def test_try_load_from_rust_core(self):
        """Test loading bridge script from Rust core if available."""
        try:
            from auroraview import _core

            if hasattr(_core, "get_midscene_bridge_js"):
                script = _core.get_midscene_bridge_js()
                assert isinstance(script, str)
                # Rust version should contain the bridge
                if script:
                    assert "__midscene_bridge__" in script
        except ImportError:
            pytest.skip("Rust core not available")


class TestMidsceneConfig:
    """Tests for MidsceneConfig."""

    def test_default_config(self):
        """Test default configuration values."""
        from auroraview.testing.midscene import MidsceneConfig

        config = MidsceneConfig()
        assert config.model_name == "gpt-4o"
        assert config.timeout == 60000
        assert config.cacheable is True
        assert config.debug is False

    def test_config_to_env_vars(self):
        """Test converting config to environment variables."""
        from auroraview.testing.midscene import MidsceneConfig

        config = MidsceneConfig(
            model_name="qwen-vl-plus",
            api_key="test-key",
            debug=True,
        )

        env = config.to_env_vars()
        assert env["MIDSCENE_MODEL_NAME"] == "qwen-vl-plus"
        assert env["MIDSCENE_MODEL_API_KEY"] == "test-key"
        assert env["MIDSCENE_DEBUG"] == "1"
        assert env["MIDSCENE_MODEL_FAMILY"] == "qwen"

    def test_config_auto_detect_model_family(self):
        """Test auto-detection of model family from model name."""
        from auroraview.testing.midscene import MidsceneConfig

        # OpenAI models
        config = MidsceneConfig(model_name="gpt-4o")
        assert config.to_env_vars()["MIDSCENE_MODEL_FAMILY"] == "openai"

        # Qwen models
        config = MidsceneConfig(model_name="qwen-vl-plus")
        assert config.to_env_vars()["MIDSCENE_MODEL_FAMILY"] == "qwen"

        # Gemini models
        config = MidsceneConfig(model_name="gemini-1.5-flash")
        assert config.to_env_vars()["MIDSCENE_MODEL_FAMILY"] == "gemini"

        # Claude models
        config = MidsceneConfig(model_name="claude-3-opus")
        assert config.to_env_vars()["MIDSCENE_MODEL_FAMILY"] == "anthropic"


class TestMidsceneAgent:
    """Tests for MidsceneAgent initialization."""

    def test_agent_init(self):
        """Test agent initialization with mock page."""
        from auroraview.testing.midscene import MidsceneAgent, MidsceneConfig

        # Create a mock page object
        class MockPage:
            pass

        page = MockPage()
        config = MidsceneConfig()

        agent = MidsceneAgent(page, config)
        assert agent._page is page
        assert agent._config is config
        assert agent._initialized is False

    def test_agent_context_manager(self):
        """Test agent can be used as async context manager."""
        from auroraview.testing.midscene import MidsceneAgent

        class MockPage:
            async def evaluate(self, script):
                return True

        agent = MidsceneAgent(MockPage())

        # Should have async context manager methods
        assert hasattr(agent, "__aenter__")
        assert hasattr(agent, "__aexit__")


class TestTestingModuleExports:
    """Tests for testing module exports."""

    def test_midscene_exports(self):
        """Test that Midscene classes are exported from testing module."""
        from auroraview.testing import (
            MidsceneActionResult,
            MidsceneAgent,
            MidsceneConfig,
            MidscenePlaywrightFixture,
            MidsceneQueryResult,
            get_midscene_bridge_script,
            inject_midscene_bridge,
            pytest_ai_fixture,
        )

        # Just verify imports work
        assert MidsceneConfig is not None
        assert MidsceneAgent is not None
        assert MidsceneActionResult is not None
        assert MidsceneQueryResult is not None
        assert MidscenePlaywrightFixture is not None
        assert pytest_ai_fixture is not None
        assert get_midscene_bridge_script is not None
        assert inject_midscene_bridge is not None

    def test_all_exports_in_module_all(self):
        """Test that exported items are in __all__."""
        from auroraview import testing

        expected = [
            "MidsceneConfig",
            "MidsceneAgent",
            "MidsceneActionResult",
            "MidsceneQueryResult",
            "MidscenePlaywrightFixture",
            "pytest_ai_fixture",
            "get_midscene_bridge_script",
            "inject_midscene_bridge",
        ]

        for name in expected:
            assert name in testing.__all__, f"{name} not in __all__"


class TestRustCoreAssets:
    """Tests for Rust core asset functions (src/bindings/assets.rs coverage)."""

    def test_get_midscene_bridge_js(self):
        """Test get_midscene_bridge_js returns non-empty string."""
        try:
            from auroraview._core import get_midscene_bridge_js

            script = get_midscene_bridge_js()
            assert isinstance(script, str)
            assert len(script) > 0
            assert "__midscene_bridge__" in script
        except ImportError:
            pytest.skip("Rust core not available")

    def test_get_event_bridge_js(self):
        """Test get_event_bridge_js returns non-empty string."""
        try:
            from auroraview._core import get_event_bridge_js

            script = get_event_bridge_js()
            assert isinstance(script, str)
            assert len(script) > 0
            assert "auroraview" in script
        except ImportError:
            pytest.skip("Rust core not available")

    def test_get_test_callback_js(self):
        """Test get_test_callback_js returns non-empty string."""
        try:
            from auroraview._core import get_test_callback_js

            script = get_test_callback_js()
            assert isinstance(script, str)
            assert len(script) > 0
        except ImportError:
            pytest.skip("Rust core not available")

    def test_get_context_menu_js(self):
        """Test get_context_menu_js returns a string."""
        try:
            from auroraview._core import get_context_menu_js

            script = get_context_menu_js()
            assert isinstance(script, str)
            # May be empty if context menu is not disabled
        except ImportError:
            pytest.skip("Rust core not available")

    def test_all_rust_asset_functions_available(self):
        """Test that all Rust asset functions are available."""
        try:
            from auroraview import _core

            # Check all expected functions exist
            expected_functions = [
                "get_midscene_bridge_js",
                "get_event_bridge_js",
                "get_test_callback_js",
                "get_context_menu_js",
            ]

            for func_name in expected_functions:
                assert hasattr(_core, func_name), f"Missing function: {func_name}"
                func = getattr(_core, func_name)
                assert callable(func), f"{func_name} is not callable"
        except ImportError:
            pytest.skip("Rust core not available")

    def test_rust_asset_functions_return_consistent_results(self):
        """Test that Rust asset functions return consistent results on multiple calls."""
        try:
            from auroraview._core import (
                get_event_bridge_js,
                get_midscene_bridge_js,
                get_test_callback_js,
            )

            # Call each function twice and compare
            midscene1 = get_midscene_bridge_js()
            midscene2 = get_midscene_bridge_js()
            assert midscene1 == midscene2

            event1 = get_event_bridge_js()
            event2 = get_event_bridge_js()
            assert event1 == event2

            callback1 = get_test_callback_js()
            callback2 = get_test_callback_js()
            assert callback1 == callback2
        except ImportError:
            pytest.skip("Rust core not available")
