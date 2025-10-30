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
print("✓ WebView created in Maya's main thread")
print("✓ scriptJob handles event processing via process_events()")
print("✓ Maya remains responsive while WebView is open")
print("=" * 70)
print("")

# Get Maya main window
print("🔍 Getting Maya main window handle...")
main_window_ptr = omui.MQtUtil.mainWindow()
maya_window = wrapInstance(int(main_window_ptr), QWidget)
hwnd = maya_window.winId()
print(f"✅ Maya window HWND: {hwnd}")
print("")

# Create WebView in Maya's main thread
print("🔨 Creating WebView in main thread...")
print("   - Mode: Owner (cross-thread safe)")
print("   - Parent HWND:", hwnd)
print("   - Decorations: False (no title bar)")
webview = NativeWebView(
    title="Maya Outliner",
    width=400,
    height=600,
    decorations=False,  # Remove title bar - use custom HTML controls
    parent_hwnd=hwnd,
    parent_mode="owner",  # Owner mode is safer for cross-thread scenarios
)
print("✅ WebView created successfully")
print("")

def get_scene_hierarchy():
    """Get Maya scene hierarchy as a tree structure"""
    print("🔍 [get_scene_hierarchy] Starting...")

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
                        valid_children.append(build_tree(child))
                except Exception as e:
                    print(f"⚠️ [build_tree] Error processing child {child}: {e}")

            return {
                'name': short_name,
                'fullPath': node,
                'type': node_type,
                'children': valid_children
            }
        except Exception as e:
            print(f"❌ [build_tree] Error building tree for {node}: {e}")
            traceback.print_exc()
            return None

    try:
        # Get all root transforms (objects without parents)
        root_nodes = cmds.ls(assemblies=True)
        print(f"🔍 [get_scene_hierarchy] Found {len(root_nodes)} root nodes: {root_nodes}")

        hierarchy = []
        for node in root_nodes:
            tree = build_tree(node)
            if tree:
                hierarchy.append(tree)

        print(f"✅ [get_scene_hierarchy] Built hierarchy with {len(hierarchy)} root nodes")
        return hierarchy
    except Exception as e:
        print(f"❌ [get_scene_hierarchy] Error: {e}")
        traceback.print_exc()
        return []

def refresh_outliner():
    """Refresh the outliner view"""
    print("🔄 [refresh_outliner] Called")

    def _do_refresh():
        try:
            print("🔄 [refresh_outliner._do_refresh] Executing in Maya main thread...")

            # Get webview from __main__
            import __main__
            if not hasattr(__main__, 'maya_outliner'):
                print("❌ [refresh_outliner._do_refresh] WebView not found in __main__.maya_outliner")
                return

            wv = __main__.maya_outliner
            print(f"✅ [refresh_outliner._do_refresh] Got WebView: {wv}")

            hierarchy = get_scene_hierarchy()
            print(f"🔄 [refresh_outliner._do_refresh] Got hierarchy: {len(hierarchy)} root nodes")
            print(f"🔍 [refresh_outliner._do_refresh] Hierarchy data: {json.dumps(hierarchy, indent=2)}")

            # Emit to JavaScript
            print(f"📤 [refresh_outliner._do_refresh] Emitting 'scene_updated' event...")
            wv.emit('scene_updated', {'hierarchy': hierarchy})
            print(f"✅ [refresh_outliner._do_refresh] Outliner refreshed ({len(hierarchy)} root nodes)")
        except Exception as e:
            print(f"❌ [refresh_outliner._do_refresh] Error: {e}")
            traceback.print_exc()

    import maya.utils as mutils
    print("🔄 [refresh_outliner] Queueing to Maya main thread...")
    mutils.executeDeferred(_do_refresh)

# Event handlers
@webview.on("webview_ready")
def handle_webview_ready(data):
    """Handle WebView ready notification from JavaScript"""
    print(f"📥 [handle_webview_ready] WebView is ready: {data}")
    print("🔄 [handle_webview_ready] Triggering initial refresh...")
    refresh_outliner()

@webview.on("refresh_scene")
def handle_refresh(data):
    """Handle refresh request from UI"""
    print(f"📥 [handle_refresh] Event received: {data}")
    refresh_outliner()

