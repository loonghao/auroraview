"""Pytest configuration for AuroraView MCP tests."""

from __future__ import annotations

import pytest


@pytest.fixture
def mock_cdp_response():
    """Create a mock CDP response factory."""

    def _create(result=None, error=None):
        response = {"id": 1}
        if result is not None:
            response["result"] = result
        if error is not None:
            response["error"] = error
        return response

    return _create
