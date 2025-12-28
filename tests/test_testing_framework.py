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
            MidsceneConfig,
            MidsceneAgent,
            MidsceneActionResult,
            MidsceneQueryResult,
            MidscenePlaywrightFixture,
            pytest_ai_fixture,
            get_midscene_bridge_script,
            inject_midscene_bridge,
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
