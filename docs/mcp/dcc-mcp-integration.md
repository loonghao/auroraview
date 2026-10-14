# DCC-MCP Integration

Expose a running AuroraView window to the [DCC-MCP](https://github.com/loonghao/dcc-mcp-core)
ecosystem, so `dcc-mcp-cli` can discover and drive AuroraView tools.

## Why

AuroraView is a WebView host, not a DCC. It has no scene graph, timeline,
selection, undo stack, or render farm. `dcc-mcp-core` models exactly that shape
with its `WebViewAdapter` contract, whose documentation calls out
"AuroraView-style browser / tool panels" as the intended target.

The integration therefore reuses AuroraView's existing WebView API rather than
introducing a second execution bridge.

## Install

`dcc-mcp-core` is an optional dependency. AuroraView keeps its
"no mandatory third-party Python dependency" guarantee.

```bash
pip install "auroraview[dcc-mcp]"
```

## Usage

```python
from auroraview import create_webview
from auroraview.dcc_mcp import AuroraViewAdapter, start_server

webview = create_webview(title="My Tool", url="https://example.com")

adapter = AuroraViewAdapter(webview, host_dcc="maya")
server = start_server(adapter, skill_paths=["skills/auroraview-webview"])
server.start()
```

The instance then appears in the DCC-MCP registry:

```bash
dcc-mcp-cli list
```

## What registration requires

`start_server` rejects `gateway_port=0`. In `dcc-mcp-core`, a zero gateway
port is the explicit opt-out that disables FileRegistry self-registration —
the server would run, but the CLI could never see it.

Once registered, the row carries a PID, an OS-held sentinel lock, and a
heartbeat, so a crashed instance disappears from the registry instead of
lingering as a ghost entry.

## Tools

| Tool | Purpose |
|---|---|
| `eval_js` | Evaluate JavaScript and return the value. |
| `screenshot` | Capture the page via `window.auroraview.screenshot`. |
| `load_url` | Load a URL (`http://`, `https://`, `file://`). |
| `load_html` | Load raw HTML content. |

These mirror AuroraView's own MCP surface so agents see one consistent tool
set rather than two competing ones.

## Tool invocation

`start_server` registers the adapter in a process-wide registry, so skill scripts resolve the live adapter with no configuration:

```python
server = start_server(adapter)   # adapter becomes reachable to skill dispatch
server.start()
```

Every tool the skill advertises is executable through that path, which is what makes calling `eval_js` return a value rather than an error.

Note that `start_server` deliberately does **not** register an in-process dispatcher with core. Core's `HostExecutionBridge` accepts only a dispatcher exposing `is_host_thread`, `dispatch_callable`, or the `post`/`tick` queue API; anything else raises a `TypeError` that core converts into an error envelope, so calls would fail while still appearing successful. Leaving it unset makes core run the callable inline, which works. Host-thread dispatch for tool execution belongs on a `QueueDispatcher` (see `AuroraViewQtHost`).

## Coexisting with the host DCC's own adapter

An AuroraView panel is usually embedded inside a DCC that also runs its own
dcc-mcp adapter, so two adapters write to the same FileRegistry. This is
supported: the registry is keyed by `(dcc_type, instance_id)`, guarded by a
cross-process file lock, so the two rows do not collide.

What makes the relationship legible is `host_dcc`. It is **auto-detected**
from the environment (mirroring `DccType::detect()`), so a panel running
inside Maya registers as belonging to that Maya session with no extra work:

```python
adapter = AuroraViewAdapter(webview)        # -> host_dcc == "maya" when MAYA_LOCATION is set
adapter = AuroraViewAdapter(webview, host_dcc="blender")  # explicit override
```

Set `AURORAVIEW_HOST_DCC` to force a name, or to an empty string to declare
the window standalone. Recognised hosts: `maya`, `houdini`, `nuke`,
`blender`, `3dsmax`, `unreal`.

## Host-thread dispatch

AuroraView runs a Qt event loop. `AuroraViewQtHost` wires the DCC-MCP
dispatcher tick onto a `QTimer`, so tool invocations execute on the thread
that owns the WebView — never on an HTTP or Tokio worker thread.

```python
from auroraview.dcc_mcp import AuroraViewQtHost

host = AuroraViewQtHost(dispatcher)
host.start()
```

## Python 3.7

All code in this integration is Python 3.7 compatible, matching AuroraView's
`requires-python = ">=3.7"` floor. Skill scripts run under the host
interpreter, so they too must stay 3.7 compatible.

## Known limitations

- The Rust `ServiceEntry.extras` field (intended for WebView fields such as
  `cdp_port`, `url`, `window_title`, `host_dcc`) has no setter exposed to
  Python in `dcc-mcp-core` 0.20.x. Those values are published through the
  adapter's `get_context()` instead, which is the contract's intended surface.
- `screenshot` depends on the `window.auroraview.screenshot` helper that
  AuroraView injects into every page. Pages with a strict Content Security
  Policy may block its html2canvas dependency.
- AuroraView does not currently expose a Chrome DevTools Protocol port, so
  `cdp_port` is reported as `0` unless the host supplies one.
- Skills are scoped by DCC: `auroraview-webview` is tagged to `auroraview`
  and exposes only `eval_js`, `screenshot`, `load_url` and `load_html`. It
  never advertises DCC scene operations, so an agent looking for Maya scene
  tools is not routed to a WebView panel.
