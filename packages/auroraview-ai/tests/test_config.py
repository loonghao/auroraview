"""Tests for auroraview_ai.config module."""

from __future__ import annotations

import pytest

from auroraview_ai.config import (
    AVAILABLE_MODELS,
    AgentConfig,
    ModelInfo,
    ProviderType,
    SidebarConfig,
    get_model_by_id,
    get_models_for_provider,
)


class TestProviderType:
    """Tests for ProviderType enum."""

    def test_all_providers_defined(self) -> None:
        members = {p.value for p in ProviderType}
        expected = {"openai", "anthropic", "google", "gemini", "deepseek", "ollama", "groq", "mistral", "cohere", "xai", "custom"}
        assert expected.issubset(members)

    def test_provider_values_are_strings(self) -> None:
        for p in ProviderType:
            assert isinstance(p.value, str)


class TestAgentConfig:
    """Tests for AgentConfig dataclass."""

    def test_defaults(self) -> None:
        cfg = AgentConfig()
        assert cfg.model == "gpt-4o"
        assert cfg.temperature == 0.7
        assert cfg.max_tokens == 4096
        assert cfg.system_prompt is None
        assert cfg.api_key is None
        assert cfg.base_url is None
        assert cfg.timeout == 60.0
        assert cfg.provider_options == {}

    def test_get_pydantic_model_name_known(self) -> None:
        cfg = AgentConfig(model="gpt-4o")
        assert cfg.get_pydantic_model_name() == "openai:gpt-4o"

    def test_get_pydantic_model_name_already_prefixed(self) -> None:
        cfg = AgentConfig(model="openai:gpt-4o-mini")
        assert cfg.get_pydantic_model_name() == "openai:gpt-4o-mini"

    def test_get_pydantic_model_name_unknown_defaults_to_openai(self) -> None:
        cfg = AgentConfig(model="my-custom-model")
        assert cfg.get_pydantic_model_name() == "openai:my-custom-model"

    def test_get_pydantic_model_name_anthropic(self) -> None:
        cfg = AgentConfig(model="claude-sonnet-4-0")
        assert cfg.get_pydantic_model_name() == "anthropic:claude-sonnet-4-0"

    def test_get_pydantic_model_name_gemini(self) -> None:
        cfg = AgentConfig(model="gemini-2.0-flash-exp")
        assert cfg.get_pydantic_model_name() == "google-gla:gemini-2.0-flash-exp"

    def test_get_pydantic_model_name_deepseek(self) -> None:
        cfg = AgentConfig(model="deepseek-chat")
        assert cfg.get_pydantic_model_name() == "openai:deepseek-chat"

    def test_infer_provider_openai_gpt(self) -> None:
        assert AgentConfig(model="gpt-4o").infer_provider() == ProviderType.OPENAI

    def test_infer_provider_openai_o1(self) -> None:
        assert AgentConfig(model="o1").infer_provider() == ProviderType.OPENAI

    def test_infer_provider_openai_o1_mini(self) -> None:
        assert AgentConfig(model="o1-mini").infer_provider() == ProviderType.OPENAI

    def test_infer_provider_anthropic(self) -> None:
        assert AgentConfig(model="claude-3-5-sonnet-20241022").infer_provider() == ProviderType.ANTHROPIC

    def test_infer_provider_gemini(self) -> None:
        assert AgentConfig(model="gemini-1.5-pro").infer_provider() == ProviderType.GEMINI

    def test_infer_provider_deepseek(self) -> None:
        assert AgentConfig(model="deepseek-chat").infer_provider() == ProviderType.DEEPSEEK

    def test_infer_provider_ollama_llama(self) -> None:
        assert AgentConfig(model="llama3.2").infer_provider() == ProviderType.OLLAMA

    def test_infer_provider_ollama_mistral(self) -> None:
        assert AgentConfig(model="mistral").infer_provider() == ProviderType.OLLAMA

    def test_infer_provider_ollama_phi(self) -> None:
        assert AgentConfig(model="phi-3").infer_provider() == ProviderType.OLLAMA

    def test_infer_provider_ollama_qwen(self) -> None:
        assert AgentConfig(model="qwen2.5").infer_provider() == ProviderType.OLLAMA

    def test_infer_provider_xai(self) -> None:
        assert AgentConfig(model="grok-2").infer_provider() == ProviderType.XAI

    def test_infer_provider_cohere(self) -> None:
        assert AgentConfig(model="command-r").infer_provider() == ProviderType.COHERE

    def test_infer_provider_custom(self) -> None:
        assert AgentConfig(model="unknown-model-xyz").infer_provider() == ProviderType.CUSTOM

    def test_classmethod_openai(self) -> None:
        cfg = AgentConfig.openai()
        assert cfg.model == "gpt-4o"
        assert cfg.infer_provider() == ProviderType.OPENAI

    def test_classmethod_openai_custom_model(self) -> None:
        cfg = AgentConfig.openai(model="gpt-4o-mini")
        assert cfg.model == "gpt-4o-mini"

    def test_classmethod_anthropic(self) -> None:
        cfg = AgentConfig.anthropic()
        assert cfg.model == "claude-sonnet-4-0"
        assert cfg.infer_provider() == ProviderType.ANTHROPIC

    def test_classmethod_gemini(self) -> None:
        cfg = AgentConfig.gemini()
        assert cfg.model == "gemini-2.0-flash-exp"
        assert cfg.infer_provider() == ProviderType.GEMINI

    def test_classmethod_kwargs_passthrough(self) -> None:
        cfg = AgentConfig.openai(temperature=0.0, max_tokens=100)
        assert cfg.temperature == 0.0
        assert cfg.max_tokens == 100

    def test_provider_options_field_independent(self) -> None:
        cfg1 = AgentConfig()
        cfg2 = AgentConfig()
        cfg1.provider_options["key"] = "value"
        assert "key" not in cfg2.provider_options


