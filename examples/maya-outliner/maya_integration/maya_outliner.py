"""
Maya Outliner Example for AuroraView

This module demonstrates how to integrate AuroraView with Maya to create
a modern web-based outliner interface.

IMPORTANT: This module is in 'maya_integration' package to avoid namespace
conflicts with Maya's core 'maya' package.
"""

from typing import Any, Dict, List, Optional

try:
    import maya.api.OpenMaya as om
    import maya.cmds as cmds
    import maya.utils as mutils

    MAYA_AVAILABLE = True
except ImportError:
    MAYA_AVAILABLE = False
    print("[MayaOutliner] Warning: Maya not available, using mock data")

# Try Qt backend first (recommended for Maya)
USE_QT_BACKEND = False  # Default to native backend
QtWebView = None
WebView = None
EventTimer = None
omui = None
wrapInstance = None
QWidget = None

try:
    import maya.OpenMayaUI as omui
    from PySide2.QtWidgets import QWidget
    from shiboken2 import wrapInstance

    from auroraview import EventTimer, QtWebView
    from auroraview import WebView as NativeWebView

    # Both backends available, prefer Qt in Maya
    WebView = NativeWebView  # Keep reference to native backend
    USE_QT_BACKEND = True
    print("[MayaOutliner] Qt backend available (will be used)")
    print("[MayaOutliner] Native backend also available (fallback)")
except ImportError as e:
    print(f"[MayaOutliner] Qt backend not available: {e}")
    # Try native backend only
    try:
        from auroraview import EventTimer, WebView

        USE_QT_BACKEND = False
        print("[MayaOutliner] Using Native backend only")
    except ImportError as e2:
        print(f"[MayaOutliner] ERROR: Failed to import auroraview: {e2}")
        print("[MayaOutliner] Make sure auroraview is installed and PYTHONPATH is set correctly")
        raise


