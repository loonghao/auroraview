# -*- coding: utf-8 -*-
"""Unit tests for the AuroraView DCC-MCP adapter.

These tests exercise the adapter contract against a fake WebView, so they run
without a display, a Qt binding, or a compiled Rust core.

The live path (FileRegistry registration, heartbeat, skill loading) is covered
by ``tests/python/integration/test_dcc_mcp_registration.py``, which requires
``dcc-mcp-core`` and is skipped when it is absent.
"""

from __future__ import annotations

import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", "python"))

from auroraview.dcc_mcp import DCC_MCP_CORE_IMPORT_ERROR  # noqa: E402

pytestmark = pytest.mark.unit

_HAS_CORE = DCC_MCP_CORE_IMPORT_ERROR is None

if _HAS_CORE:
    from auroraview.dcc_mcp.adapter import TOOL_SPECS, AuroraViewAdapter
else:  # pragma: no cover - only when the optional dep is missing
    AuroraViewAdapter = None  # type: ignore[assignment]
    TOOL_SPECS = ()


class _FakeCore:
    """Minimal stand-in for the Rust core's JS round-trip API."""

    def __init__(self, result="\"hello\"", status="complete"):
        self.result = result
        self.status = status
        self.scripts = []

    def eval_js_future(self, script, timeout_ms):
        self.scripts.append((script, timeout_ms))
        return "cb-1"

    def get_js_result(self, callback_id):
        return {"status": self.status, "result": self.result}


class _FakeWebView:
    """Minimal stand-in for an AuroraView WebView."""

    def __init__(self, core=None, title="AuroraView", url="http://localhost:3000/"):
        self._core = core if core is not None else _FakeCore()
        self.title = title
        self.current_url = url
        self.loaded_urls = []
        self.loaded_html = []
        self.processed = 0

    def eval_js(self, script):
        self._core.scripts.append((script, None))

    def load_url(self, url):
        self.loaded_urls.append(url)

    def load_html(self, html):
        self.loaded_html.append(html)

    def process_events(self):
        self.processed += 1
        return True


@pytest.fixture()
def adapter():
    if AuroraViewAdapter is None:
        pytest.skip("dcc-mcp-core not installed")
    return AuroraViewAdapter(_FakeWebView(), host_dcc="maya", cdp_port=9222)


def test_dcc_name_and_capabilities_match_webview_host():
    """AuroraView registers as 'auroraview' with no DCC-style capabilities."""
    assert AuroraViewAdapter.dcc_name == "auroraview"
    assert AuroraViewAdapter.capabilities == {
        "scene": False,
        "timeline": False,
        "selection": False,
        "undo": False,
        "render": False,
    }
    assert AuroraViewAdapter.supports("scene") is False
    assert AuroraViewAdapter.matches_requirements(["scene"]) is False
    assert AuroraViewAdapter.matches_requirements([]) is True


def test_advertised_capabilities_returns_mutable_copy():
    """Mutating the returned map must not leak into the class attribute."""
    first = AuroraViewAdapter.advertised_capabilities()
    first["scene"] = True
    assert AuroraViewAdapter.capabilities["scene"] is False


def test_list_tools_exposes_the_four_first_slice_tools(adapter):
    """The first slice mirrors AuroraView's existing MCP tool surface."""
    names = {tool["name"] for tool in adapter.list_tools()}
    assert names == {"eval_js", "screenshot", "load_url", "load_html"}
    for spec in TOOL_SPECS:
        assert spec.name in names


def test_get_context_carries_webview_descriptor(adapter):
    """get_context publishes the fields core's WebViewContext documents."""
    context = adapter.get_context()
    assert context["window_title"] == "AuroraView"
    assert context["url"] == "http://localhost:3000/"
    assert context["cdp_port"] == 9222
    assert context["host_dcc"] == "maya"
    assert context["pid"] == os.getpid()


