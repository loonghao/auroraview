"""Tests for the WebViewSettings module."""

from auroraview.core.settings import DEFAULT_SETTINGS, WebViewSettings


class TestWebViewSettings:
    """Tests for WebViewSettings class."""

    def test_default_values(self):
        """Test default settings values."""
        settings = WebViewSettings()
        assert settings.javascript_enabled is True
        assert settings.local_storage_enabled is True
        assert settings.dev_tools_enabled is True
        assert settings.context_menu_enabled is True
        assert settings.allow_file_access is False
        assert settings.user_agent is None
        assert settings.background_color is None
        assert settings.zoom_level == 100
        assert settings.minimum_font_size == 0
        assert settings.default_font_size == 16
        assert settings.default_encoding == "UTF-8"

    def test_custom_values(self):
        """Test creating settings with custom values."""
        settings = WebViewSettings(
            javascript_enabled=False,
            dev_tools_enabled=False,
            user_agent="Custom Agent",
            zoom_level=150,
        )
        assert settings.javascript_enabled is False
        assert settings.dev_tools_enabled is False
        assert settings.user_agent == "Custom Agent"
        assert settings.zoom_level == 150
        # Other values should be defaults
        assert settings.local_storage_enabled is True

    def test_set_known_attribute(self):
        """Test setting a known attribute."""
        settings = WebViewSettings()
        settings.set("javascript_enabled", False)
        assert settings.javascript_enabled is False

    def test_set_custom_attribute(self):
        """Test setting a custom attribute."""
        settings = WebViewSettings()
        settings.set("custom_key", "custom_value")
        assert settings.get("custom_key") == "custom_value"

    def test_get_known_attribute(self):
        """Test getting a known attribute."""
        settings = WebViewSettings(user_agent="Test Agent")
        assert settings.get("user_agent") == "Test Agent"

    def test_get_unknown_attribute_with_default(self):
        """Test getting unknown attribute returns default."""
        settings = WebViewSettings()
        assert settings.get("unknown_key", "default") == "default"

    def test_to_dict(self):
        """Test converting settings to dictionary."""
        settings = WebViewSettings(
            javascript_enabled=False,
            user_agent="Test Agent",
        )
        settings.set("custom_key", "custom_value")

        result = settings.to_dict()
        assert result["javascript_enabled"] is False
        assert result["user_agent"] == "Test Agent"
        assert result["custom_key"] == "custom_value"
        # Check all expected keys are present
        assert "local_storage_enabled" in result
        assert "dev_tools_enabled" in result

    def test_from_dict(self):
        """Test creating settings from dictionary."""
        data = {
            "javascript_enabled": False,
            "user_agent": "Test Agent",
            "custom_key": "custom_value",
        }
        settings = WebViewSettings.from_dict(data)
        assert settings.javascript_enabled is False
        assert settings.user_agent == "Test Agent"
        assert settings.get("custom_key") == "custom_value"
        # Defaults should be applied for missing keys
        assert settings.local_storage_enabled is True

    def test_copy(self):
        """Test copying settings."""
        original = WebViewSettings(
            javascript_enabled=False,
            user_agent="Original Agent",
        )
        original.set("custom_key", "custom_value")

        copy = original.copy()
        assert copy.javascript_enabled is False
        assert copy.user_agent == "Original Agent"
        assert copy.get("custom_key") == "custom_value"

        # Modifying copy should not affect original
        copy.javascript_enabled = True
        assert original.javascript_enabled is False


class TestDefaultSettings:
    """Tests for DEFAULT_SETTINGS constant."""

    def test_default_settings_exists(self):
        """Test that DEFAULT_SETTINGS is available."""
        assert DEFAULT_SETTINGS is not None
        assert isinstance(DEFAULT_SETTINGS, WebViewSettings)

    def test_default_settings_values(self):
        """Test DEFAULT_SETTINGS has expected default values."""
        assert DEFAULT_SETTINGS.javascript_enabled is True
        assert DEFAULT_SETTINGS.dev_tools_enabled is True


class TestSettingsImport:
    """Tests for settings module imports."""

    def test_import_from_auroraview(self):
        """Test importing settings from main package."""
        from auroraview import DEFAULT_SETTINGS, WebViewSettings

        assert WebViewSettings is not None
        assert DEFAULT_SETTINGS is not None
