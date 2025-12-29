#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Floating Toolbar Demo - Transparent floating tool shelf with GSAP animations.

This example demonstrates how to create a floating toolbar that displays
local application shortcuts with smooth GSAP animations.

Features demonstrated:
- Transparent, frameless window (truly transparent with no shadow)
- Circular trigger button that expands into a toolbar
- GSAP-powered animations for smooth transitions
- Dynamic tool discovery from Windows Start Menu / Applications
- Drag support for repositioning
- Tool window style (hide from taskbar/Alt+Tab)

Use cases:
- Quick launcher for DCC applications
- Floating tool palette for workflows
- Application dock/launcher
- Context-sensitive tool shelf

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import os
import subprocess
import sys

# HTML for the floating toolbar with GSAP animations
TOOLBAR_HTML = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/gsap.min.js"></script>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        html, body {
            width: 100%;
            height: 100%;
            background: transparent !important;
            overflow: hidden;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        .container {
            width: 100%;
            height: 100%;
            display: flex;
            align-items: flex-start;
            justify-content: flex-start;
            padding: 8px;
        }

        /* Trigger button */
        .trigger-btn {
            width: 48px;
            height: 48px;
            border-radius: 50%;
            background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
            border: none;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            box-shadow: none;
            transition: none;

            position: relative;
            z-index: 100;
            flex-shrink: 0;
        }



        .trigger-btn:hover {
            box-shadow: none;
        }


        .trigger-btn svg {
            width: 24px;
            height: 24px;
            fill: white;
            transition: transform 0.3s;
        }

        .trigger-btn.expanded svg {
            transform: rotate(45deg);
        }

        /* Toolbar container */
        .toolbar {
            display: flex;
            align-items: center;
            gap: 8px;
            margin-left: 12px;
            overflow: hidden;
        }

        .toolbar-inner {
            display: flex;
            align-items: center;
            gap: 8px;
            background: rgba(30, 30, 46, 0.95);
            backdrop-filter: blur(20px);
            border-radius: 28px;
            padding: 8px 16px;
            border: 1px solid rgba(255, 255, 255, 0.1);
            box-shadow: none;

        }

        /* Tool item */
        .tool-item {
            width: 40px;
            height: 40px;
            border-radius: 12px;
            background: rgba(255, 255, 255, 0.05);
            border: 1px solid rgba(255, 255, 255, 0.1);
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.2s;
            position: relative;
            opacity: 0;
            transform: scale(0.5);
        }



        .tool-item:hover {
            background: rgba(99, 102, 241, 0.2);
            border-color: #6366f1;
            transform: translateY(-2px) scale(1.05);
        }

        .tool-item:active {
            transform: translateY(0) scale(0.95);
        }

        .tool-item img {
            width: 24px;
            height: 24px;
            object-fit: contain;
        }

        .tool-item .icon-placeholder {
            width: 24px;
            height: 24px;
            border-radius: 6px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 12px;
            font-weight: 600;
            color: white;
        }

        /* Tooltip */
        .tool-item::after {
            content: attr(data-tooltip);
            position: absolute;
            bottom: 100%;
            left: 50%;
            transform: translateX(-50%);
            background: rgba(0, 0, 0, 0.9);
            color: white;
            padding: 6px 10px;
            border-radius: 6px;
            font-size: 11px;
            white-space: nowrap;
            opacity: 0;
            pointer-events: none;
            transition: opacity 0.2s, transform 0.2s;
            margin-bottom: 8px;
        }

        .tool-item:hover::after {
            opacity: 1;
            transform: translateX(-50%) translateY(-4px);
        }

        /* Add tool button */
        .add-tool {
            width: 40px;
            height: 40px;
            border-radius: 12px;
            background: rgba(255, 255, 255, 0.05);
            border: 2px dashed rgba(255, 255, 255, 0.2);
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.2s;
            opacity: 0;
            transform: scale(0.5);
        }

        .add-tool:hover {
            background: rgba(99, 102, 241, 0.1);
            border-color: #6366f1;
        }

        .add-tool svg {
            width: 20px;
            height: 20px;
            fill: rgba(255, 255, 255, 0.5);
        }

        /* Drag handle */
        .drag-handle {
            -webkit-app-region: drag;
            cursor: move;
        }

        .no-drag {
            -webkit-app-region: no-drag;
        }

        /* Separator */
        .separator {
            width: 1px;
            height: 24px;
            background: rgba(255, 255, 255, 0.1);
            margin: 0 4px;
            opacity: 0;
        }

        /* Color palette for tool icons */
        .color-1 { background: linear-gradient(135deg, #f43f5e, #ec4899); }
        .color-2 { background: linear-gradient(135deg, #f97316, #f59e0b); }
        .color-3 { background: linear-gradient(135deg, #22c55e, #10b981); }
        .color-4 { background: linear-gradient(135deg, #3b82f6, #6366f1); }
        .color-5 { background: linear-gradient(135deg, #8b5cf6, #a855f7); }
        .color-6 { background: linear-gradient(135deg, #06b6d4, #0ea5e9); }
    </style>
</head>
<body>
    <div class="container">
        <button class="trigger-btn no-drag" id="triggerBtn" onclick="toggleToolbar()">
            <svg viewBox="0 0 24 24">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
            </svg>
        </button>

        <div class="toolbar" id="toolbar">
            <div class="toolbar-inner drag-handle" id="toolbarInner">
                <!-- Tools will be inserted here -->
            </div>
        </div>
    </div>

    <script>
        let isExpanded = false;
        let tools = [];

        // Initialize
        window.addEventListener('auroraviewready', () => {
            console.log('[FloatingToolbar] AuroraView ready');
            installNativeDrag();
            loadTools();
        });

        function installNativeDrag() {
            try {
                if (window.__floating_toolbar_native_drag_installed) return;
                window.__floating_toolbar_native_drag_installed = true;

                const DRAG_THRESHOLD_PX = 4;
                let pending = null; // { x, y, pointerId }
                let suppressClickUntil = 0;

                function isDragSource(el) {
                    // Allow dragging from the trigger button and tool icons.
                    // Keep the add button clickable.
                    if (el.closest('.add-tool')) return false;
                    return !!el.closest('.trigger-btn, .tool-item, .toolbar-inner');
                }

                document.addEventListener('pointerdown', (e) => {
                    try {
                        if (e.button !== 0) return;
                        const t = e.target;
                        if (!t || !(t instanceof Element)) return;
                        if (!isDragSource(t)) return;

                        pending = { x: e.clientX, y: e.clientY, pointerId: e.pointerId };
                    } catch (err) {
                        console.warn('[FloatingToolbar] installNativeDrag pointerdown error:', err);
                    }
                }, true);

                document.addEventListener('pointermove', (e) => {
                    try {
                        if (!pending) return;
                        if (e.pointerId !== pending.pointerId) return;
                        const dx = e.clientX - pending.x;
                        const dy = e.clientY - pending.y;
                        if (dx * dx + dy * dy < DRAG_THRESHOLD_PX * DRAG_THRESHOLD_PX) return;
                        pending = null;

                        if (window.auroraview && typeof window.auroraview.startDrag === 'function') {
                            suppressClickUntil = Date.now() + 800;
                            window.auroraview.startDrag();
                            e.preventDefault();
                        }
                    } catch (err) {
                        console.warn('[FloatingToolbar] installNativeDrag pointermove error:', err);
                    }
                }, true);

                function clearPending() {
                    pending = null;
                }

                document.addEventListener('pointerup', clearPending, true);
                document.addEventListener('pointercancel', clearPending, true);
                window.addEventListener('blur', clearPending, true);

                document.addEventListener('click', (e) => {
                    try {
                        if (Date.now() < suppressClickUntil) {
                            e.preventDefault();
                            e.stopPropagation();
                        }
                    } catch (_) {
                        // ignore
                    }
                }, true);
            } catch (e) {
                console.warn('[FloatingToolbar] Failed to install native drag:', e);
            }
        }


        // Fallback for standalone testing
        setTimeout(() => {
            if (tools.length === 0) {
                console.log('[FloatingToolbar] Using default tools');
                setTools(getDefaultTools());
            }
        }, 1000);

        function getDefaultTools() {
            return [
                { name: 'Maya', path: 'maya.exe', icon: null, color: 1 },
                { name: '3ds Max', path: '3dsmax.exe', icon: null, color: 2 },
                { name: 'Houdini', path: 'houdini.exe', icon: null, color: 3 },
                { name: 'Blender', path: 'blender.exe', icon: null, color: 4 },
                { name: 'Photoshop', path: 'photoshop.exe', icon: null, color: 5 },
                { name: 'VS Code', path: 'code.exe', icon: null, color: 6 },
            ];
        }

        async function loadTools() {
            try {
                if (window.auroraview && window.auroraview.call) {
                    const result = await window.auroraview.call('get_tools');
                    if (result && result.tools) {
                        setTools(result.tools);
                    }
                }
            } catch (e) {
                console.error('[FloatingToolbar] Failed to load tools:', e);
                setTools(getDefaultTools());
            }
        }

        function setTools(newTools) {
            tools = newTools;
            renderTools();
        }

        function renderTools() {
            const container = document.getElementById('toolbarInner');
            container.innerHTML = '';

            tools.forEach((tool, index) => {
                const item = document.createElement('div');
                item.className = 'tool-item no-drag';
                item.setAttribute('data-tooltip', tool.name);
                item.onclick = () => launchTool(tool);

                if (tool.icon) {
                    const img = document.createElement('img');
                    img.src = tool.icon;
                    img.alt = tool.name;
                    item.appendChild(img);
                } else {
                    const placeholder = document.createElement('div');
                    placeholder.className = `icon-placeholder color-${(tool.color || (index % 6)) + 1}`;
                    placeholder.textContent = tool.name.substring(0, 2).toUpperCase();
                    item.appendChild(placeholder);
                }

                container.appendChild(item);
            });

            // Add separator
            const separator = document.createElement('div');
            separator.className = 'separator';
            container.appendChild(separator);

            // Add "add tool" button
            const addBtn = document.createElement('div');
            addBtn.className = 'add-tool no-drag';
            addBtn.setAttribute('data-tooltip', 'Add Tool');
            addBtn.onclick = addTool;
            addBtn.innerHTML = '<svg viewBox="0 0 24 24"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/></svg>';
            container.appendChild(addBtn);
        }

        function toggleToolbar() {
            isExpanded = !isExpanded;
            const btn = document.getElementById('triggerBtn');
            const toolbar = document.getElementById('toolbar');
            const items = document.querySelectorAll('.tool-item, .add-tool, .separator');

            if (isExpanded) {
                btn.classList.add('expanded');

                // Animate toolbar expansion
                gsap.fromTo(toolbar,
                    { width: 0, opacity: 0 },
                    { width: 'auto', opacity: 1, duration: 0.4, ease: 'power3.out' }
                );

                // Stagger animate items
                gsap.to(items, {
                    opacity: 1,
                    scale: 1,
                    duration: 0.3,
                    stagger: 0.05,
                    delay: 0.1,
                    ease: 'back.out(1.7)'
                });

                // Notify backend about expansion for window resize
                if (window.auroraview && window.auroraview.call) {
                    window.auroraview.call('on_expand', { expanded: true });
                }
            } else {
                btn.classList.remove('expanded');

                // Animate items out
                gsap.to(items, {
                    opacity: 0,
                    scale: 0.5,
                    duration: 0.2,
                    stagger: 0.02,
                    ease: 'power2.in'
                });

                // Collapse toolbar
                gsap.to(toolbar, {
                    width: 0,
                    opacity: 0,
                    duration: 0.3,
                    delay: 0.1,
                    ease: 'power3.in'
                });

                // Notify backend
                if (window.auroraview && window.auroraview.call) {
                    window.auroraview.call('on_expand', { expanded: false });
                }
            }
        }

        async function launchTool(tool) {
            console.log('[FloatingToolbar] Launching:', tool.name);

            // Click animation
            const items = document.querySelectorAll('.tool-item');
            items.forEach(item => {
                if (item.getAttribute('data-tooltip') === tool.name) {
                    gsap.to(item, {
                        scale: 0.8,
                        duration: 0.1,
                        yoyo: true,
                        repeat: 1,
                        ease: 'power2.inOut'
                    });
                }
            });

            try {
                if (window.auroraview && window.auroraview.call) {
                    await window.auroraview.call('launch_tool', { path: tool.path, name: tool.name });
                }
            } catch (e) {
                console.error('[FloatingToolbar] Failed to launch tool:', e);
            }
        }

        async function addTool() {
            console.log('[FloatingToolbar] Add tool clicked');
            try {
                if (window.auroraview && window.auroraview.call) {
                    await window.auroraview.call('add_tool');
                }
            } catch (e) {
                console.error('[FloatingToolbar] Failed to add tool:', e);
            }
        }

        // Initial render
        renderTools();

        // Subscribe to tool updates
        if (window.auroraview && window.auroraview.on) {
            window.auroraview.on('tools_updated', (data) => {
                if (data && data.tools) {
                    setTools(data.tools);
                }
            });
        }
    </script>
</body>
</html>
"""


def get_installed_applications():
    """Discover installed applications from common locations.

    Returns:
        list: List of tool dictionaries with name, path, and icon info.
    """
    tools = []
    color_index = 0

    # Common DCC application paths on Windows
    dcc_apps = [
        {
            "name": "Maya",
            "paths": [
                r"C:\Program Files\Autodesk\Maya*\bin\maya.exe",
            ],
        },
        {
            "name": "3ds Max",
            "paths": [
                r"C:\Program Files\Autodesk\3ds Max*\3dsmax.exe",
            ],
        },
        {
            "name": "Houdini",
            "paths": [
                r"C:\Program Files\Side Effects Software\Houdini*\bin\houdini.exe",
            ],
        },
        {
            "name": "Blender",
            "paths": [
                r"C:\Program Files\Blender Foundation\Blender*\blender.exe",
                r"C:\Program Files\Blender\blender.exe",
            ],
        },
        {
            "name": "Photoshop",
            "paths": [
                r"C:\Program Files\Adobe\Adobe Photoshop*\Photoshop.exe",
            ],
        },
        {
            "name": "VS Code",
            "paths": [
                r"C:\Users\*\AppData\Local\Programs\Microsoft VS Code\Code.exe",
                r"C:\Program Files\Microsoft VS Code\Code.exe",
            ],
        },
        {
            "name": "Nuke",
            "paths": [
                r"C:\Program Files\Nuke*\Nuke*.exe",
            ],
        },
        {
            "name": "Substance Painter",
            "paths": [
                r"C:\Program Files\Adobe\Adobe Substance 3D Painter\Adobe Substance 3D Painter.exe",
            ],
        },
    ]

    import glob

    for app in dcc_apps:
        for pattern in app["paths"]:
            matches = glob.glob(pattern)
            if matches:
                # Use the first match (usually the latest version)
                path = matches[-1]  # Latest version typically has highest number
                tools.append(
                    {
                        "name": app["name"],
                        "path": path,
                        "icon": None,  # Could extract icon from exe
                        "color": color_index % 6,
                    }
                )
                color_index += 1
                break

    return tools


def run_floating_toolbar_demo():
    """Run the floating toolbar demo."""
    from auroraview import AuroraView

    # Discover installed applications
    discovered_tools = get_installed_applications()
    if not discovered_tools:
        # Fallback to placeholder tools
        discovered_tools = [
            {"name": "Maya", "path": "maya.exe", "icon": None, "color": 0},
            {"name": "3ds Max", "path": "3dsmax.exe", "icon": None, "color": 1},
            {"name": "Houdini", "path": "houdini.exe", "icon": None, "color": 2},
            {"name": "Blender", "path": "blender.exe", "icon": None, "color": 3},
            {"name": "Photoshop", "path": "photoshop.exe", "icon": None, "color": 4},
            {"name": "VS Code", "path": "code.exe", "icon": None, "color": 5},
        ]

    print(f"[FloatingToolbar] Discovered {len(discovered_tools)} tools:")
    for tool in discovered_tools:
        print(f"  - {tool['name']}: {tool['path']}")

    class FloatingToolbar(AuroraView):
        """Floating toolbar with expandable tool shelf."""

        def __init__(self):
            # Start with collapsed size
            super().__init__(
                html=TOOLBAR_HTML,
                width=64,  # Just the trigger button
                height=64,
                frame=False,
                transparent=True,
                undecorated_shadow=False,  # No shadow for truly transparent
                always_on_top=True,
                tool_window=True,
            )
            self.tools = discovered_tools
            self.is_expanded = False

            # Bind API methods
            self.bind_call("get_tools", self.get_tools)
            self.bind_call("launch_tool", self.launch_tool)
            self.bind_call("add_tool", self.add_tool)
            self.bind_call("on_expand", self.on_expand)

        def get_tools(self, *args, **kwargs):
            """Return the list of available tools."""
            return {"tools": self.tools}

        def launch_tool(self, path: str = "", name: str = ""):
            """Launch a tool by its path."""
            print(f"[FloatingToolbar] Launching: {name} ({path})")

            if not path or not os.path.exists(path):
                print(f"[FloatingToolbar] Tool not found: {path}")
                return {"ok": False, "error": f"Tool not found: {path}"}

            try:
                # Launch the application
                if sys.platform == "win32":
                    os.startfile(path)
                else:
                    subprocess.Popen([path], start_new_session=True)
                return {"ok": True}
            except Exception as e:
                print(f"[FloatingToolbar] Failed to launch: {e}")
                return {"ok": False, "error": str(e)}

        def add_tool(self, *args, **kwargs):
            """Open file dialog to add a new tool."""
            print("[FloatingToolbar] Add tool requested")
            # In a real implementation, this would open a file dialog
            # For now, just log the request
            return {"ok": True, "message": "Add tool dialog would open here"}

        def on_expand(self, expanded: bool = False):
            """Handle toolbar expansion/collapse."""
            self.is_expanded = expanded
            print(f"[FloatingToolbar] Expanded: {expanded}")

            # Resize window based on expansion state
            if expanded:
                # Calculate width based on number of tools
                # Each tool is 40px + 8px gap, plus padding
                tool_count = len(self.tools) + 1  # +1 for add button
                toolbar_width = tool_count * 48 + 32 + 12  # items + padding + gap
                new_width = 64 + toolbar_width  # trigger + toolbar
                self.set_size(new_width, 64)
            else:
                self.set_size(64, 64)

    print("\n" + "=" * 60)
    print("Floating Toolbar Demo")
    print("=" * 60)
    print("\nFeatures:")
    print("  - Click the + button to expand/collapse the toolbar")
    print("  - Click a tool icon to launch the application")
    print("  - Drag the toolbar to reposition")
    print("  - The toolbar auto-discovers installed DCC applications")
    print("\nPress Ctrl+C to exit.")
    print("=" * 60 + "\n")

    toolbar = FloatingToolbar()
    toolbar.show()


if __name__ == "__main__":
    run_floating_toolbar_demo()
