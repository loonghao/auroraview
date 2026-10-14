# -*- coding: utf-8 -*-
"""Coexistence: AuroraView and a second DCC adapter share one FileRegistry.

An AuroraView panel is normally embedded *inside* a DCC (Maya, Houdini, ...).
That DCC usually runs its own dcc-mcp adapter, so two adapters write to the
same ``services.json`` at the same time. This suite proves they coexist.

It drives core's real ``DccServerBase`` for the second DCC rather than
installing ``dcc-mcp-maya``, because the latter only registers when a live
Maya session exists. The registration, locking, heartbeat and reaping path
under test is core's own, which is what actually matters here.

Assertions:
1. Both rows are present -- concurrent registration does not clobber.
2. Both heartbeats advance independently.
3. The two instances hold distinct ports.
4. Shutting one down does not deregister the other -- the failure mode that
   is easiest to miss and worst to ship.

Requires the optional ``dcc-mcp-core`` dependency; skipped when absent.
"""

from __future__ import annotations

import json
import os
import sys
import time

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
    from auroraview.dcc_mcp.adapter import AuroraViewAdapter
    from auroraview.dcc_mcp.server import start_server

SKILLS_DIR = os.path.join(
    os.path.dirname(__file__), "..", "..", "..", "skills", "auroraview-webview"
)

#: dcc_type used for the "other" adapter standing in for dcc-mcp-maya.
OTHER_DCC = "maya"


def _free_port() -> int:
    import socket

    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def _read_rows(registry_dir: str):
    path = os.path.join(registry_dir, "services.json")
    if not os.path.exists(path):
        return []
    with open(path, "r", encoding="utf-8") as handle:
        data = json.load(handle)
    return data if isinstance(data, list) else data.get("services", [])


def _row(registry_dir: str, dcc_type: str):
    for entry in _read_rows(registry_dir):
        if entry.get("dcc_type") == dcc_type:
            return entry
    return None


