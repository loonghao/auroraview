"""
Tests for maya_integration.config module

This module tests the environment configuration functionality.
"""

import os
import pytest
from pathlib import Path


@pytest.fixture
def clean_env():
    """Clean environment variable before and after test"""
    original = os.environ.get("AURORAVIEW_ENV")
    if "AURORAVIEW_ENV" in os.environ:
        del os.environ["AURORAVIEW_ENV"]
    yield
    if original is not None:
        os.environ["AURORAVIEW_ENV"] = original
    elif "AURORAVIEW_ENV" in os.environ:
        del os.environ["AURORAVIEW_ENV"]


def test_import_config():
    """Test that config module can be imported"""
    from maya_integration.config import (
        EnvironmentConfig,
        get_environment_info,
        get_frontend_url,
    )

    assert EnvironmentConfig is not None
    assert get_environment_info is not None
    assert get_frontend_url is not None


def test_default_is_development(clean_env):
    """Test that default mode is development"""
    from maya_integration.config import EnvironmentConfig

    config = EnvironmentConfig()
    assert config.is_development is True
    assert config.is_production is False


def test_production_mode(clean_env):
    """Test production mode detection"""
    from maya_integration.config import EnvironmentConfig

    # Test "production"
    os.environ["AURORAVIEW_ENV"] = "production"
    config = EnvironmentConfig()
    assert config.is_production is True
    assert config.is_development is False

    # Test "prod"
    os.environ["AURORAVIEW_ENV"] = "prod"
    config = EnvironmentConfig()
    assert config.is_production is True
    assert config.is_development is False


def test_development_mode(clean_env):
    """Test development mode detection"""
    from maya_integration.config import EnvironmentConfig

    # Test "development"
    os.environ["AURORAVIEW_ENV"] = "development"
    config = EnvironmentConfig()
    assert config.is_development is True
    assert config.is_production is False

    # Test "dev"
    os.environ["AURORAVIEW_ENV"] = "dev"
    config = EnvironmentConfig()
    assert config.is_development is True
    assert config.is_production is False


def test_case_insensitive(clean_env):
    """Test that environment variable is case-insensitive"""
    from maya_integration.config import EnvironmentConfig

    # Test uppercase
    os.environ["AURORAVIEW_ENV"] = "PRODUCTION"
    config = EnvironmentConfig()
    assert config.is_production is True

    # Test mixed case
    os.environ["AURORAVIEW_ENV"] = "DeVeLoPmEnT"
    config = EnvironmentConfig()
    assert config.is_development is True


def test_dev_url(clean_env):
    """Test development URL"""
    from maya_integration.config import EnvironmentConfig

    config = EnvironmentConfig()
    dev_url = config.get_dev_url()
    assert dev_url == "http://localhost:5173"


def test_get_url_development(clean_env):
    """Test get_url in development mode"""
    from maya_integration.config import get_frontend_url

    os.environ["AURORAVIEW_ENV"] = "development"
    url = get_frontend_url()
    assert url == "http://localhost:5173"


def test_get_url_force_development(clean_env):
    """Test get_url with force_development flag"""
    from maya_integration.config import get_frontend_url

    os.environ["AURORAVIEW_ENV"] = "production"
    url = get_frontend_url(force_development=True)
    assert url == "http://localhost:5173"


def test_get_url_force_production_no_dist(clean_env):
    """Test get_url with force_production when dist doesn't exist"""
    from maya_integration.config import get_frontend_url

    # Assuming dist doesn't exist in test environment
    with pytest.raises(FileNotFoundError, match="Production mode requested but dist files not found"):
        get_frontend_url(force_production=True)


def test_get_url_conflicting_flags(clean_env):
    """Test get_url with conflicting force flags"""
    from maya_integration.config import get_frontend_url

    with pytest.raises(ValueError, match="Cannot force both production and development mode"):
        get_frontend_url(force_production=True, force_development=True)


def test_environment_info(clean_env):
    """Test get_environment_info"""
    from maya_integration.config import get_environment_info

    os.environ["AURORAVIEW_ENV"] = "development"
    info = get_environment_info()

    assert "env_var" in info
    assert "env_value" in info
    assert "is_production" in info
    assert "is_development" in info
    assert "dist_exists" in info
    assert "dist_path" in info
    assert "index_html_path" in info
    assert "dev_server_url" in info
    assert "current_url" in info

    assert info["env_var"] == "AURORAVIEW_ENV"
    assert info["env_value"] == "development"
    assert info["is_development"] is True
    assert info["is_production"] is False
    assert info["dev_server_url"] == "http://localhost:5173"


def test_dist_path_structure(clean_env):
    """Test that dist path structure is correct"""
    from maya_integration.config import EnvironmentConfig

    config = EnvironmentConfig()
    dist_path = Path(config._dist_dir)
    index_path = Path(config._index_html)

    # Check that paths are relative to project root
    assert dist_path.name == "dist"
    assert index_path.name == "index.html"
    assert index_path.parent == dist_path

