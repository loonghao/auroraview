# -*- coding: utf-8 -*-
"""
Maya Scene Outliner - Real-time Scene Hierarchy Viewer

This example demonstrates:
1. Real-time scene hierarchy display
2. Right-click context menu (rename, delete)
3. Bidirectional communication (Python ↔ JavaScript)
4. Event-driven updates

CRITICAL THREADING PATTERN FOR MAYA:
- WebView MUST be created in Maya's main thread (not in background thread)
- Use scriptJob to periodically call process_events() for message processing
- All Maya commands MUST be queued with maya.utils.executeDeferred()
- DO NOT use show_async() - it creates the window in a background thread which causes freezing

CORRECT PATTERN:
1. Create WebView in main thread (this script runs in main thread)
2. Load HTML content
3. Register event handlers
4. Create scriptJob to call process_events() periodically
5. Window will be responsive and Maya won't freeze
"""

import sys
sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\python')

import json
import traceback
import maya.cmds as cmds
import maya.OpenMayaUI as omui
from auroraview import NativeWebView
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QWidget

print("=" * 70)
print("Maya Scene Outliner - FIXED VERSION")
print("=" * 70)
print("")
print("THREADING MODEL:")
print("[OK] WebView created in Maya's main thread")
print("[OK] scriptJob handles event processing via process_events()")
print("[OK] Maya remains responsive while WebView is open")
print("=" * 70)
print("")

# Get Maya main window
print("[SEARCH] Getting Maya main window handle...")
main_window_ptr = omui.MQtUtil.mainWindow()
maya_window = wrapInstance(int(main_window_ptr), QWidget)
hwnd = maya_window.winId()
print(f"[OK] Maya window HWND: {hwnd}")
print("")

# Create WebView using new factory method (cleaner API)
print("Creating embedded WebView...")
print("   - Using NativeWebView.embedded() factory method")
print("   - Mode: Owner (cross-thread safe)")
print("   - Parent HWND:", hwnd)
print("   - Decorations: False (no title bar)")
webview = NativeWebView.embedded(
    parent_hwnd=hwnd,
    title="Maya Outliner",
    width=400,
    height=600,
    decorations=False,  # Remove title bar - use custom HTML controls
    mode="owner",  # Safer for cross-thread scenarios
)
print("[OK] WebView created successfully")
print("")

def get_scene_hierarchy():
    """Get Maya scene hierarchy as a tree structure"""
    print("[INFO] [get_scene_hierarchy] Starting...")

    def build_tree(node):
        """Recursively build tree structure"""
        try:
            children = cmds.listRelatives(node, children=True, fullPath=True) or []
            node_type = cmds.nodeType(node)
            short_name = node.split('|')[-1]

            # Filter children to only include transforms and shapes
            valid_children = []
            for child in children:
                try:
                    if cmds.objectType(child, isAType='transform') or cmds.objectType(child, isAType='shape'):
                        child_tree = build_tree(child)
                        # Only add non-None children to avoid rendering issues
                        if child_tree is not None:
                            valid_children.append(child_tree)
                except Exception as e:
                    print(f"[WARNING] [build_tree] Error processing child {child}: {e}")

            return {
                'name': short_name,
                'fullPath': node,
                'type': node_type,
                'children': valid_children
            }
        except Exception as e:
            print(f"[ERROR] [build_tree] Error building tree for {node}: {e}")
            traceback.print_exc()
            return None

    try:
        # Get all root transforms (objects without parents)
        root_nodes = cmds.ls(assemblies=True)
        print(f"[SEARCH] [get_scene_hierarchy] Found {len(root_nodes)} root nodes: {root_nodes}")

        hierarchy = []
        for node in root_nodes:
            tree = build_tree(node)
            # Only add non-None trees to avoid rendering issues
            if tree is not None:
                hierarchy.append(tree)

        print(f"[OK] [get_scene_hierarchy] Built hierarchy with {len(hierarchy)} root nodes")
        print(f"[SEARCH] [get_scene_hierarchy] Sample data: {json.dumps(hierarchy[:1], indent=2) if hierarchy else 'No data'}")
        return hierarchy
    except Exception as e:
        print(f"[ERROR] [get_scene_hierarchy] Error: {e}")
        traceback.print_exc()
        return []