def _wait_row(registry_dir: str, dcc_type: str, timeout: float = 15.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        found = _row(registry_dir, dcc_type)
        if found is not None:
            return found
        time.sleep(0.2)
    return None


def _heartbeat_seconds(entry) -> float:
    value = entry.get("last_heartbeat")
    if isinstance(value, dict):
        return float(value.get("secs_since_epoch", 0)) + (
            float(value.get("nanos_since_epoch", 0)) / 1e9
        )
    return float(value)


class _FakeCore:
    """Minimal stand-in for the Rust core's JavaScript round-trip API."""

    def __init__(self):
        self.scripts = []

    def eval_js_future(self, script, timeout_ms):
        self.scripts.append((script, timeout_ms))
        return "cb-1"

    def get_js_result(self, callback_id):
        return {"status": "complete", "result": '"hello"'}


class _FakeWebView:
    """Minimal stand-in for an AuroraView WebView."""

    def __init__(self, title="AuroraView", url="http://localhost:3000/"):
        self._core = _FakeCore()
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


def _start_other_dcc(registry_dir: str, gateway_port: int):
    """Start a second, non-AuroraView DCC instance on the same registry.

    Uses core's own server so the second row is written through the real
    registration path, just with a different ``dcc_name``.
    """
    import tempfile
    from pathlib import Path

    from dcc_mcp_core.server_base import DccServerBase, DccServerOptions

    builtin = Path(tempfile.gettempdir()) / "auroraview-dcc-mcp" / "builtin-skills"
    builtin.mkdir(parents=True, exist_ok=True)

    options = DccServerOptions.from_env(
        OTHER_DCC,
        builtin_skills_dir=builtin,
        gateway_port=gateway_port,
        registry_dir=registry_dir,
        dcc_version="2024.2",
        dcc_pid=os.getpid(),
        adapter_version="0.9.26",
        enable_telemetry=False,
        enable_file_logging=False,
    )
    server = DccServerBase(options)
    server.start()
    return server


@pytest.fixture()
def registry_dir(tmp_path):
    path = str(tmp_path / "registry")
    os.makedirs(path, exist_ok=True)
    old = os.environ.get("DCC_MCP_REGISTRY_DIR")
    os.environ["DCC_MCP_REGISTRY_DIR"] = path
    try:
        yield path
    finally:
        if old is None:
            os.environ.pop("DCC_MCP_REGISTRY_DIR", None)
        else:
            os.environ["DCC_MCP_REGISTRY_DIR"] = old


@pytest.fixture()
def coexisting(registry_dir):
    """Two live instances on one registry: AuroraView plus a second DCC."""
    aurora = start_server(
        AuroraViewAdapter(_FakeWebView(), host_dcc=OTHER_DCC),
        gateway_port=_free_port(),
        registry_dir=registry_dir,
        skill_paths=[SKILLS_DIR],
        enable_telemetry=False,
    )
    aurora.start()
    other = _start_other_dcc(registry_dir, _free_port())
    servers = [aurora, other]
    try:
        yield aurora, other, registry_dir
    finally:
        for srv in servers:
            try:
                srv.stop()
            except Exception:  # noqa: BLE001 - teardown must not mask failures
                pass


def test_both_rows_present_after_concurrent_registration(coexisting):
    """Neither adapter clobbers the other's row in a shared registry."""
    _aurora, _other, registry_dir = coexisting
    aurora_row = _wait_row(registry_dir, "auroraview")
    other_row = _wait_row(registry_dir, OTHER_DCC)
    assert aurora_row is not None, "auroraview row missing when sharing a registry"
    assert other_row is not None, "maya row missing when sharing a registry"
    assert aurora_row["instance_id"] != other_row["instance_id"]
    assert aurora_row.get("adapter_dcc") == "auroraview"
    assert other_row.get("adapter_dcc") == OTHER_DCC


def test_heartbeats_advance_independently(coexisting):
    """Both rows keep beating while both instances are alive."""
    _aurora, _other, registry_dir = coexisting
    first_a = _wait_row(registry_dir, "auroraview")
    first_o = _wait_row(registry_dir, OTHER_DCC)
    assert first_a is not None and first_o is not None
    # The registry heartbeat period is ~5s, so wait past one full interval.
    time.sleep(7.0)
    second_a = _wait_row(registry_dir, "auroraview")
    second_o = _wait_row(registry_dir, OTHER_DCC)
    assert second_a is not None and second_o is not None
    assert _heartbeat_seconds(second_a) > _heartbeat_seconds(first_a), "auroraview heartbeat frozen"
    assert _heartbeat_seconds(second_o) > _heartbeat_seconds(first_o), "maya heartbeat frozen"


def test_instances_hold_distinct_ports(coexisting):
    """Two live instances never collide on their transport port."""
    _aurora, _other, registry_dir = coexisting
    aurora_row = _wait_row(registry_dir, "auroraview")
    other_row = _wait_row(registry_dir, OTHER_DCC)
    assert aurora_row is not None and other_row is not None
    ports = {aurora_row["port"], other_row["port"]}
    assert len(ports) == 2, "two instances ended up on the same port: %r" % ports
    # Guard against a degenerate {0, N} set passing the cardinality check.
    assert all(1 <= p <= 65535 for p in ports), "port outside valid range: %r" % sorted(ports)


def test_stopping_one_does_not_deregister_the_other(coexisting):
    """Shutting down one adapter must not reap the other's row.

    This is the coexistence failure that is easiest to miss: a shared-reaper
    or a full-file rewrite on shutdown would take the healthy instance with it.
    """
    aurora, _other, registry_dir = coexisting
    # Ensure both are registered before stopping anything.
    assert _wait_row(registry_dir, "auroraview") is not None
    assert _wait_row(registry_dir, OTHER_DCC) is not None

    aurora.stop()

    # The AuroraView row goes away...
    deadline = time.monotonic() + 15.0
    while time.monotonic() < deadline:
        if _row(registry_dir, "auroraview") is None:
            break
        time.sleep(0.2)
    assert _row(registry_dir, "auroraview") is None, "auroraview row survived its own stop()"

    # ...but the other DCC's row must still be there and still beating.
    survivor = _wait_row(registry_dir, OTHER_DCC, timeout=10.0)
    assert survivor is not None, (
        "stopping the auroraview instance deregistered the %s instance too" % OTHER_DCC
    )
    first = _heartbeat_seconds(survivor)
    # Past one full heartbeat interval.
    time.sleep(7.0)
    later = _wait_row(registry_dir, OTHER_DCC, timeout=10.0)
    assert later is not None, "survivor row vanished after its peer stopped"
    assert _heartbeat_seconds(later) > first, "survivor heartbeat stalled"


def test_host_dcc_is_published_when_embedded(registry_dir):
    """An embedded panel declares which DCC session it belongs to.

    Without this, a panel inside Maya registers as a bare auroraview row and a
    downstream agent cannot attribute it to the Maya session.
    """
    adapter = AuroraViewAdapter(_FakeWebView(), host_dcc=OTHER_DCC)
    assert adapter.get_context()["host_dcc"] == OTHER_DCC