@webview.on("rename_object")
def handle_rename(data):
    """Handle rename request"""
    print(f"✏️ Rename request: {data}")

    def _do_rename():
        try:
            import __main__
            if not hasattr(__main__, 'maya_outliner'):
                print("❌ [handle_rename] WebView not found")
                return
            wv = __main__.maya_outliner

            full_path = data.get('fullPath')
            new_name = data.get('newName', '').strip()

            if not full_path or not new_name:
                wv.emit('rename_result', {'ok': False, 'error': 'Invalid parameters'})
                return

            # Check if object exists
            if not cmds.objExists(full_path):
                wv.emit('rename_result', {'ok': False, 'error': 'Object not found'})
                return

            # Rename
            new_full_path = cmds.rename(full_path, new_name)
            print(f"✅ Renamed: {full_path} → {new_full_path}")

            wv.emit('rename_result', {'ok': True, 'oldPath': full_path, 'newPath': new_full_path})

            # Refresh outliner
            refresh_outliner()

        except Exception as e:
            print(f"❌ Rename error: {e}")
            import __main__
            if hasattr(__main__, 'maya_outliner'):
                __main__.maya_outliner.emit('rename_result', {'ok': False, 'error': str(e)})

    import maya.utils as mutils
    mutils.executeDeferred(_do_rename)

@webview.on("delete_object")
def handle_delete(data):
    """Handle delete request"""
    print(f"🗑️ Delete request: {data}")

    def _do_delete():
        try:
            import __main__
            if not hasattr(__main__, 'maya_outliner'):
                print("❌ [handle_delete] WebView not found")
                return
            wv = __main__.maya_outliner

            full_path = data.get('fullPath')

            if not full_path:
                wv.emit('delete_result', {'ok': False, 'error': 'Invalid parameters'})
                return

            # Check if object exists
            if not cmds.objExists(full_path):
                wv.emit('delete_result', {'ok': False, 'error': 'Object not found'})
                return

            # Delete
            cmds.delete(full_path)
            print(f"✅ Deleted: {full_path}")

            wv.emit('delete_result', {'ok': True, 'path': full_path})

            # Refresh outliner
            refresh_outliner()

        except Exception as e:
            print(f"❌ Delete error: {e}")
            import __main__
            if hasattr(__main__, 'maya_outliner'):
                __main__.maya_outliner.emit('delete_result', {'ok': False, 'error': str(e)})

    import maya.utils as mutils
    mutils.executeDeferred(_do_delete)

@webview.on("select_object")
def handle_select(data):
    """Handle selection request"""
    print(f"👆 Select request: {data}")
    
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
            print(f"✅ Selected: {full_path}")
            
        except Exception as e:
            print(f"❌ Select error: {e}")
    
    import maya.utils as mutils
    mutils.executeDeferred(_do_select)

