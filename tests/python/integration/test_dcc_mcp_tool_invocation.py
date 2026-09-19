# -*- coding: utf-8 -*-
"""End-to-end tool invocation: the "drive" half of the acceptance criteria.

Discovery (a registry row exists) and semantics (a skill loads and declares
its tools) are both necessary but not sufficient. An agent that calls
``eval_js`` must get a real value back. This suite proves that by invoking
tools through the same dispatch path a skill script uses.

It is the regression guard for a failure mode that is otherwise invisible:
``load_skill`` and ``get_skill_info`` both stay green while every actual call
returns ``{"ok": false, "error": "No live AuroraView adapter found..."}``.

Requires the optional ``dcc-mcp-core`` dependency; skipped when absent.
"""

from __future__ import annotations

import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", "python"))

from auroraview.dcc_mcp import DCC_MCP_CORE_IMPORT_ERROR  # noqa: E402

pytestmark = [
    pytest.mark.integration,
    pytest.mark.skipif(
        DCC_MCP_CORE_IMPORT_ERROR is not None,
        reason="dcc-mcp-core not installed (pip install auroraview[dcc-mcp])",
    ),
]

if DCC_MCP_CORE_IMPORT_ERROR is None:
    from auroraview.dcc_mcp import adapter_registry
    from auroraview.dcc_mcp.adapter import AuroraViewAdapter
    from auroraview.dcc_mcp.server import start_server

SKILL_NAME = "auroraview-webview"
SCRIPTS_DIR = os.path.join(
    os.path.dirname(__file__), "..", "..", "..", "skills", "auroraview-webview", "scripts"
)
SKILLS_DIR = os.path.join(
    os.path.dirname(__file__), "..", "..", "..", "skills", "auroraview-webview"
)


def _free_port() -> int:
    import socket

    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


class _FakeCore:
    """Minimal stand-in for the Rust core's JavaScript round-trip API."""

    def __init__(self, result='"hello"'):
        self.result = result
        self.scripts = []

    def eval_js_future(self, script, timeout_ms):
        self.scripts.append((script, timeout_ms))
        return "cb-1"

    def get_js_result(self, callback_id):
        return {"status": "complete", "result": self.result}


class _FakeWebView:
    """Minimal stand-in for an AuroraView WebView."""

    def __init__(self, core=None):
        self._core = core if core is not None else _FakeCore()
        self.title = "AuroraView"
        self.current_url = "http://localhost:3000/"
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
def started(tmp_path):
    """A started server with its adapter reachable from skill dispatch."""
    registry = str(tmp_path / "registry")
    os.makedirs(registry, exist_ok=True)
    old = os.environ.get("DCC_MCP_REGISTRY_DIR")
    os.environ["DCC_MCP_REGISTRY_DIR"] = registry
    old_override = os.environ.pop("AURORAVIEW_ADAPTER", None)

    adapter_registry.clear()
    adapter = AuroraViewAdapter(_FakeWebView(), host_dcc="maya")
    server = start_server(
        adapter,
        gateway_port=_free_port(),
        registry_dir=registry,
        skill_paths=[SKILLS_DIR],
        enable_telemetry=False,
    )
    server.start()
    try:
        yield adapter, server
    finally:
        try:
            server.stop()
        except Exception:  # noqa: BLE001 - teardown must not mask failures
            pass
        adapter_registry.clear()
        if old is None:
            os.environ.pop("DCC_MCP_REGISTRY_DIR", None)
        else:
            os.environ["DCC_MCP_REGISTRY_DIR"] = old
        if old_override is not None:
            os.environ["AURORAVIEW_ADAPTER"] = old_override


def test_start_server_makes_the_adapter_discoverable(started):
    """start_server registers the adapter, which is what dispatch reads."""
    adapter, _server = started
    assert adapter_registry.current() is adapter


def test_skill_dispatch_reaches_the_adapter(started):
    """A skill script resolves the live adapter and executes a tool.

    Runs the dispatcher that every script in the skill package calls, so this
    covers the path an agent actually triggers rather than only the adapter
    method. Skill scripts execute in-process under the host, which is the
    documented execution model.
    """
    adapter, _server = started
    sys.path.insert(0, SCRIPTS_DIR)
    try:
        import _dispatch

        result = _dispatch.run_tool("eval_js", {"script": "document.title"})
    finally:
        if SCRIPTS_DIR in sys.path:
            sys.path.remove(SCRIPTS_DIR)

    assert result["ok"] is True, result
    assert result["result"] == '"hello"'
    assert adapter._view._core.scripts, "eval_js was never dispatched to the WebView"


def test_skill_dispatch_reports_errors_instead_of_raising(started):
    """A failed tool call yields a structured error, not a traceback."""
    _adapter, _server = started
    sys.path.insert(0, SCRIPTS_DIR)
    try:
        import _dispatch

        result = _dispatch.run_tool("load_url", {})
    finally:
        if SCRIPTS_DIR in sys.path:
            sys.path.remove(SCRIPTS_DIR)

    assert result["ok"] is False
    assert "required" in result["error"]


def test_tool_invocation_returns_a_value(started):
    """adapter.execute returns the evaluated value, not an error."""
    adapter, _server = started
    result = adapter.execute("eval_js", {"script": "document.title"})
    assert result["ok"] is True, result
    assert result["result"] == '"hello"'


def test_tool_invocation_reaches_the_webview(started):
    """load_url and load_html actually mutate the WebView."""
    adapter, _server = started
    assert adapter.execute("load_url", {"url": "https://example.com"})["ok"] is True
    assert adapter._view.loaded_urls == ["https://example.com"]
    assert adapter.execute("load_html", {"html": "<h1>hi</h1>"})["ok"] is True
    assert adapter._view.loaded_html == ["<h1>hi</h1>"]


def test_loaded_skill_tools_are_invocable(started):
    """Every tool the skill advertises can actually be executed.

    Guards against a skill declaring tools the dispatch path cannot reach.
    """
    adapter, server = started
    server.load_skill(SKILL_NAME)
    info = server.get_skill_info(SKILL_NAME)
    tools = {t.get("name") if isinstance(t, dict) else t for t in info.get("tools", [])}
    assert tools, "skill declared no tools"

    params = {
        "eval_js": {"script": "1+1"},
        "screenshot": {},
        "load_url": {"url": "https://example.com"},
        "load_html": {"html": "<p>x</p>"},
    }
    for name in sorted(tools):
        result = adapter.execute(name, params.get(name, {}))
        assert result["ok"] is True, "tool %r failed: %r" % (name, result)