def refresh_outliner():
    """Refresh the outliner view"""
    print("[REFRESH] [refresh_outliner] Called")

    def _do_refresh():
        try:
            print("[REFRESH] [refresh_outliner._do_refresh] Executing in Maya main thread...")

            # Get webview from __main__
            import __main__
            if not hasattr(__main__, 'maya_outliner'):
                print("[ERROR] [refresh_outliner._do_refresh] WebView not found in __main__.maya_outliner")
                return

            wv = __main__.maya_outliner
            print(f"[OK] [refresh_outliner._do_refresh] Got WebView: {wv}")

            hierarchy = get_scene_hierarchy()
            print(f"[REFRESH] [refresh_outliner._do_refresh] Got hierarchy: {len(hierarchy)} root nodes")

            # Validate hierarchy data before serialization
            if not hierarchy:
                print("[WARNING] [refresh_outliner._do_refresh] Empty hierarchy, skipping refresh")
                return

            # Serialize with error handling
            try:
                data_json = json.dumps({'hierarchy': hierarchy})
                print(f"[OK] [refresh_outliner._do_refresh] Hierarchy serialized successfully ({len(data_json)} bytes)")
            except (TypeError, ValueError) as e:
                print(f"[ERROR] [refresh_outliner._do_refresh] JSON serialization failed: {e}")
                print(f"[ERROR] [refresh_outliner._do_refresh] Hierarchy data: {hierarchy}")
                traceback.print_exc()
                return

            # Use emit() instead of eval_js() to avoid JSON injection and event loop issues
            print(f"[SEND] [refresh_outliner._do_refresh] Emitting scene_updated event via webview.emit()...")
            try:
                wv.emit('scene_updated', {'hierarchy': hierarchy})
                print(f"[OK] [refresh_outliner._do_refresh] Outliner refreshed ({len(hierarchy)} root nodes)")
            except Exception as e:
                print(f"[ERROR] [refresh_outliner._do_refresh] emit() failed: {e}")
                traceback.print_exc()

        except Exception as e:
            print(f"[ERROR] [refresh_outliner._do_refresh] Error: {e}")
            traceback.print_exc()

    import maya.utils as mutils
    print("[REFRESH] [refresh_outliner] Queueing to Maya main thread...")
    mutils.executeDeferred(_do_refresh)

# Event handlers
@webview.on("webview_ready")
def handle_webview_ready(data):
    """Handle WebView ready notification from JavaScript"""
    print(f"[RECV] [handle_webview_ready] WebView is ready: {data}")
    print("[REFRESH] [handle_webview_ready] Triggering initial refresh...")
    refresh_outliner()

@webview.on("refresh_scene")
def handle_refresh(data):
    """Handle refresh request from UI"""
    print(f"[RECV] [handle_refresh] Event received: {data}")
    refresh_outliner()

@webview.on("rename_object")
def handle_rename(data):
    """Handle rename request"""
    print(f"[EDIT] Rename request: {data}")

    def _do_rename():
        try:
            import __main__
            if not hasattr(__main__, 'maya_outliner'):
                print("[ERROR] [handle_rename] WebView not found")
                return
            wv = __main__.maya_outliner

            full_path = data.get('fullPath')
            new_name = data.get('newName', '').strip()

            if not full_path or not new_name:
                data_json = json.dumps({'ok': False, 'error': 'Invalid parameters'})
                wv._core.eval_js(f"if (window.__handlePythonEvent) {{ window.__handlePythonEvent('rename_result', {data_json}); }}")
                return

            # Check if object exists
            if not cmds.objExists(full_path):
                data_json = json.dumps({'ok': False, 'error': 'Object not found'})
                wv._core.eval_js(f"if (window.__handlePythonEvent) {{ window.__handlePythonEvent('rename_result', {data_json}); }}")
                return

            # Rename
            new_full_path = cmds.rename(full_path, new_name)
            print(f"[OK] Renamed: {full_path} → {new_full_path}")

            data_json = json.dumps({'ok': True, 'oldPath': full_path, 'newPath': new_full_path})
            wv._core.eval_js(f"if (window.__handlePythonEvent) {{ window.__handlePythonEvent('rename_result', {data_json}); }}")

            # Refresh outliner
            refresh_outliner()

        except Exception as e:
            print(f"[ERROR] Rename error: {e}")
            import __main__
            if hasattr(__main__, 'maya_outliner'):
                data_json = json.dumps({'ok': False, 'error': str(e)})
                __main__.maya_outliner._core.eval_js(f"if (window.__handlePythonEvent) {{ window.__handlePythonEvent('rename_result', {data_json}); }}")

    import maya.utils as mutils
    mutils.executeDeferred(_do_rename)