class MayaOutliner:
    """Maya Outliner with AuroraView integration

    Supports singleton mode to ensure only one instance exists at a time.
    """

    # Class-level singleton registry
    _instances: Dict[str, "MayaOutliner"] = {}
    _singleton_lock = None  # Will be initialized when needed

    def __init__(self, singleton_key: Optional[str] = None):
        """Initialize Maya Outliner

        Args:
            singleton_key: If provided, enables singleton mode with this key.
                          Only one instance per key can exist at a time.
        """
        self.webview: Optional[Any] = None  # WebView or QtWebView
        self.callback_ids: List[Any] = []
        self._is_embedded = False  # Track if using embedded mode
        self._event_timer: Optional[Any] = None  # EventTimer for automatic event processing
        self._singleton_key = singleton_key
        self._is_closing = False  # Prevent re-entrant close calls

    def get_node_type(self, node: str) -> str:
        """Get the type of a Maya node"""
        if not MAYA_AVAILABLE:
            return "transform"

        node_type = cmds.nodeType(node)

        # Map Maya node types to simplified types
        type_mapping = {
            "mesh": "mesh",
            "camera": "camera",
            "light": "light",
            "pointLight": "light",
            "directionalLight": "light",
            "spotLight": "light",
            "joint": "joint",
            "locator": "locator",
            "transform": "transform",
        }

        return type_mapping.get(node_type, "unknown")

    def get_scene_hierarchy(self) -> List[Dict[str, Any]]:
        """Get the complete scene hierarchy"""
        if not MAYA_AVAILABLE:
            return self._get_mock_hierarchy()

        def build_node_tree(node: str, parent: Optional[str] = None) -> Dict[str, Any]:
            """Recursively build node tree"""
            children_names = cmds.listRelatives(node, children=True, fullPath=False) or []
            children = [build_node_tree(child, node) for child in children_names]

            # Get visibility
            visible = True
            try:
                visible = cmds.getAttr(f"{node}.visibility")
            except Exception:
                pass

            # Check if selected
            selected = node in (cmds.ls(selection=True) or [])

            return {
                "name": node,
                "type": self.get_node_type(node),
                "path": cmds.ls(node, long=True)[0] if cmds.objExists(node) else node,
                "parent": parent,
                "children": children,
                "visible": visible,
                "selected": selected,
            }

        # Get all root nodes (nodes without parents)
        all_nodes = cmds.ls(assemblies=True) or []
        return [build_node_tree(node) for node in all_nodes]

    def _get_mock_hierarchy(self) -> List[Dict[str, Any]]:
        """Get mock hierarchy for testing without Maya"""
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
            },
            {
                "name": "pSphere1",
                "type": "transform",
                "path": "|pSphere1",
                "parent": None,
                "visible": True,
                "selected": True,
                "children": [
                    {
                        "name": "pSphereShape1",
                        "type": "mesh",
                        "path": "|pSphere1|pSphereShape1",
                        "parent": "pSphere1",
                        "visible": True,
                        "selected": False,
                        "children": [],
                    }
                ],
            },
            {
                "name": "persp",
                "type": "camera",
                "path": "|persp",
                "parent": None,
                "visible": True,
                "selected": False,
                "children": [],
            },
        ]

    def select_node(self, node_name: str):
        """Select a node in Maya"""
        if not MAYA_AVAILABLE:
            print(f"[MayaOutliner] Mock: Select node '{node_name}'")
            return

        try:
            cmds.select(node_name, replace=True)
            print(f"[MayaOutliner] Selected: {node_name}")
        except Exception as e:
            print(f"[MayaOutliner] Error selecting node: {e}")

    def set_visibility(self, node_name: str, visible: bool):
        """Set node visibility"""
        if not MAYA_AVAILABLE:
            print(f"[MayaOutliner] Mock: Set '{node_name}' visibility to {visible}")
            return

        try:
            cmds.setAttr(f"{node_name}.visibility", visible)
            print(f"[MayaOutliner] Set '{node_name}' visibility to {visible}")

            # Notify frontend
            if self.webview:
                self.send_scene_update()
        except Exception as e:
            print(f"[MayaOutliner] Error setting visibility: {e}")

    def send_scene_update(self):
        """Send scene update to frontend"""
        if not self.webview:
            return

        hierarchy = self.get_scene_hierarchy()
        self.webview.emit("scene_updated", hierarchy)

    def send_selection_changed(self):
        """Send selection change to frontend"""
        if not self.webview or not MAYA_AVAILABLE:
            return

        selected = cmds.ls(selection=True)
        if selected:
            self.webview.emit("selection_changed", {"node": selected[0]})

    def setup_maya_callbacks(self):
        """Setup Maya scene callbacks"""
        if not MAYA_AVAILABLE:
            print("[MayaOutliner] Skipping callbacks (Maya not available)")
            return

        # Selection changed callback
        def on_selection_changed(*args):
            self.send_selection_changed()

        # Scene changed callback
        def on_scene_changed(*args):
            self.send_scene_update()

        try:
            # Register callbacks
            sel_callback = om.MEventMessage.addEventCallback(
                "SelectionChanged", on_selection_changed
            )
            scene_callback = om.MEventMessage.addEventCallback("SceneOpened", on_scene_changed)

            self.callback_ids.extend([sel_callback, scene_callback])
            print("[MayaOutliner] Maya callbacks registered")
        except Exception as e:
            print(f"[MayaOutliner] Error registering callbacks: {e}")

    def cleanup_callbacks(self):
        """Remove Maya callbacks"""
        if not MAYA_AVAILABLE:
            return

        for callback_id in self.callback_ids:
            try:
                om.MMessage.removeCallback(callback_id)
            except Exception:
                pass

        self.callback_ids.clear()
        print("[MayaOutliner] Maya callbacks removed")

    def _start_event_processing(self):
        """Start automatic event processing using EventTimer.

        This replaces the old scriptJob + QTimer approach with a unified EventTimer
        that handles all event processing automatically.
        """
        if self.webview is None or EventTimer is None:
            print(
                "[MayaOutliner] Cannot start event processing (webview or EventTimer not available)"
            )
            return

        try:
            # Create EventTimer with 16ms interval (60 FPS)
            self._event_timer = EventTimer(self.webview, interval_ms=16, check_window_validity=True)

            # Lightweight debug throttle
            _counter = {"n": 0}

            @self._event_timer.on_tick
            def handle_tick():
                _counter["n"] += 1
                if _counter["n"] <= 5 or (_counter["n"] % 60 == 0):
                    # Print first few ticks, then every ~60 ticks (~1s @60FPS)
                    print(f"[MayaOutliner] EventTimer tick #{_counter['n']}")

            @self._event_timer.on_close
            def handle_close():
                print(
                    "[MayaOutliner] Close signal detected from EventTimer -> invoking self.close()"
                )
                # Ensure full cleanup path executes (handles HWND force-close, registry, etc.)
                try:
                    self.close()
                except Exception as e:
                    print(f"[MayaOutliner] Error during close from EventTimer: {e}")

            # Start the timer
            self._event_timer.start()
            print("[MayaOutliner] EventTimer started (interval=16ms, ~60 FPS)")

        except Exception as e:
            print(f"[MayaOutliner] Failed to start EventTimer: {e}")
            import traceback

            traceback.print_exc()

    def _stop_event_processing(self):
        """Stop automatic event processing (EventTimer) and cleanup resources.

        Uses EventTimer.cleanup() to properly clear all references including
        the webview reference to prevent circular references.
        """
        if self._event_timer is not None:
            try:
                # Use cleanup() instead of stop() to clear webview reference
                self._event_timer.cleanup()
                print("[MayaOutliner] EventTimer stopped and cleaned up")
                self._event_timer = None
            except Exception as e:
                print(f"[MayaOutliner] Failed to stop EventTimer: {e}")

    @classmethod
    def _get_or_create_singleton(cls, singleton_key: str, factory_fn) -> "MayaOutliner":
        """Get existing singleton instance or create new one

        Args:
            singleton_key: Unique key for this singleton instance
            factory_fn: Function to create new instance if needed

        Returns:
            MayaOutliner instance (existing or newly created)
        """
        # Check if instance already exists
        if singleton_key in cls._instances:
            existing = cls._instances[singleton_key]
            print(f"[MayaOutliner] Singleton '{singleton_key}' already exists")

            # Check if it's still valid (webview not closed)
            if existing.webview is not None:
                print("[MayaOutliner] Returning existing singleton instance")
                return existing
            else:
                print("[MayaOutliner] Existing singleton is closed, creating new one")
                # Remove dead instance
                del cls._instances[singleton_key]

        # Create new instance
        print(f"[MayaOutliner] Creating new singleton instance: '{singleton_key}'")
        instance = factory_fn()
        instance._singleton_key = singleton_key
        cls._instances[singleton_key] = instance
        return instance

    def _remove_from_registry(self):
        """Remove this instance from singleton registry"""
        if self._singleton_key and self._singleton_key in self._instances:
            del self._instances[self._singleton_key]
            print(f"[MayaOutliner] Removed from singleton registry: '{self._singleton_key}'")

    def run(self, url: Optional[str] = None, use_local: bool = False):
        """Run the Maya Outliner WebView

        Args:
            url: URL to load. If None, auto-detect based on use_local flag
            use_local: If True, use local built files. If False, use dev server (default: False)
        """
        global USE_QT_BACKEND

        # Auto-detect URL if not provided
        if url is None:
            if use_local:
                # Use local built files
                import os

                dist_dir = os.path.join(os.path.dirname(__file__), "..", "dist")
                index_html = os.path.join(dist_dir, "index.html")
                if os.path.exists(index_html):
                    url = f"file:///{os.path.abspath(index_html).replace(os.sep, '/')}"
                    print(f"[MayaOutliner] Using local build: {url}")
                else:
                    print(f"[MayaOutliner] Warning: Local build not found at {dist_dir}")
                    print("[MayaOutliner] Run 'npm run build' to create local build")
                    print("[MayaOutliner] Falling back to dev server")
                    url = "http://localhost:5173"
            else:
                # Use dev server
                url = "http://localhost:5173"
                print(f"[MayaOutliner] Using dev server: {url}")

        print("[MayaOutliner] Creating WebView...")
        print(f"[MayaOutliner] Backend: {'Qt' if USE_QT_BACKEND else 'Native'}")

        # Create WebView based on available backend
        if USE_QT_BACKEND:
            # Qt backend - integrates as a Qt widget
            if QtWebView is None or omui is None or wrapInstance is None or QWidget is None:
                print("[MayaOutliner] Qt backend components not available, falling back to Native")
                USE_QT_BACKEND = False
            else:
                try:
                    # Get Maya main window
                    main_window_ptr = omui.MQtUtil.mainWindow()
                    maya_window = wrapInstance(int(main_window_ptr), QWidget)

                    self.webview = QtWebView(
                        parent=maya_window,
                        title="Maya Outliner",
                        width=400,
                        height=800,
                    )
                    self.webview.load_url(url)
                    print("[MayaOutliner] Qt WebView created")
                except Exception as e:
                    print(f"[MayaOutliner] Error creating Qt WebView: {e}")
                    print("[MayaOutliner] Falling back to Native backend")
                    USE_QT_BACKEND = False

        if not USE_QT_BACKEND:
            # Native backend - embedded in Maya (non-blocking)
            if WebView is None:
                raise RuntimeError(
                    "Native WebView backend is not available. "
                    "Make sure auroraview is properly installed."
                )

            # Get Maya main window handle for embedding
            maya_hwnd = None
            try:
                if omui is not None and wrapInstance is not None and QWidget is not None:
                    # Get Maya main window pointer
                    main_window_ptr = omui.MQtUtil.mainWindow()
                    if main_window_ptr:
                        # Wrap the pointer to get the QWidget
                        maya_window = wrapInstance(int(main_window_ptr), QWidget)

                        # Get the native window handle (HWND on Windows)
                        maya_hwnd = int(maya_window.winId())
                        print(f"[MayaOutliner] Maya window handle (HWND): {maya_hwnd}")
                        print(f"[MayaOutliner] Maya window handle (hex): 0x{maya_hwnd:X}")
                    else:
                        print("[MayaOutliner] Warning: Could not get Maya main window pointer")
                else:
                    print("[MayaOutliner] Warning: Maya not available, using standalone mode")
            except Exception as e:
                print(f"[MayaOutliner] Warning: Failed to get Maya window handle: {e}")
                import traceback

                traceback.print_exc()
                maya_hwnd = None

            self.webview = WebView(
                title="Maya Outliner",
                width=400,
                height=800,
                url=url,
                debug=True,  # Enable developer tools
                parent=maya_hwnd,
                mode="owner" if maya_hwnd else None,  # Use owner mode for cross-thread safety
            )
            print(f"[MayaOutliner] Native WebView created (embedded: {maya_hwnd is not None})")

        print("[MayaOutliner] Registering IPC handlers...")

        # Register IPC handlers
        @self.webview.on("get_scene_hierarchy")
        def handle_get_hierarchy(data):
            print("[MayaOutliner] Received: get_scene_hierarchy")
            hierarchy = self.get_scene_hierarchy()
            print(f"[MayaOutliner] Sending {len(hierarchy)} root nodes")
            self.webview.emit("scene_updated", hierarchy)

        @self.webview.on("select_node")
        def handle_select_node(data):
            print(f"[MayaOutliner] Received: select_node - {data}")
            node_name = data.get("node_name")
            if node_name:
                # Use executeDeferred to avoid Maya freezing
                if MAYA_AVAILABLE:
                    mutils.executeDeferred(lambda: self.select_node(node_name))
                else:
                    self.select_node(node_name)

        @self.webview.on("set_visibility")
        def handle_set_visibility(data):
            print(f"[MayaOutliner] Received: set_visibility - {data}")
            node_name = data.get("node_name")
            visible = data.get("visible", True)
            if node_name is not None:
                # Use executeDeferred to avoid Maya freezing
                if MAYA_AVAILABLE:
                    mutils.executeDeferred(lambda: self.set_visibility(node_name, visible))
                else:
                    self.set_visibility(node_name, visible)

        # Setup Maya callbacks
        self.setup_maya_callbacks()

        # Show WebView
        print("[MayaOutliner] Starting WebView...")
        print(f"[MayaOutliner] URL: {url}")

        if USE_QT_BACKEND:
            # Qt backend - show as Qt widget (non-blocking)
            self.webview.show()
            print("[MayaOutliner] Qt WebView shown (non-blocking)")
        else:
            # Native backend - use different methods based on embedding
            if self.webview._parent is not None:
                # Embedded mode - use core.show() directly (non-blocking)
                print("[MayaOutliner] Using embedded mode (owner)")
                self._is_embedded = True
                self.webview._core.show()
                print("[MayaOutliner] Native WebView shown (embedded, non-blocking)")
                # Start event processing for embedded native backend to handle WM_CLOSE, etc.
                self._start_event_processing()

            else:
                # Standalone mode - use show() which runs in background thread
                print("[MayaOutliner] Using standalone mode")
                self._is_embedded = False
                self.webview.show()
                print("[MayaOutliner] Native WebView shown (standalone, background thread)")

        print("[MayaOutliner] Maya Outliner is running!")
        print("[MayaOutliner] Use outliner.close() to close the window")

    def close(self):
        """Close the WebView window and cleanup

        This method is safe to call multiple times and handles all cleanup:
        - Stops EventTimer
        - Removes Maya callbacks
        - Closes WebView window
        - Removes from singleton registry
        """
        # Prevent re-entrant calls
        if self._is_closing:
            print("[MayaOutliner] Already closing, skipping...")
            return

        if self.webview is None:
            print("[MayaOutliner] No WebView to close")
            # Still remove from singleton registry
            self._remove_from_registry()
            return

        print("[MayaOutliner] Closing WebView...")
        self._is_closing = True

        try:
            # Step 1: Stop event processing timer first to avoid re-entrancy during close
            print("[MayaOutliner] Step 1: Stopping EventTimer...")
            self._stop_event_processing()

            # Step 2: Remove Maya callbacks
            print("[MayaOutliner] Step 2: Removing Maya callbacks...")
            self.cleanup_callbacks()

            # Step 3: Close the WebView
            print("[MayaOutliner] Step 3: Closing WebView window...")
            close_success = False

            if USE_QT_BACKEND:
                # Qt backend - close the widget
                if hasattr(self.webview, "close"):
                    self.webview.close()
                    print("[MayaOutliner] Qt WebView closed")
                    close_success = True
            else:
                # Native backend - always use the wrapper's close method
                # The wrapper handles both embedded and standalone modes correctly
                if hasattr(self.webview, "close"):
                    try:
                        self.webview.close()
                        print("[MayaOutliner] Native WebView close() invoked")
                        # Verify if the window actually closed (embedded mode needs this)
                        close_success = False
                        try:
                            core = getattr(self.webview, "_core", None)
                            if core is not None and hasattr(core, "is_window_valid"):
                                valid = core.is_window_valid()
                                print(f"[MayaOutliner] is_window_valid -> {valid}")
                                close_success = not valid
                            elif hasattr(self.webview, "is_alive"):
                                close_success = not self.webview.is_alive()
                        except Exception:
                            # If verification fails, fall back to force-close path below
                            close_success = False
                    except Exception as e:
                        print(f"[MayaOutliner] Error calling WebView.close(): {e}")
                        traceback.print_exc()
                else:
                    print("[MayaOutliner] Warning: WebView has no close method")

            # If normal close failed, try force close using HWND
            if not close_success:
                print("[MayaOutliner] Attempting force close using HWND...")
                try:
                    from auroraview import close_window_by_hwnd, destroy_window_by_hwnd

                    # Get HWND from webview (support wrapper or core object)
                    hwnd = None
                    try:
                        if hasattr(self.webview, "get_hwnd"):
                            hwnd = self.webview.get_hwnd()
                        elif hasattr(self.webview, "_core") and hasattr(
                            self.webview._core, "get_hwnd"
                        ):
                            hwnd = self.webview._core.get_hwnd()
                    except Exception:
                        hwnd = None

                    if hwnd:
                        print(f"[MayaOutliner] Got HWND: 0x{hwnd:x}")

                        # Try WM_CLOSE first (graceful)
                        print("[MayaOutliner] Trying WM_CLOSE...")
                        if close_window_by_hwnd(hwnd):
                            print("[MayaOutliner] WM_CLOSE sent successfully")
                            # Pump a few message cycles so WM_CLOSE is processed even after stopping the timer
                            import time

                            for _ in range(10):
                                if hasattr(self.webview, "_core") and hasattr(
                                    self.webview._core, "process_events"
                                ):
                                    try:
                                        self.webview._core.process_events()
                                    except Exception:
                                        pass
                                time.sleep(0.01)

                        # If still not closed, use DestroyWindow (aggressive)
                        print("[MayaOutliner] Trying DestroyWindow...")
                        if destroy_window_by_hwnd(hwnd):
                            print("[MayaOutliner] ✅ Window destroyed successfully")
                        else:
                            print("[MayaOutliner] ❌ DestroyWindow failed")
                    else:
                        print(
                            "[MayaOutliner] Could not get HWND from webview (no get_hwnd on wrapper/core)"
                        )
                except Exception as e:
                    print(f"[MayaOutliner] Error during force close: {e}")
                    traceback.print_exc()

            # Step 4: Clear reference
            self.webview = None

            # Step 5: Remove from singleton registry
            self._remove_from_registry()

            print("[MayaOutliner] WebView cleanup complete")

        except Exception as e:
            print(f"[MayaOutliner] Error closing WebView: {e}")
            import traceback

            traceback.print_exc()
        finally:
            self._is_closing = False