# System control handlers
@webview.on("close_window")
def _handle_close(data):
    """Handle close request from JavaScript"""
    print("=" * 80)
    print("🔒 [_handle_close] Close requested from UI")
    print(f"🔒 [_handle_close] Event data: {data}")
    print("=" * 80)

    def _do_close():
        try:
            print("🔒 [_do_close] Attempting to close WebView...")
            print(f"🔒 [_do_close] WebView object: {webview}")
            print(f"🔒 [_do_close] WebView._core: {webview._core}")

            # Close the WebView window
            webview.close()
            print("✅ [_do_close] WebView.close() called successfully")

            # Also try to kill the scriptJob
            import __main__
            if hasattr(__main__, 'maya_outliner_timer'):
                print(f"🔒 [_do_close] Killing scriptJob: {__main__.maya_outliner_timer}")
                cmds.scriptJob(kill=__main__.maya_outliner_timer)
                del __main__.maya_outliner_timer
                print("✅ [_do_close] ScriptJob killed")

        except Exception as e:
            print(f"❌ [_do_close] Close error: {e}")
            traceback.print_exc()

    import maya.utils as mutils
    print("🔒 [_handle_close] Queueing close operation to Maya main thread...")
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
            padding: 4px 8px;
            margin: 2px 0;
            cursor: pointer;
            border-radius: 4px;
            user-select: none;
        }
        .tree-node:hover {
            background: #3e3e3e;
        }
        .tree-node.selected {
            background: #0e639c;
        }
        .tree-node-content {
            display: flex;
            align-items: center;
            gap: 6px;
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
        }
        .tree-children {
            margin-left: 20px;
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
        <h1>🌲 Maya Outliner</h1>
        <div class="header-buttons">
            <button onclick="refreshScene()">🔄 Refresh</button>
            <button class="close-btn" onclick="closeWindow()">✕ Close</button>
        </div>
    </div>
    <div class="content" id="content"></div>
    
    <div class="context-menu" id="contextMenu">
        <div class="context-menu-item" onclick="renameSelected()">✏️ Rename</div>
        <div class="context-menu-separator"></div>
        <div class="context-menu-item" onclick="deleteSelected()">🗑️ Delete</div>
    </div>

    <script>
        let sceneData = [];
        let selectedNode = null;
        let contextMenuNode = null;

        function renderTree(nodes, container) {
            container.innerHTML = '';
            nodes.forEach(node => {
                const nodeEl = document.createElement('div');
                nodeEl.className = 'tree-node';

                const icon = node.children && node.children.length > 0 ? '📁' : '📄';
                nodeEl.innerHTML =
                    '<div class="tree-node-content">' +
                        '<span class="tree-node-icon">' + icon + '</span>' +
                        '<span class="tree-node-name">' + node.name + '</span>' +
                        '<span class="tree-node-type">' + node.type + '</span>' +
                    '</div>';

                nodeEl.addEventListener('click', (e) => {
                    e.stopPropagation();
                    selectNode(node, nodeEl);
                });

                nodeEl.addEventListener('contextmenu', (e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    showContextMenu(e.clientX, e.clientY, node);
                });

                container.appendChild(nodeEl);

                if (node.children && node.children.length > 0) {
                    const childrenEl = document.createElement('div');
                    childrenEl.className = 'tree-children';
                    renderTree(node.children, childrenEl);
                    container.appendChild(childrenEl);
                }
            });
        }

        function selectNode(node, element) {
            document.querySelectorAll('.tree-node').forEach(el => el.classList.remove('selected'));
            element.classList.add('selected');
            selectedNode = node;

            try {
                window.dispatchEvent(new CustomEvent('select_object', {
                    detail: { fullPath: node.fullPath }
                }));
            } catch (e) {
                console.error('❌ [selectNode] Failed to dispatch event:', e);
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

        function refreshScene() {
            console.log('📤 [refreshScene] Dispatching refresh_scene event...');
            try {
                window.dispatchEvent(new CustomEvent('refresh_scene', {
                    detail: { timestamp: Date.now() }
                }));
                console.log('✅ [refreshScene] Event dispatched');
            } catch (e) {
                console.error('❌ [refreshScene] Failed to dispatch event:', e);
            }
        }

        function closeWindow() {
            console.log('=' + '='.repeat(79));
            console.log('📤 [closeWindow] Close button clicked!');
            console.log('📤 [closeWindow] Dispatching close_window event...');
            console.log('📤 [closeWindow] window.ipc:', window.ipc);
            console.log('📤 [closeWindow] EventTarget.prototype.dispatchEvent:', EventTarget.prototype.dispatchEvent);

            try {
                const event = new CustomEvent('close_window', {
                    detail: { timestamp: Date.now(), source: 'close_button' }
                });
                console.log('📤 [closeWindow] Event created:', event);

                const result = window.dispatchEvent(event);
                console.log('✅ [closeWindow] Close event dispatched, result:', result);
            } catch (e) {
                console.error('❌ [closeWindow] Failed to dispatch close event:', e);
                console.error('❌ [closeWindow] Stack trace:', e.stack);
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
                    console.error('❌ [renameSelected] Failed to dispatch event:', e);
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
                    console.error('❌ [deleteSelected] Failed to dispatch event:', e);
                }
            }
            hideContextMenu();
        }

        window.addEventListener('scene_updated', (e) => {
            console.log('📥 [scene_updated] Event received:', e.detail);
            sceneData = e.detail.hierarchy || [];
            console.log('📥 [scene_updated] Scene data:', sceneData);
            console.log('📥 [scene_updated] Number of root nodes:', sceneData.length);
            renderTree(sceneData, document.getElementById('content'));
            console.log('✅ [scene_updated] Tree rendered');
        });

        window.addEventListener('rename_result', (e) => {
            console.log('📥 [rename_result] Event received:', e.detail);
            const d = e.detail || {};
            if (!d.ok) alert('Rename failed: ' + (d.error || 'unknown error'));
        });

        window.addEventListener('delete_result', (e) => {
            console.log('📥 [delete_result] Event received:', e.detail);
            const d = e.detail || {};
            if (!d.ok) alert('Delete failed: ' + (d.error || 'unknown error'));
        });

        // Notify Python that JavaScript is ready
        console.log('✅ [init] JavaScript initialized');
        console.log('📤 [init] Notifying Python that WebView is ready...');

        // Use setTimeout to ensure the event system is fully initialized
        setTimeout(() => {
            try {
                window.dispatchEvent(new CustomEvent('webview_ready', {
                    detail: { timestamp: Date.now() }
                }));
                console.log('✅ [init] webview_ready event dispatched');
            } catch (e) {
                console.error('❌ [init] Failed to dispatch webview_ready event:', e);
            }
        }, 100);
    </script>
</body>
</html>
"""

# Load HTML
print("📄 [main] Loading HTML...")
webview.load_html(html)
print("✅ [main] HTML loaded")

# Store in global variable BEFORE showing
import __main__
__main__.maya_outliner = webview
print("✅ [main] WebView stored in __main__.maya_outliner")

# CRITICAL: Create event processing timer BEFORE showing window
# This ensures process_events() is called immediately after window creation
print("⏱️ [main] Creating event processing timer...")

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
                print("🔴 [process_webview_events] Window close signal detected!")
                print("🔴 [process_webview_events] Cleaning up resources...")
                print("=" * 80)

                # Kill all related scriptJobs
                if hasattr(__main__, 'maya_outliner_timer'):
                    print(f"🔴 Killing timer job: {__main__.maya_outliner_timer}")
                    cmds.scriptJob(kill=__main__.maya_outliner_timer)
                    del __main__.maya_outliner_timer
                    print("✅ Timer job killed")

                if hasattr(__main__, 'maya_outliner_scene_jobs'):
                    print(f"🔴 Killing {len(__main__.maya_outliner_scene_jobs)} scene jobs")
                    for job_id in __main__.maya_outliner_scene_jobs:
                        cmds.scriptJob(kill=job_id)
                    del __main__.maya_outliner_scene_jobs
                    print("✅ Scene jobs killed")

                # Delete the WebView object
                print("🔴 Deleting WebView object...")
                del __main__.maya_outliner
                print("✅ WebView object deleted")
                print("=" * 80)

    except Exception as e:
        print(f"⚠️ [process_webview_events] Error: {e}")
        traceback.print_exc()

# Create the timer BEFORE showing the window
timer_id = cmds.scriptJob(event=["idle", process_webview_events])
__main__.maya_outliner_timer = timer_id
print(f"✅ [main] Event processing timer created (ID: {timer_id})")

# NOW show the window
# CRITICAL: Use show() NOT show_async()
# The window is created in Maya's main thread (this thread)
# The scriptJob will handle message processing via process_events()
print("🪟 [main] Showing window in main thread...")
webview.show()
print("✅ [main] Window shown (non-blocking via scriptJob)")

# Initial refresh will be triggered by the webview_ready event from JavaScript
print("✅ [main] Initial refresh will be triggered by webview_ready event")

# Create scene change listeners for auto-refresh
print("👂 [main] Creating scene change listeners...")
scene_jobs = []

# Listen for DAG object creation (new objects)
job1 = cmds.scriptJob(event=["DagObjectCreated", refresh_outliner])
scene_jobs.append(job1)
print(f"✅ [main] DagObjectCreated listener created (ID: {job1})")

# Listen for name changes (rename)
job2 = cmds.scriptJob(event=["NameChanged", refresh_outliner])
scene_jobs.append(job2)
print(f"✅ [main] NameChanged listener created (ID: {job2})")

# Listen for parent changes (reparenting in hierarchy)
job3 = cmds.scriptJob(event=["DagObjectParentChanged", refresh_outliner])
scene_jobs.append(job3)
print(f"✅ [main] DagObjectParentChanged listener created (ID: {job3})")

# Store scene job IDs for cleanup
__main__.maya_outliner_scene_jobs = scene_jobs
print(f"✅ [main] Created {len(scene_jobs)} scene change listeners")

print("✅ Maya Outliner created (DEBUG VERSION)")
print("=" * 70)
print("Features:")
print("• Real-time scene hierarchy display")
print("• Click to select objects in Maya")
print("• Right-click for context menu (Rename, Delete)")
print("• Auto-refresh after operations")
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
print("✅ [main] Refresh function stored in __main__.maya_outliner_refresh_outliner")