@webview.on("delete_object")
def handle_delete(data):
    """Handle delete request"""
    print(f"[DELETE] Delete request: {data}")

    def _do_delete():
        try:
            import __main__
            if not hasattr(__main__, 'maya_outliner'):
                print("[ERROR] [handle_delete] WebView not found")
                return
            wv = __main__.maya_outliner

            full_path = data.get('fullPath')

            if not full_path:
                data_json = json.dumps({'ok': False, 'error': 'Invalid parameters'})
                wv._core.eval_js(f"if (window.__handlePythonEvent) {{ window.__handlePythonEvent('delete_result', {data_json}); }}")
                return

            # Check if object exists
            if not cmds.objExists(full_path):
                data_json = json.dumps({'ok': False, 'error': 'Object not found'})
                wv._core.eval_js(f"if (window.__handlePythonEvent) {{ window.__handlePythonEvent('delete_result', {data_json}); }}")
                return

            # Delete
            cmds.delete(full_path)
            print(f"[OK] Deleted: {full_path}")

            data_json = json.dumps({'ok': True, 'path': full_path})
            wv._core.eval_js(f"if (window.__handlePythonEvent) {{ window.__handlePythonEvent('delete_result', {data_json}); }}")

            # Refresh outliner
            refresh_outliner()

        except Exception as e:
            print(f"[ERROR] Delete error: {e}")
            import __main__
            if hasattr(__main__, 'maya_outliner'):
                data_json = json.dumps({'ok': False, 'error': str(e)})
                __main__.maya_outliner._core.eval_js(f"if (window.__handlePythonEvent) {{ window.__handlePythonEvent('delete_result', {data_json}); }}")

    import maya.utils as mutils
    mutils.executeDeferred(_do_delete)

@webview.on("select_object")
def handle_select(data):
    """Handle selection request"""
    print(f"[CLICK] Select request: {data}")

    def _do_select():
        try:
            full_path = data.get('fullPath')

            if not full_path:
                return

            # Check if object exists
            if not cmds.objExists(full_path):
                return

            # Select
            cmds.select(full_path, replace=True)
            print(f"[OK] Selected: {full_path}")

        except Exception as e:
            print(f"[ERROR] Select error: {e}")

    import maya.utils as mutils
    mutils.executeDeferred(_do_select)

# Window dragging handler
@webview.on("move_window")
def handle_move_window(data):
    """Handle window move request from JavaScript (for custom title bar dragging)"""
    print(f"[RECV] [handle_move_window] Event received: {data}")
    print(f"[RECV] [handle_move_window] Data type: {type(data)}, Keys: {data.keys() if isinstance(data, dict) else 'N/A'}")

    # Validate data structure
    if not isinstance(data, dict):
        print(f"[ERROR] [handle_move_window] Invalid data type: {type(data)}, expected dict")
        return

    # Extract coordinates with validation
    x = data.get('x')
    y = data.get('y')

    print(f"[RECV] [handle_move_window] Extracted coordinates: x={x}, y={y}")

    # Validate coordinates
    if x is None or y is None:
        print(f"[ERROR] [handle_move_window] Missing coordinates: x={x}, y={y}")
        return

    # Validate types
    try:
        x = int(x)
        y = int(y)
    except (ValueError, TypeError) as e:
        print(f"[ERROR] [handle_move_window] Invalid coordinate types: x={x} ({type(x)}), y={y} ({type(y)}), error: {e}")
        return

    # Call the Rust backend to move the window
    try:
        print(f"[WINDOW] [handle_move_window] Moving window to ({x}, {y})")
        webview._core.set_window_position(x, y)
        print(f"[OK] [handle_move_window] Window moved successfully")
    except Exception as e:
        print(f"[ERROR] [handle_move_window] Failed to move window: {e}")
        traceback.print_exc()

# System control handlers
@webview.on("close_window")
def _handle_close(data):
    """Handle close request from JavaScript"""
    print("=" * 80)
    print("[LOCK] [_handle_close] Close requested from UI")
    print(f"[LOCK] [_handle_close] Event data: {data}")
    print("=" * 80)

    def _do_close():
        try:
            print("[LOCK] [_do_close] Attempting to close WebView...")
            print(f"[LOCK] [_do_close] WebView object: {webview}")
            print(f"[LOCK] [_do_close] WebView._core: {webview._core}")

            # Close the WebView window
            webview.close()
            print("[OK] [_do_close] WebView.close() called successfully")

            # Also try to kill the scriptJob
            import __main__
            if hasattr(__main__, 'maya_outliner_timer'):
                print(f"[LOCK] [_do_close] Killing scriptJob: {__main__.maya_outliner_timer}")
                cmds.scriptJob(kill=__main__.maya_outliner_timer)
                del __main__.maya_outliner_timer
                print("[OK] [_do_close] ScriptJob killed")

        except Exception as e:
            print(f"[ERROR] [_do_close] Close error: {e}")
            traceback.print_exc()

    import maya.utils as mutils
    print("[LOCK] [_handle_close] Queueing close operation to Maya main thread...")
    mutils.executeDeferred(_do_close)