def main(url: Optional[str] = None, use_local: bool = False, singleton: bool = True):
    """Main entry point with singleton support

    Args:
        url: URL to load. If None, auto-detect based on use_local flag
        use_local: If True, use local built files. If False, use dev server (default: False)
        singleton: If True, only allow one instance at a time (default: True)

    Usage in Maya:
        >>> from maya_integration import maya_outliner
        >>>
        >>> # Use dev server with singleton mode (default)
        >>> outliner = maya_outliner.main()
        >>>
        >>> # Calling again returns the same instance
        >>> outliner2 = maya_outliner.main()  # Returns existing instance
        >>> assert outliner is outliner2
        >>>
        >>> # Use local build
        >>> outliner = maya_outliner.main(use_local=True)
        >>>
        >>> # Allow multiple instances
        >>> outliner1 = maya_outliner.main(singleton=False)
        >>> outliner2 = maya_outliner.main(singleton=False)  # Creates new instance
        >>>
        >>> # Close the window
        >>> outliner.close()
    """
    print("=" * 60)
    print("Maya Outliner - AuroraView Example")
    print("=" * 60)
    print()

    if not MAYA_AVAILABLE:
        print("Warning: Running without Maya (using mock data)")
        print()

    if singleton:
        # Singleton mode - return existing instance or create new one
        def create_instance():
            outliner = MayaOutliner(singleton_key="maya_outliner_default")
            outliner.run(url=url, use_local=use_local)
            return outliner

        outliner = MayaOutliner._get_or_create_singleton("maya_outliner_default", create_instance)
    else:
        # Multi-instance mode - always create new instance
        outliner = MayaOutliner()
        outliner.run(url=url, use_local=use_local)

    print()
    print("=" * 60)
    print("Maya Outliner started successfully!")
    print("=" * 60)
    print()
    print("Tips:")
    print("- Click nodes to select them in Maya")
    print("- Toggle visibility with the eye icon")
    print("- Use search to filter nodes")
    print("- Press F12 to open DevTools")
    print("- Use outliner.close() to close the window")
    if singleton:
        print("- Singleton mode: Only one instance allowed at a time")
    else:
        print("- Multi-instance mode: Multiple windows can coexist")
    print()

    return outliner


if __name__ == "__main__":
    main()