class TestSidebarConfig:
    """Tests for SidebarConfig dataclass."""

    def test_defaults(self) -> None:
        cfg = SidebarConfig()
        assert cfg.position == "right"
        assert cfg.width == 380
        assert cfg.min_width == 280
        assert cfg.max_width == 600
        assert cfg.collapsed is False
        assert cfg.resizable is True
        assert cfg.keyboard_shortcut == "Ctrl+Shift+A"
        assert cfg.theme == "auto"
        assert cfg.animation_duration == 200
        assert cfg.animation_easing == "ease-in-out"
        assert cfg.header_title == "AI Assistant"
        assert cfg.placeholder_text == "Ask me anything..."
        assert cfg.show_thinking is True

    def test_custom_position(self) -> None:
        cfg = SidebarConfig(position="left", width=500)
        assert cfg.position == "left"
        assert cfg.width == 500


class TestModelInfo:
    """Tests for ModelInfo dataclass."""

    def test_defaults(self) -> None:
        info = ModelInfo(id="test-id", name="Test", provider=ProviderType.OPENAI)
        assert info.description == ""
        assert info.context_window == 128000
        assert info.supports_vision is False
        assert info.supports_tools is True

    def test_custom_attributes(self) -> None:
        info = ModelInfo(
            id="gpt-4o",
            name="GPT-4o",
            provider=ProviderType.OPENAI,
            description="Capable",
            context_window=128000,
            supports_vision=True,
            supports_tools=True,
        )
        assert info.id == "gpt-4o"
        assert info.supports_vision is True


class TestAvailableModels:
    """Tests for AVAILABLE_MODELS and lookup functions."""

    def test_available_models_not_empty(self) -> None:
        assert len(AVAILABLE_MODELS) > 0

    def test_all_models_have_ids(self) -> None:
        for model in AVAILABLE_MODELS:
            assert model.id
            assert model.name

    def test_get_model_by_id_found(self) -> None:
        model = get_model_by_id("gpt-4o")
        assert model is not None
        assert model.id == "gpt-4o"
        assert model.provider == ProviderType.OPENAI

    def test_get_model_by_id_not_found(self) -> None:
        assert get_model_by_id("nonexistent-model") is None

    def test_get_models_for_provider_openai(self) -> None:
        models = get_models_for_provider(ProviderType.OPENAI)
        assert len(models) > 0
        for m in models:
            assert m.provider == ProviderType.OPENAI

    def test_get_models_for_provider_anthropic(self) -> None:
        models = get_models_for_provider(ProviderType.ANTHROPIC)
        assert len(models) > 0
        for m in models:
            assert m.provider == ProviderType.ANTHROPIC

    def test_get_models_for_provider_empty(self) -> None:
        # GROQ and MISTRAL have no entries in AVAILABLE_MODELS
        models = get_models_for_provider(ProviderType.GROQ)
        assert models == []

    def test_get_models_for_provider_deepseek(self) -> None:
        models = get_models_for_provider(ProviderType.DEEPSEEK)
        assert len(models) > 0

    def test_get_models_for_provider_ollama(self) -> None:
        models = get_models_for_provider(ProviderType.OLLAMA)
        assert len(models) > 0