# HTML UI
html = """
<!DOCTYPE html>
<html>
<head>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
            background: #2b2b2b; 
            color: #e0e0e0;
            height: 100vh;
            display: flex;
            flex-direction: column;
        }
        .header {
            background: #1e1e1e;
            padding: 12px 16px;
            border-bottom: 1px solid #3e3e3e;
            display: flex;
            justify-content: space-between;
            align-items: center;
            cursor: move;
            user-select: none;
        }
        .header h1 {
            font-size: 16px;
            font-weight: 600;
        }
        .header-buttons {
            display: flex;
            gap: 8px;
        }
        .header button {
            padding: 6px 12px;
            background: #0e639c;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 13px;
        }
        .header button:hover {
            background: #1177bb;
        }
        .header button.close-btn {
            background: #d13438;
            padding: 4px 8px;
            font-size: 12px;
        }
        .header button.close-btn:hover {
            background: #e81123;
        }
        .content {
            flex: 1;
            overflow-y: auto;
            padding: 8px;
        }
        .tree-node {
            user-select: none;
        }
        .tree-node-content {
            display: flex;
            align-items: center;
            gap: 4px;
            padding: 4px 8px;
            margin: 1px 0;
            cursor: pointer;
            border-radius: 3px;
        }
        .tree-node-content:hover {
            background: #3e3e3e;
        }
        .tree-node-content.selected {
            background: #0e639c;
        }
        .tree-toggle {
            width: 16px;
            height: 16px;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
            font-size: 10px;
            color: #888;
            transition: transform 0.2s;
        }
        .tree-toggle.expanded {
            transform: rotate(90deg);
        }
        .tree-toggle.empty {
            visibility: hidden;
        }
        .tree-node-icon {
            width: 16px;
            text-align: center;
            font-size: 12px;
        }
        .tree-node-name {
            flex: 1;
            font-size: 13px;
        }
        .tree-node-type {
            font-size: 11px;
            color: #888;
            margin-left: 8px;
        }
        .tree-children {
            margin-left: 16px;
            display: none;
        }
        .tree-children.expanded {
            display: block;
        }
        .context-menu {
            position: fixed;
            background: #1e1e1e;
            border: 1px solid #3e3e3e;
            border-radius: 4px;
            padding: 4px 0;
            box-shadow: 0 4px 12px rgba(0,0,0,0.5);
            display: none;
            z-index: 1000;
        }
        .context-menu-item {
            padding: 8px 16px;
            cursor: pointer;
            font-size: 13px;
        }
        .context-menu-item:hover {
            background: #3e3e3e;
        }
        .context-menu-separator {
            height: 1px;
            background: #3e3e3e;
            margin: 4px 0;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>[TREE] Maya Outliner</h1>
        <div class="header-buttons">
            <button onclick="testRender()">[TEST] Test</button>
            <button onclick="refreshScene()">[REFRESH] Refresh</button>
            <button class="close-btn" onclick="closeWindow()">[CLOSE] Close</button>
        </div>
    </div>
    <div class="content" id="content"></div>
    
    <div class="context-menu" id="contextMenu">
        <div class="context-menu-item" onclick="renameSelected()">[EDIT] Rename</div>
        <div class="context-menu-separator"></div>
        <div class="context-menu-item" onclick="deleteSelected()">[DELETE] Delete</div>
    </div>

    <script>
        let sceneData = [];
        let selectedNode = null;
        let contextMenuNode = null;
        let expandedNodes = new Set();  // Track expanded nodes

        function renderTree(nodes, container, level = 0) {
            console.log(' [renderTree] Called with:', {
                nodesCount: nodes.length,
                container: container,
                level: level,
                nodes: nodes
            });

            container.innerHTML = '';

            if (!nodes || nodes.length === 0) {
                console.warn('[WARNING] [renderTree] No nodes to render');
                container.innerHTML = '<div style="padding: 16px; color: #888;">No objects in scene</div>';
                return;
            }

            nodes.forEach((node, index) => {
                console.log(` [renderTree] Rendering node ${index}:`, node);
                const nodeEl = document.createElement('div');
                nodeEl.className = 'tree-node';
                nodeEl.dataset.nodePath = node.fullPath;

                // Create node content
                const contentEl = document.createElement('div');
                contentEl.className = 'tree-node-content';

                // Add toggle arrow for nodes with children
                const hasChildren = node.children && node.children.length > 0;
                const isExpanded = expandedNodes.has(node.fullPath);

                const toggleEl = document.createElement('span');
                toggleEl.className = 'tree-toggle' + (hasChildren ? (isExpanded ? ' expanded' : '') : ' empty');
                toggleEl.textContent = hasChildren ? '[PLAY]' : '';
                toggleEl.onclick = (e) => {
                    e.stopPropagation();
                    toggleNode(node, nodeEl);
                };

                // Add icon
                const iconEl = document.createElement('span');
                iconEl.className = 'tree-node-icon';
                iconEl.textContent = hasChildren ? '[FOLDER]' : '[DOCUMENT]';

                // Add name
                const nameEl = document.createElement('span');
                nameEl.className = 'tree-node-name';
                nameEl.textContent = node.name;

                // Add type
                const typeEl = document.createElement('span');
                typeEl.className = 'tree-node-type';
                typeEl.textContent = node.type;

                contentEl.appendChild(toggleEl);
                contentEl.appendChild(iconEl);
                contentEl.appendChild(nameEl);
                contentEl.appendChild(typeEl);

                // Click handler for selection
                contentEl.addEventListener('click', (e) => {
                    e.stopPropagation();
                    selectNode(node, contentEl);
                });

                // Context menu
                contentEl.addEventListener('contextmenu', (e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    showContextMenu(e.clientX, e.clientY, node);
                });

                nodeEl.appendChild(contentEl);

                // Add children container
                if (hasChildren) {
                    const childrenEl = document.createElement('div');
                    childrenEl.className = 'tree-children' + (isExpanded ? ' expanded' : '');
                    renderTree(node.children, childrenEl, level + 1);
                    nodeEl.appendChild(childrenEl);
                }

                container.appendChild(nodeEl);
            });
        }

        function toggleNode(node, nodeEl) {
            const childrenEl = nodeEl.querySelector('.tree-children');
            const toggleEl = nodeEl.querySelector('.tree-toggle');

            if (!childrenEl) return;

            const isExpanded = expandedNodes.has(node.fullPath);

            if (isExpanded) {
                // Collapse
                expandedNodes.delete(node.fullPath);
                childrenEl.classList.remove('expanded');
                toggleEl.classList.remove('expanded');
            } else {
                // Expand
                expandedNodes.add(node.fullPath);
                childrenEl.classList.add('expanded');
                toggleEl.classList.add('expanded');
            }
        }

        function selectNode(node, contentEl) {
            // Remove previous selection
            document.querySelectorAll('.tree-node-content').forEach(el => {
                el.classList.remove('selected');
            });

            // Add selection to clicked node
            contentEl.classList.add('selected');
            selectedNode = node;

            // Notify Python
            try {
                window.dispatchEvent(new CustomEvent('select_object', {
                    detail: { fullPath: node.fullPath }
                }));
            } catch (e) {
                console.error('[ERROR] [selectNode] Failed to dispatch event:', e);
            }
        }

        function showContextMenu(x, y, node) {
            const menu = document.getElementById('contextMenu');
            menu.style.left = x + 'px';
            menu.style.top = y + 'px';
            menu.style.display = 'block';
            contextMenuNode = node;
        }

        function hideContextMenu() {
            document.getElementById('contextMenu').style.display = 'none';
        }

        document.addEventListener('click', hideContextMenu);

        function testRender() {
            console.log('[TEST] [testRender] Testing render with mock data...');
            const testData = [
                {
                    name: 'TestCube',
                    fullPath: '|TestCube',
                    type: 'transform',
                    children: [
                        {
                            name: 'TestCubeShape',
                            fullPath: '|TestCube|TestCubeShape',
                            type: 'mesh',
                            children: []
                        }
                    ]
                },
                {
                    name: 'TestSphere',
                    fullPath: '|TestSphere',
                    type: 'transform',
                    children: []
                }
            ];
            console.log('[TEST] [testRender] Test data:', testData);
            renderTree(testData, document.getElementById('content'));
            console.log('[OK] [testRender] Test render complete');
        }

        function refreshScene() {
            console.log('[SEND] [refreshScene] Dispatching refresh_scene event...');
            try {
                window.dispatchEvent(new CustomEvent('refresh_scene', {
                    detail: { timestamp: Date.now() }
                }));
                console.log('[OK] [refreshScene] Event dispatched');
            } catch (e) {
                console.error('[ERROR] [refreshScene] Failed to dispatch event:', e);
            }
        }

        function closeWindow() {
            console.log('=' + '='.repeat(79));
            console.log('[SEND] [closeWindow] Close button clicked!');
            console.log('[SEND] [closeWindow] Dispatching close_window event...');
            console.log('[SEND] [closeWindow] window.ipc:', window.ipc);
            console.log('[SEND] [closeWindow] EventTarget.prototype.dispatchEvent:', EventTarget.prototype.dispatchEvent);

            try {
                const event = new CustomEvent('close_window', {
                    detail: { timestamp: Date.now(), source: 'close_button' }
                });
                console.log('[SEND] [closeWindow] Event created:', event);

                const result = window.dispatchEvent(event);
                console.log('[OK] [closeWindow] Close event dispatched, result:', result);
            } catch (e) {
                console.error('[ERROR] [closeWindow] Failed to dispatch close event:', e);
                console.error('[ERROR] [closeWindow] Stack trace:', e.stack);
            }
            console.log('=' + '='.repeat(79));
        }

        function renameSelected() {
            if (!contextMenuNode) return;
            const newName = prompt('Enter new name:', contextMenuNode.name);
            if (newName && newName.trim()) {
                try {
                    window.dispatchEvent(new CustomEvent('rename_object', {
                        detail: { fullPath: contextMenuNode.fullPath, newName: newName.trim() }
                    }));
                } catch (e) {
                    console.error('[ERROR] [renameSelected] Failed to dispatch event:', e);
                }
            }
            hideContextMenu();
        }

        function deleteSelected() {
            if (!contextMenuNode) return;
            if (confirm('Delete "' + contextMenuNode.name + '"?')) {
                try {
                    window.dispatchEvent(new CustomEvent('delete_object', {
                        detail: { fullPath: contextMenuNode.fullPath }
                    }));
                } catch (e) {
                    console.error('[ERROR] [deleteSelected] Failed to dispatch event:', e);
                }
            }
            hideContextMenu();
        }

        // Create a global handler for Python events (bypass event bridge issues)
        window.__handlePythonEvent = function(eventName, data) {
            console.log('[RECV] [__handlePythonEvent] Received:', eventName, data);

            // Dispatch as CustomEvent for compatibility
            const event = new CustomEvent(eventName, {
                detail: data,
                // Mark as already processed to avoid IPC loop
                __processed: true
            });
            window.dispatchEvent(event);
        };

        // Debug: Log all CustomEvents
        const originalDispatch = EventTarget.prototype.dispatchEvent;
        EventTarget.prototype.dispatchEvent = function(event) {
            if (event instanceof CustomEvent) {
                console.log('[SEARCH] [DEBUG] CustomEvent dispatched:', {
                    type: event.type,
                    detail: event.detail,
                    fromPython: event.detail?.__aurora_from_python,
                    processed: event.__processed
                });
            }
            return originalDispatch.call(this, event);
        };

        window.addEventListener('scene_updated', (e) => {
            console.log('[RECV] [scene_updated] Event received');
            console.log('[RECV] [scene_updated] Event detail:', e.detail);
            console.log('[RECV] [scene_updated] Event detail type:', typeof e.detail);

            // Validate event data
            if (!e.detail) {
                console.error('[ERROR] [scene_updated] Event detail is empty');
                return;
            }

            // Extract hierarchy with validation
            sceneData = e.detail.hierarchy;

            if (!Array.isArray(sceneData)) {
                console.error('[ERROR] [scene_updated] Hierarchy is not an array:', sceneData);
                console.error('[ERROR] [scene_updated] Hierarchy type:', typeof sceneData);
                return;
            }

            console.log('[RECV] [scene_updated] Scene data:', sceneData);
            console.log('[RECV] [scene_updated] Number of root nodes:', sceneData.length);

            const contentEl = document.getElementById('content');
            console.log('[RECV] [scene_updated] Content element:', contentEl);

            if (!contentEl) {
                console.error('[ERROR] [scene_updated] Content element not found!');
                return;
            }

            try {
                renderTree(sceneData, contentEl);
                console.log('[OK] [scene_updated] Tree rendered successfully');
            } catch (err) {
                console.error('[ERROR] [scene_updated] renderTree failed:', err);
                console.error('[ERROR] [scene_updated] Error stack:', err.stack);
            }
        });

        window.addEventListener('rename_result', (e) => {
            console.log('[RECV] [rename_result] Event received:', e.detail);
            const d = e.detail || {};
            if (!d.ok) alert('Rename failed: ' + (d.error || 'unknown error'));
        });

        window.addEventListener('delete_result', (e) => {
            console.log('[RECV] [delete_result] Event received:', e.detail);
            const d = e.detail || {};
            if (!d.ok) alert('Delete failed: ' + (d.error || 'unknown error'));
        });

        // Window dragging functionality for frameless window
        let isDragging = false;
        let dragStartX = 0;
        let dragStartY = 0;
        let windowStartX = 0;
        let windowStartY = 0;
        let lastMoveTime = 0;
        const MOVE_THROTTLE_MS = 16; // ~60fps

        const header = document.querySelector('.header');

        header.addEventListener('mousedown', (e) => {
            // Don't drag if clicking on buttons
            if (e.target.closest('.header-buttons')) {
                return;
            }

            isDragging = true;
            dragStartX = e.screenX;
            dragStartY = e.screenY;

            // Calculate current window position from screen coordinates
            // This works because screenX/Y are absolute screen coordinates
            // and clientX/Y are relative to the window
            windowStartX = e.screenX - e.clientX;
            windowStartY = e.screenY - e.clientY;

            console.log('� [drag] Started:', {
                dragStartX,
                dragStartY,
                windowStartX,
                windowStartY,
                screenX: e.screenX,
                screenY: e.screenY,
                clientX: e.clientX,
                clientY: e.clientY
            });
        });

        document.addEventListener('mousemove', (e) => {
            if (!isDragging) return;

            // Throttle move events for better performance
            const now = Date.now();
            if (now - lastMoveTime < MOVE_THROTTLE_MS) {
                return;
            }
            lastMoveTime = now;

            // Calculate delta from drag start
            const deltaX = e.screenX - dragStartX;
            const deltaY = e.screenY - dragStartY;

            // Calculate new window position based on initial position + delta
            const newX = windowStartX + deltaX;
            const newY = windowStartY + deltaY;

            // Send move_window event to Python
            try {
                console.log('[SEND] [drag] Sending move_window event:', { x: newX, y: newY });

                // Create event with explicit data structure
                const eventData = {
                    x: Math.round(newX),
                    y: Math.round(newY)
                };

                console.log('[SEND] [drag] Event data:', eventData);
                console.log('[SEND] [drag] Event data type:', typeof eventData);
                console.log('[SEND] [drag] Event data keys:', Object.keys(eventData));

                window.dispatchEvent(new CustomEvent('move_window', {
                    detail: eventData,
                    bubbles: true,
                    cancelable: true
                }));

                console.log('[OK] [drag] move_window event dispatched');
            } catch (err) {
                console.error('[ERROR] [drag] Failed to send move_window event:', err);
                console.error('[ERROR] [drag] Error stack:', err.stack);
            }
        });

        document.addEventListener('mouseup', () => {
            if (isDragging) {
                isDragging = false;
                console.log('[OK] [drag] Stopped');
            }
        });

        // Notify Python that JavaScript is ready
        console.log('[OK] [init] JavaScript initialized');
        console.log('[SEND] [init] Notifying Python that WebView is ready...');

        // Show loading message
        document.getElementById('content').innerHTML = '<div style="padding: 16px; color: #888;">Loading scene data...</div>';

        // Use setTimeout to ensure the event system is fully initialized
        setTimeout(() => {
            try {
                console.log('[SEND] [init] Dispatching webview_ready event...');
                window.dispatchEvent(new CustomEvent('webview_ready', {
                    detail: { timestamp: Date.now() }
                }));
                console.log('[OK] [init] webview_ready event dispatched');

                // Also trigger an immediate refresh in case the event doesn't reach Python
                console.log('[SEND] [init] Triggering immediate refresh as fallback...');
                refreshScene();
            } catch (e) {
                console.error('[ERROR] [init] Failed to dispatch webview_ready event:', e);
            }
        }, 100);
    </script>
</body>
</html>
"""

