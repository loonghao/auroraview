# Launch embedded WebView2 inside Maya using AuroraView's Windows backend
# Language: Python 3.11 (Maya 2025+)
# No third-party deps; relies on auroraview .pyd only.

from __future__ import annotations

import json
import logging
from typing import Any, Callable, Dict

log = logging.getLogger("auroraview.examples.webview2")


def _get_maya_main_hwnd() -> int:
    try:
        import maya.OpenMayaUI as omui  # type: ignore
    except Exception as e:  # pragma: no cover
        raise RuntimeError("This script must be run inside Maya") from e
    ptr = int(omui.MQtUtil.mainWindow())
    if not ptr:
        raise RuntimeError("Failed to acquire Maya main window HWND")
    return ptr


_BRIDGE_JS = r"""
// AuroraView WebView2 bridge: map CustomEvent <-> chrome.webview.postMessage
(function(){
  if (window.__av_bridge_installed) return; window.__av_bridge_installed = true;
  const origDispatch = window.dispatchEvent;
  window.dispatchEvent = function(ev){
    try {
      if (ev && typeof CustomEvent !== 'undefined' && ev instanceof CustomEvent) {
        // Forward CustomEvent to host
        window.chrome?.webview?.postMessage({ type: 'event', event: ev.type, detail: ev.detail });
      }
    } catch (e) {}
    return origDispatch.apply(this, arguments);
  };

  // Host -> Page: {type:'emit', event, detail}
  try {
    window.chrome?.webview?.addEventListener('message', function(e){
      const msg = e && e.data; if (!msg || msg.type !== 'emit') return;
      const ce = new CustomEvent(msg.event, { detail: msg.detail });
      window.dispatchEvent(ce);
    });
  } catch (e) {}
})();
"""


def launch(url: str = "http://localhost:5173", width: int = 800, height: int = 600) -> int:
    """Create an embedded WebView2 under Maya's main window and inject IPC bridge.

    Returns an integer handle you can pass to control functions.
    """
    parent = _get_maya_main_hwnd()

    try:
        from auroraview import _core as av_core  # type: ignore
    except Exception as e:
        raise RuntimeError("auroraview core module not found") from e

    # Ensure WebView2 API is available (feature-gated)
    required = [
        "win_webview2_create_embedded",
        "win_webview2_navigate",
        "win_webview2_eval",
        "win_webview2_post_message",
        "win_webview2_on_message",
    ]
    for name in required:
        if not hasattr(av_core, name):
            raise RuntimeError(
                "AuroraView built without 'win-webview2' support or missing symbol: %s" % name
            )

    handle = av_core.win_webview2_create_embedded(parent, 0, 0, width, height, url)
    log.info("WebView2 handle: %s", handle)

    # Inject JS bridge so the existing frontend (CustomEvent-based) works as-is
    av_core.win_webview2_eval(handle, _BRIDGE_JS)

    return handle


def navigate(handle: int, url: str) -> None:
    from auroraview import _core as av_core  # type: ignore

    av_core.win_webview2_navigate(handle, url)


def eval_js(handle: int, script: str) -> None:
    from auroraview import _core as av_core  # type: ignore

    av_core.win_webview2_eval(handle, script)


def on_message(handle: int, handler: Callable[[Dict[str, Any]], None]) -> None:
    """Register a Python callback to receive messages from the page.

    Receives dict decoded from JSON with shape {type, event, detail}.
    """
    from auroraview import _core as av_core  # type: ignore

    def _py_cb(json_text: str):
        try:
            data = json.loads(json_text) if json_text else {}
        except Exception:
            data = {"type": "raw", "payload": json_text}
        try:
            handler(data)
        except Exception as e:  # pragma: no cover
            log.exception("Error in on_message handler: %s", e)

    av_core.win_webview2_on_message(handle, _py_cb)


def dispose(handle: int) -> None:
    from auroraview import _core as av_core  # type: ignore

    av_core.win_webview2_dispose(handle)


if __name__ == "__main__":  # Manual test inside Maya's Script Editor (Python)
    h = launch("http://localhost:5173", 900, 600)
    log.info("Launched WebView2: handle=%s", h)
