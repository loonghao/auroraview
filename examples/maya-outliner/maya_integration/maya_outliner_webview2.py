"""
Maya Outliner (WebView2 backend) — minimal but real interactive example.

- Embeds a WebView2 child window into a Qt dialog container
- Bridges CustomEvent <-> chrome.webview.postMessage so existing frontend works
- Implements two-way IPC for:
  * get_scene_hierarchy → scene_updated
  * select_node
  * set_visibility → scene_updated

Requirements:
- Windows + AuroraView built with --features win-webview2
- PySide2 available in Maya (standard)
- Vite dev server running (http://localhost:5173) or change URL
"""

from __future__ import annotations

import json
from typing import Any, Dict, List, Optional

# Maya imports (guarded for standalone)
try:
    import maya.api.OpenMaya as om  # type: ignore
    import maya.cmds as cmds  # type: ignore
    import maya.utils as mutils  # type: ignore

    MAYA_AVAILABLE = True
except Exception:
    MAYA_AVAILABLE = False

from PySide2 import QtCore, QtWidgets  # type: ignore

try:
    from auroraview import _core as av_core  # type: ignore
except Exception as e:  # pragma: no cover
    raise RuntimeError(
        "auroraview core module not found — build with --features win-webview2"
    ) from e


class MayaOutlinerWV2:
    def __init__(self, url: str = "http://localhost:5173", width: int = 420, height: int = 720):
        self.url = url
        self._handle: Optional[int] = None
        self._container = QtWidgets.QDialog(parent=QtWidgets.QApplication.activeWindow())
        self._container.setWindowTitle("AuroraView Outliner (WebView2)")
        self._container.setAttribute(QtCore.Qt.WA_NativeWindow, True)
        self._container.resize(width, height)
        self._container.show()  # must show before embedding
        # Runtime watcher state
        self._script_jobs = []  # Maya scriptJob ids
        self._om_callbacks = []  # OpenMaya callback ids
        self._pending_scene_push = False  # throttle scene_updated

        if not hasattr(av_core, "win_webview2_create_embedded"):
            raise RuntimeError(
                "AuroraView built without 'win-webview2' — rebuild with --features win-webview2"
            )

        # Create embedded WebView2
        hwnd = int(self._container.winId())
        self._handle = av_core.win_webview2_create_embedded(hwnd, 0, 0, width, height, self.url)

        # First-frame bounds sync after show
        QtCore.QTimer.singleShot(
            0,
            lambda: av_core.win_webview2_set_bounds(
                self._handle, 0, 0, self._container.width(), self._container.height()
            ),
        )

        # Auto-resize binding
        old_resize = self._container.resizeEvent

        def _sync(_: QtCore.QResizeEvent):
            av_core.win_webview2_set_bounds(
                self._handle, 0, 0, self._container.width(), self._container.height()
            )
            if old_resize:
                old_resize(_)

        self._container.resizeEvent = _sync  # type: ignore
        # Register Maya-side watchers for real-time updates
        if MAYA_AVAILABLE:
            self._register_maya_watchers()

        # Inject bridge and register Python message handler
        self._inject_bridge()
        self._register_handlers()

    # ----- Frontend<->Host Bridge -----
    def _inject_bridge(self):
        bridge_js = r"""
// AuroraView WebView2 bridge: map CustomEvent <-> chrome.webview.postMessage
(function(){
  if (window.__av_bridge_installed) return; window.__av_bridge_installed = true;
  const origDispatch = window.dispatchEvent;
  window.dispatchEvent = function(ev){
    try {
      if (ev && typeof CustomEvent !== 'undefined' && ev instanceof CustomEvent) {
        window.chrome?.webview?.postMessage({ type: 'event', event: ev.type, detail: ev.detail });
      }
    } catch (e) {}
    return origDispatch.apply(this, arguments);
  };
  try {
    window.chrome?.webview?.addEventListener('message', function(e){
      const msg = e && e.data; if (!msg || msg.type !== 'emit') return;
      const ce = new CustomEvent(msg.event, { detail: msg.detail });
      window.dispatchEvent(ce);
    });
  } catch (e) {}
})();
"""
        av_core.win_webview2_eval(self._handle, bridge_js)

    def _emit(self, event: str, detail: Any):
        av_core.win_webview2_post_message(
            self._handle, json.dumps({"type": "emit", "event": event, "detail": detail})
        )

    def _register_handlers(self):
        def on_msg(data_json: str):
            try:
                data = json.loads(data_json) if data_json else {}
            except Exception:
                data = {"type": "raw", "payload": data_json}

            if not isinstance(data, dict):
                return

            if data.get("type") != "event":
                return

            ev = data.get("event")
            detail = data.get("detail", {})

            if ev == "get_scene_hierarchy":
                self._emit("scene_updated", self.get_scene_hierarchy())
            elif ev == "select_node":
                node = detail.get("node_name")
                if node:
                    if MAYA_AVAILABLE:
                        mutils.executeDeferred(lambda: self.select_node(node))
                    else:
                        self.select_node(node)
            elif ev == "set_visibility":
                node = detail.get("node_name")
                vis = bool(detail.get("visible", True))
                if node:
                    if MAYA_AVAILABLE:
                        mutils.executeDeferred(lambda: self.set_visibility(node, vis))
                    else:
                        self.set_visibility(node, vis)

        av_core.win_webview2_on_message(self._handle, on_msg)

    # ----- Real-time watchers (Maya -> Frontend) -----
    def _schedule_scene_push(self, delay_ms: int = 80):
        """Throttle scene_updated pushes to avoid spamming on bulk changes."""
        if self._pending_scene_push:
            return
        self._pending_scene_push = True

        def _do():
            try:
                self._emit("scene_updated", self.get_scene_hierarchy())
            finally:
                self._pending_scene_push = False

        if MAYA_AVAILABLE:
            # Ensure run on main thread/UI-safe
            mutils.executeDeferred(lambda: QtCore.QTimer.singleShot(delay_ms, _do))
        else:
            QtCore.QTimer.singleShot(delay_ms, _do)

    def _on_maya_selection_changed(self):
        if not MAYA_AVAILABLE:
            return
        try:
            sel = cmds.ls(selection=True) or []
            payload = {"node": (sel[0] if sel else None), "selection": sel}
            self._emit("selection_changed", payload)
        except Exception as e:
            print("[WV2] selection watcher error:", e)

    def _register_maya_watchers(self):
        # Selection change
        try:
            jid = cmds.scriptJob(
                event=[
                    "SelectionChanged",
                    lambda: mutils.executeDeferred(self._on_maya_selection_changed),
                ],
                protected=True,
            )
            self._script_jobs.append(jid)
        except Exception as e:
            print("[WV2] scriptJob SelectionChanged failed:", e)

        # DAG object create/remove -> refresh hierarchy
        for ev_name in ("DagObjectCreated", "DagObjectRemoved"):
            try:
                jid2 = cmds.scriptJob(
                    event=[
                        ev_name,
                        lambda ev=ev_name: mutils.executeDeferred(
                            lambda: self._schedule_scene_push(120)
                        ),
                    ],
                    protected=True,
                )
                self._script_jobs.append(jid2)
            except Exception as e:
                print(f"[WV2] scriptJob {ev_name} failed:", e)

        # Scene open/new -> refresh hierarchy
        try:
            cb1 = om.MSceneMessage.addCallback(
                om.MSceneMessage.kAfterOpen, lambda *_: self._schedule_scene_push(120)
            )
            cb2 = om.MSceneMessage.addCallback(
                om.MSceneMessage.kAfterNew, lambda *_: self._schedule_scene_push(120)
            )
            self._om_callbacks.extend([cb1, cb2])
        except Exception as e:
            print("[WV2] OpenMaya scene callbacks failed:", e)

    def _teardown_watchers(self):
        if not MAYA_AVAILABLE:
            return
        # Kill scriptJobs
        try:
            for jid in list(getattr(self, "_script_jobs", [])):
                try:
                    if cmds.scriptJob(exists=jid):
                        cmds.scriptJob(kill=jid, force=True)
                except Exception:
                    pass
        finally:
            self._script_jobs = []
        # Remove OpenMaya callbacks
        try:
            for cb in list(getattr(self, "_om_callbacks", [])):
                try:
                    om.MMessage.removeCallback(cb)
                except Exception:
                    pass
        finally:
            self._om_callbacks = []

    # ----- Maya helpers -----
    def get_scene_hierarchy(self) -> List[Dict[str, Any]]:
        if not MAYA_AVAILABLE:
            return [
                {
                    "name": "pCube1",
                    "type": "transform",
                    "path": "|pCube1",
                    "parent": None,
                    "visible": True,
                    "selected": False,
                    "children": [
                        {
                            "name": "pCubeShape1",
                            "type": "mesh",
                            "path": "|pCube1|pCubeShape1",
                            "parent": "pCube1",
                            "visible": True,
                            "selected": False,
                            "children": [],
                        }
                    ],
                }
            ]

        def build(node: str, parent: Optional[str] = None) -> Dict[str, Any]:
            children = cmds.listRelatives(node, children=True, fullPath=False) or []
            visible = True
            try:
                visible = cmds.getAttr(f"{node}.visibility")
            except Exception:
                pass
            selected = node in (cmds.ls(selection=True) or [])
            return {
                "name": node,
                "type": cmds.nodeType(node),
                "path": cmds.ls(node, long=True)[0] if cmds.objExists(node) else node,
                "parent": parent,
                "children": [build(c, node) for c in children],
                "visible": bool(visible),
                "selected": bool(selected),
            }

        roots = cmds.ls(assemblies=True) or []
        return [build(n) for n in roots]

    def select_node(self, node: str):
        if not MAYA_AVAILABLE:
            print(f"[WV2] Mock select: {node}")
            # reflect selection to frontend in mock mode
            self._emit("selection_changed", {"node": node, "selection": [node]})
            return
        try:
            cmds.select(node, replace=True)
            # push selection event to frontend
            self._on_maya_selection_changed()
        except Exception as e:
            print("[WV2] select_node error:", e)

    def set_visibility(self, node: str, visible: bool):
        if not MAYA_AVAILABLE:
            print(f"[WV2] Mock visibility: {node} -> {visible}")
            # reflect change to frontend in mock mode
            self._emit("scene_updated", self.get_scene_hierarchy())
            return
        try:
            cmds.setAttr(f"{node}.visibility", bool(visible))
            # throttle and push a scene refresh
            self._schedule_scene_push(80)
        except Exception as e:
            print("[WV2] set_visibility error:", e)

    def close(self):
        # teardown watchers first
        try:
            if MAYA_AVAILABLE:
                self._teardown_watchers()
        except Exception:
            pass

        if self._handle is not None:
            try:
                av_core.win_webview2_dispose(self._handle)
            except Exception:
                pass
            self._handle = None
        if self._container is not None:
            try:
                self._container.close()
            except Exception:
                pass


def main(url: Optional[str] = None) -> MayaOutlinerWV2:
    url = url or "http://localhost:5173"
    outliner = MayaOutlinerWV2(url=url)
    return outliner