# Load HTML
print("[DOCUMENT] [main] Loading HTML...")
webview.load_html(html)
print("[OK] [main] HTML loaded")

# Store in global variable BEFORE showing
import __main__
__main__.maya_outliner = webview
print("[OK] [main] WebView stored in __main__.maya_outliner")

# CRITICAL: Create event processing timer BEFORE showing window
# This ensures process_events() is called immediately after window creation
print("[TIMER] [main] Creating event processing timer...")

def process_webview_events():
    """Process WebView events and check if window should close.

    This function is called by Maya's scriptJob on every idle event.
    It processes Windows messages for the WebView window without blocking.
    """
    try:
        if hasattr(__main__, 'maya_outliner'):
            # Process events and check if window should close
            # This is NON-BLOCKING - it only processes pending messages
            should_close = __main__.maya_outliner._core.process_events()

            if should_close:
                print("=" * 80)
                print("[CLOSE] [process_webview_events] Window close signal detected!")
                print("[CLOSE] [process_webview_events] Cleaning up resources...")
                print("=" * 80)

                # Kill all related scriptJobs
                if hasattr(__main__, 'maya_outliner_timer'):
                    print(f"[CLOSE] Killing timer job: {__main__.maya_outliner_timer}")
                    cmds.scriptJob(kill=__main__.maya_outliner_timer)
                    del __main__.maya_outliner_timer
                    print("[OK] Timer job killed")

                if hasattr(__main__, 'maya_outliner_scene_jobs'):
                    print(f"[CLOSE] Killing {len(__main__.maya_outliner_scene_jobs)} scene jobs")
                    for job_id in __main__.maya_outliner_scene_jobs:
                        cmds.scriptJob(kill=job_id)
                    del __main__.maya_outliner_scene_jobs
                    print("[OK] Scene jobs killed")

                # Delete the WebView object
                print("[CLOSE] Deleting WebView object...")
                del __main__.maya_outliner
                print("[OK] WebView object deleted")
                print("=" * 80)

    except Exception as e:
        print(f"[WARNING] [process_webview_events] Error: {e}")
        traceback.print_exc()