def test_get_context_omits_host_dcc_when_standalone():
    """A standalone window reports no embedding DCC."""
    if AuroraViewAdapter is None:
        pytest.skip("dcc-mcp-core not installed")
    standalone = AuroraViewAdapter(_FakeWebView())
    assert "host_dcc" not in standalone.get_context()


def test_execute_eval_js_returns_value(adapter):
    """eval_js waits for the result rather than firing and forgetting."""
    result = adapter.execute("eval_js", {"script": "document.title"})
    assert result["ok"] is True
    assert result["result"] == '"hello"'


def test_execute_load_url_delegates_to_webview(adapter):
    """load_url reuses the WebView's own navigation API."""
    result = adapter.execute("load_url", {"url": "https://example.com"})
    assert result["ok"] is True
    assert adapter._view.loaded_urls == ["https://example.com"]


def test_execute_load_html_delegates_to_webview(adapter):
    """load_html reuses the WebView's own content API."""
    result = adapter.execute("load_html", {"html": "<h1>hi</h1>"})
    assert result["ok"] is True
    assert result["length"] == len("<h1>hi</h1>")
    assert adapter._view.loaded_html == ["<h1>hi</h1>"]


def test_execute_screenshot_uses_injected_helper(adapter):
    """screenshot calls window.auroraview.screenshot, not a new bridge."""
    adapter.execute("screenshot", {"format": "png", "full_page": True})
    scripts = adapter._view._core.scripts
    assert scripts
    assert "window.auroraview.screenshot" in scripts[0][0]


def test_execute_unknown_tool_is_reported_not_raised(adapter):
    """An unknown tool returns a structured failure the agent can act on."""
    result = adapter.execute("nope", {})
    assert result["ok"] is False
    assert "unknown tool" in result["error"]


def test_execute_missing_required_param_fails_cleanly(adapter):
    """Missing required parameters surface as an error, not a traceback."""
    result = adapter.execute("load_url", {})
    assert result["ok"] is False
    assert "required" in result["error"]


def test_audit_log_records_invocations(adapter):
    """Each invocation appends one bounded audit entry."""
    adapter.execute("eval_js", {"script": "1+1"})
    adapter.execute("load_url", {"url": "https://example.com"})
    log = adapter.get_audit_log()
    assert [entry["tool"] for entry in log] == ["eval_js", "load_url"]
    assert log[0]["error"] is None
    assert log[0]["duration_ms"] >= 0


def test_audit_log_respects_limit(adapter):
    """get_audit_log honours its limit and rejects non-positive values."""
    for _ in range(5):
        adapter.execute("eval_js", {"script": "1"})
    assert len(adapter.get_audit_log(limit=2)) == 2
    assert adapter.get_audit_log(limit=0) == []


def test_execute_records_failures_in_audit_log(adapter):
    """Failures are audited too, so the agent can see what was attempted."""
    adapter.execute("nope", {})
    log = adapter.get_audit_log(limit=1)
    assert log[0]["tool"] == "nope"
    assert log[0]["error"]


def test_adapter_requires_a_webview():
    """Constructing without a WebView is a programming error."""
    if AuroraViewAdapter is None:
        pytest.skip("dcc-mcp-core not installed")
    with pytest.raises(ValueError):
        AuroraViewAdapter(None)


def test_fallback_when_core_lacks_js_roundtrip():
    """Without eval_js_future the adapter degrades to fire-and-forget."""
    if AuroraViewAdapter is None:
        pytest.skip("dcc-mcp-core not installed")
    view = _FakeWebView()

    class _PlainCore(object):
        """Core without the eval_js_future round-trip API."""

        def __init__(self):
            self.scripts = []

        def eval_js(self, script):
            self.scripts.append(script)

    view._core = _PlainCore()
    adapter = AuroraViewAdapter(view)
    result = adapter.execute("eval_js", {"script": "document.title"})
    assert result["ok"] is True
    assert result["result"] is None