# Create the timer BEFORE showing the window
timer_id = cmds.scriptJob(event=["idle", process_webview_events])
__main__.maya_outliner_timer = timer_id
print(f"[OK] [main] Event processing timer created (ID: {timer_id})")

# NOW show the window
# CRITICAL: Use show() NOT show_async()
# The window is created in Maya's main thread (this thread)
# The scriptJob will handle message processing via process_events()
print("[WINDOW] [main] Showing window in main thread...")
webview.show()
print("[OK] [main] Window shown (non-blocking via scriptJob)")

# Initial refresh will be triggered by the webview_ready event from JavaScript
print("[OK] [main] Initial refresh will be triggered by webview_ready event")

# Create scene change listeners for auto-refresh
print(" [main] Creating scene change listeners...")
scene_jobs = []

# Listen for DAG object creation (new objects)
job1 = cmds.scriptJob(event=["DagObjectCreated", refresh_outliner])
scene_jobs.append(job1)
print(f"[OK] [main] DagObjectCreated listener created (ID: {job1})")

# Listen for name changes (rename)
job2 = cmds.scriptJob(event=["NameChanged", refresh_outliner])
scene_jobs.append(job2)
print(f"[OK] [main] NameChanged listener created (ID: {job2})")

# Listen for parent changes (reparenting in hierarchy)
job3 = cmds.scriptJob(event=["DagObjectParentChanged", refresh_outliner])
scene_jobs.append(job3)
print(f"[OK] [main] DagObjectParentChanged listener created (ID: {job3})")

# Store scene job IDs for cleanup
__main__.maya_outliner_scene_jobs = scene_jobs
print(f"[OK] [main] Created {len(scene_jobs)} scene change listeners")

print("[OK] Maya Outliner created (DEBUG VERSION)")
print("=" * 70)
print("Features:")
print(" Real-time scene hierarchy display")
print(" Click to select objects in Maya")
print(" Right-click for context menu (Rename, Delete)")
print(" Auto-refresh after operations")
print("")
print("Debug Commands:")
print("  # Manual refresh:")
print("  __main__.maya_outliner_refresh_outliner()")
print("")
print("  # Check scene objects:")
print("  import maya.cmds as cmds")
print("  print(cmds.ls(assemblies=True))")
print("")
print("To close:")
print("  del __main__.maya_outliner")
print("  cmds.scriptJob(kill=__main__.maya_outliner_timer)")
print("=" * 70)

# Store refresh function for manual testing
__main__.maya_outliner_refresh_outliner = refresh_outliner
print("[OK] [main] Refresh function stored in __main__.maya_outliner_refresh_outliner")

# Debug helper function
def debug_webview_state():
    """Debug helper to check WebView state and event handlers"""
    import __main__
    print("\n" + "=" * 80)
    print("[SEARCH] [DEBUG] WebView State Check")
    print("=" * 80)

    if not hasattr(__main__, 'maya_outliner'):
        print("[ERROR] WebView not found in __main__.maya_outliner")
        return

    wv = __main__.maya_outliner
    print(f"[OK] WebView object: {wv}")
    print(f"[OK] WebView._core: {wv._core}")

    # Test scene hierarchy retrieval
    print("\n[STATS] Testing scene hierarchy retrieval...")
    try:
        hierarchy = get_scene_hierarchy()
        print(f"[OK] Scene hierarchy retrieved: {len(hierarchy)} root nodes")
        if hierarchy:
            print(f"   First node: {hierarchy[0]['name']} ({hierarchy[0]['type']})")
    except Exception as e:
        print(f"[ERROR] Scene hierarchy retrieval failed: {e}")

    # Test event emission
    print("\n[SEND] Testing event emission...")
    try:
        wv.emit('test_event', {'message': 'Debug test'})
        print("[OK] Event emission successful")
    except Exception as e:
        print(f"[ERROR] Event emission failed: {e}")

    print("=" * 80 + "\n")

__main__.debug_webview_state = debug_webview_state
print("[OK] [main] Debug function stored in __main__.debug_webview_state")

