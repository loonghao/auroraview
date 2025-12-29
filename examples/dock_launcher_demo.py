#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Dock Launcher Demo - macOS-style dock with GSAP animations.

This example demonstrates how to create a dock-style launcher that displays
local application shortcuts with magnification and smooth animations.

Features demonstrated:
- Transparent, frameless dock window
- macOS-style magnification on hover
- GSAP-powered animations
- Dynamic tool discovery from system
- Drag to reposition
- Auto-hide behavior
- Running indicator dots

Use cases:
- Application launcher dock
- Quick access toolbar
- Favorite tools palette
- DCC application switcher

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import glob
import os
import subprocess
import sys

# HTML for the dock launcher with GSAP animations
DOCK_HTML = """
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
            overflow: hidden; /* prevent Chromium scrollbars in frameless windows */
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        /* Hide scrollbars defensively (WebView2/Chromium) */
        ::-webkit-scrollbar {
            width: 0;
            height: 0;
        }

        .container {
            width: 100%;
            height: 100%;
            display: flex;
            align-items: flex-end;
            justify-content: center;
            padding-bottom: 8px;
        }

        /* Visual effects (shadow/reflection) are OFF by default.
           Enable by opening the page with `?shadows=1`. */
        .enable-shadows .container {
            /* Leave room for the reflection pseudo-element */
            padding-bottom: 28px;
        }



        /* Dock bar */
        .dock {
            position: relative; /* anchor ::after reflection */
            display: flex;
            align-items: flex-end;
            justify-content: center;
            gap: 4px;
            padding: 8px 16px;
            background: rgba(30, 30, 46, 0.85);
            backdrop-filter: blur(20px);
            border-radius: 18px;
            border: 1px solid rgba(255, 255, 255, 0.1);
            box-shadow: none; /* default: no obvious shadow */
            -webkit-app-region: drag;
        }

        .enable-shadows .dock {
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        }



        /* Dock item */
        .dock-item {
            display: flex;
            flex-direction: column;
            align-items: center;
            cursor: pointer;
            -webkit-app-region: no-drag;
            position: relative;
        }

        .dock-icon {
            width: 48px;
            height: 48px;
            border-radius: 12px;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: transform 0.1s ease-out;
            position: relative;
            overflow: hidden;
        }

        .dock-icon::before {
            content: '';
            position: absolute;
            inset: 0;
            background: linear-gradient(135deg, rgba(255,255,255,0.2) 0%, transparent 50%);
            border-radius: 12px;
            opacity: 0;
            transition: opacity 0.2s;
        }

        .dock-item:hover .dock-icon::before {
            opacity: 1;
        }

        .dock-icon img {
            width: 100%;
            height: 100%;
            object-fit: contain;
            border-radius: 10px;
        }

        .dock-icon .icon-placeholder {
            width: 100%;
            height: 100%;
            border-radius: 10px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 18px;
            font-weight: 700;
            color: white;
            text-shadow: 0 1px 2px rgba(0,0,0,0.3);
        }

        /* Tooltip */
        .tooltip {
            position: absolute;
            bottom: 100%;
            left: 50%;
            transform: translateX(-50%);
            background: rgba(0, 0, 0, 0.9);
            color: white;
            padding: 6px 12px;
            border-radius: 6px;
            font-size: 12px;
            white-space: nowrap;
            opacity: 0;
            pointer-events: none;
            margin-bottom: 8px;
            transition: opacity 0.15s;
        }

        .dock-item:hover .tooltip {
            opacity: 1;
        }

        /* Running indicator */
        .running-dot {
            width: 4px;
            height: 4px;
            border-radius: 50%;
            background: rgba(255, 255, 255, 0.8);
            margin-top: 4px;
            opacity: 0;
        }

        .dock-item.running .running-dot {
            opacity: 1;
        }

        /* Separator */
        .dock-separator {
            width: 1px;
            height: 48px;
            background: rgba(255, 255, 255, 0.15);
            margin: 0 8px;
            align-self: center;
        }

        /* Color palette */
        .color-1 { background: linear-gradient(135deg, #f43f5e, #ec4899); }
        .color-2 { background: linear-gradient(135deg, #f97316, #f59e0b); }
        .color-3 { background: linear-gradient(135deg, #22c55e, #10b981); }
        .color-4 { background: linear-gradient(135deg, #3b82f6, #6366f1); }
        .color-5 { background: linear-gradient(135deg, #8b5cf6, #a855f7); }
        .color-6 { background: linear-gradient(135deg, #06b6d4, #0ea5e9); }
        .color-7 { background: linear-gradient(135deg, #ef4444, #dc2626); }
        .color-8 { background: linear-gradient(135deg, #84cc16, #65a30d); }

        /* Bounce animation for click */
        @keyframes bounce {
            0%, 100% { transform: translateY(0); }
            50% { transform: translateY(-15px); }
        }

        .dock-item.bouncing .dock-icon {
            animation: bounce 0.5s ease;
        }

        /* Reflection effect (OFF by default) */
        .dock::after {
            content: none;
        }

        .enable-shadows .dock::after {
            content: '';
            position: absolute;
            left: 16px;
            right: 16px;
            bottom: -20px;
            height: 20px;
            background: linear-gradient(to bottom, rgba(255,255,255,0.1), transparent);
            border-radius: 0 0 18px 18px;
            pointer-events: none;
            opacity: 0.5;
        }

    </style>
</head>
<body>
    <div class="container">
        <div class="dock" id="dock">
            <!-- Items will be inserted here -->
        </div>
    </div>

    <script>
        const MAGNIFICATION = 1.5;
        const MAGNIFICATION_RANGE = 100; // pixels
        let items = [];
        let dockRect = null;

        // Visual effects toggle (OFF by default)
        const ENABLE_VISUAL_SHADOWS = (new URLSearchParams(window.location.search)).get('shadows') === '1';
        if (ENABLE_VISUAL_SHADOWS) {
            document.body.classList.add('enable-shadows');
        }

        // Initialize
        window.addEventListener('auroraviewready', () => {
            console.log('[DockLauncher] AuroraView ready');
            loadItems();
        });


        // Fallback for standalone testing
        setTimeout(() => {
            if (items.length === 0) {
                setItems(getDefaultItems());
            }
        }, 1000);

        function getDefaultItems() {
            return [
                { id: 'maya', name: 'Maya', icon: null, color: 1, running: false },
                { id: 'max', name: '3ds Max', icon: null, color: 2, running: false },
                { id: 'houdini', name: 'Houdini', icon: null, color: 3, running: true },
                { id: 'blender', name: 'Blender', icon: null, color: 4, running: false },
                { id: 'separator', type: 'separator' },
                { id: 'photoshop', name: 'Photoshop', icon: null, color: 5, running: true },
                { id: 'substance', name: 'Substance', icon: null, color: 6, running: false },
                { id: 'separator2', type: 'separator' },
                { id: 'vscode', name: 'VS Code', icon: null, color: 7, running: true },
                { id: 'terminal', name: 'Terminal', icon: null, color: 8, running: false },
            ];
        }

        async function loadItems() {
            try {
                if (window.auroraview && window.auroraview.call) {
                    const result = await window.auroraview.call('get_items');
                    if (result && result.items) {
                        setItems(result.items);
                    }
                }
            } catch (e) {
                console.error('[DockLauncher] Failed to load items:', e);
                setItems(getDefaultItems());
            }
        }

        function setItems(newItems) {
            items = newItems;
            renderDock();
        }

        function renderDock() {
            const dock = document.getElementById('dock');
            dock.innerHTML = '';

            items.forEach((item, index) => {
                if (item.type === 'separator') {
                    const sep = document.createElement('div');
                    sep.className = 'dock-separator';
                    dock.appendChild(sep);
                    return;
                }

                const element = document.createElement('div');
                element.className = `dock-item${item.running ? ' running' : ''}`;
                element.setAttribute('data-index', index);

                const iconDiv = document.createElement('div');
                iconDiv.className = 'dock-icon';

                if (item.icon) {
                    const img = document.createElement('img');
                    img.src = item.icon;
                    img.alt = item.name;
                    iconDiv.appendChild(img);
                } else {
                    const placeholder = document.createElement('div');
                    placeholder.className = `icon-placeholder color-${item.color || ((index % 8) + 1)}`;
                    placeholder.textContent = item.name.substring(0, 2).toUpperCase();
                    iconDiv.appendChild(placeholder);
                }

                const tooltip = document.createElement('div');
                tooltip.className = 'tooltip';
                tooltip.textContent = item.name;

                const dot = document.createElement('div');
                dot.className = 'running-dot';

                element.appendChild(tooltip);
                element.appendChild(iconDiv);
                element.appendChild(dot);

                element.onclick = () => handleClick(item, element);

                dock.appendChild(element);
            });

            // Update dock rect for magnification
            dockRect = dock.getBoundingClientRect();
        }

        // Magnification effect
        document.addEventListener('mousemove', (e) => {
            const dock = document.getElementById('dock');
            if (!dock) return;

            const dockItems = dock.querySelectorAll('.dock-item');
            const dockRect = dock.getBoundingClientRect();

            // Check if mouse is near the dock
            const mouseY = e.clientY;
            const dockTop = dockRect.top;
            const dockBottom = dockRect.bottom;

            if (mouseY < dockTop - MAGNIFICATION_RANGE || mouseY > dockBottom + 20) {
                // Reset all items
                dockItems.forEach(item => {
                    const icon = item.querySelector('.dock-icon');
                    gsap.to(icon, {
                        scale: 1,
                        y: 0,
                        duration: 0.2,
                        ease: 'power2.out'
                    });
                });
                return;
            }

            dockItems.forEach(item => {
                const icon = item.querySelector('.dock-icon');
                const itemRect = item.getBoundingClientRect();
                const itemCenterX = itemRect.left + itemRect.width / 2;

                const distance = Math.abs(e.clientX - itemCenterX);
                const scale = Math.max(1, MAGNIFICATION - (distance / MAGNIFICATION_RANGE) * (MAGNIFICATION - 1));
                const yOffset = (scale - 1) * -20;

                gsap.to(icon, {
                    scale: scale,
                    y: yOffset,
                    duration: 0.1,
                    ease: 'power2.out'
                });
            });
        });

        async function handleClick(item, element) {
            console.log('[DockLauncher] Clicked:', item.name);

            // Bounce animation
            element.classList.add('bouncing');
            setTimeout(() => element.classList.remove('bouncing'), 500);

            try {
                if (window.auroraview && window.auroraview.call) {
                    await window.auroraview.call('launch_item', { id: item.id, name: item.name });
                }
            } catch (e) {
                console.error('[DockLauncher] Error:', e);
            }
        }

        // Initial render
        renderDock();

        // Listen for updates
        if (window.auroraview && window.auroraview.on) {
            window.auroraview.on('items_updated', (data) => {
                if (data && data.items) {
                    setItems(data.items);
                }
            });

            window.auroraview.on('item_running', (data) => {
                const item = items.find(i => i.id === data.id);
                if (item) {
                    item.running = data.running;
                    renderDock();
                }
            });
        }
    </script>
</body>
</html>
"""


def discover_applications():
    """Discover installed applications from common locations.

    Returns:
        list: List of application dictionaries.
    """
    apps = []
    color_index = 0

    # Common application paths on Windows
    app_definitions = [
        {"name": "Maya", "patterns": [r"C:\Program Files\Autodesk\Maya*\bin\maya.exe"]},
        {"name": "3ds Max", "patterns": [r"C:\Program Files\Autodesk\3ds Max*\3dsmax.exe"]},
        {
            "name": "Houdini",
            "patterns": [r"C:\Program Files\Side Effects Software\Houdini*\bin\houdini.exe"],
        },
        {
            "name": "Blender",
            "patterns": [
                r"C:\Program Files\Blender Foundation\Blender*\blender.exe",
                r"C:\Program Files\Blender\blender.exe",
            ],
        },
        {
            "name": "Photoshop",
            "patterns": [r"C:\Program Files\Adobe\Adobe Photoshop*\Photoshop.exe"],
        },
        {
            "name": "Substance Painter",
            "patterns": [
                r"C:\Program Files\Adobe\Adobe Substance 3D Painter\Adobe Substance 3D Painter.exe"
            ],
        },
        {"name": "Nuke", "patterns": [r"C:\Program Files\Nuke*\Nuke*.exe"]},
        {
            "name": "VS Code",
            "patterns": [
                r"C:\Users\*\AppData\Local\Programs\Microsoft VS Code\Code.exe",
                r"C:\Program Files\Microsoft VS Code\Code.exe",
            ],
        },
        {
            "name": "Terminal",
            "patterns": [
                r"C:\Users\*\AppData\Local\Microsoft\WindowsApps\wt.exe",
                r"C:\Windows\System32\cmd.exe",
            ],
        },
    ]

    for app_def in app_definitions:
        for pattern in app_def["patterns"]:
            matches = glob.glob(pattern)
            if matches:
                path = matches[-1]
                apps.append(
                    {
                        "id": app_def["name"].lower().replace(" ", "_"),
                        "name": app_def["name"],
                        "path": path,
                        "icon": None,
                        "color": (color_index % 8) + 1,
                        "running": False,
                    }
                )
                color_index += 1
                break

    return apps


def run_dock_launcher_demo():
    """Run the dock launcher demo."""
    from auroraview import AuroraView

    # Discover installed applications
    discovered_apps = discover_applications()

    # Add separators
    items = []
    dcc_apps = ["maya", "3ds_max", "houdini", "blender", "nuke"]
    adobe_apps = ["photoshop", "substance_painter"]
    dev_apps = ["vs_code", "terminal"]

    for app in discovered_apps:
        if app["id"] in dcc_apps:
            items.append(app)

    if items:
        items.append({"id": "sep1", "type": "separator"})

    for app in discovered_apps:
        if app["id"] in adobe_apps:
            items.append(app)

    if any(app["id"] in adobe_apps for app in discovered_apps):
        items.append({"id": "sep2", "type": "separator"})

    for app in discovered_apps:
        if app["id"] in dev_apps:
            items.append(app)

    # Fallback if no apps found
    if not items:
        items = [
            {"id": "maya", "name": "Maya", "icon": None, "color": 1, "running": False},
            {"id": "max", "name": "3ds Max", "icon": None, "color": 2, "running": False},
            {"id": "houdini", "name": "Houdini", "icon": None, "color": 3, "running": True},
            {"id": "blender", "name": "Blender", "icon": None, "color": 4, "running": False},
            {"id": "sep1", "type": "separator"},
            {"id": "photoshop", "name": "Photoshop", "icon": None, "color": 5, "running": True},
            {"id": "sep2", "type": "separator"},
            {"id": "vscode", "name": "VS Code", "icon": None, "color": 7, "running": True},
        ]

    print(
        f"[DockLauncher] Found {len([i for i in items if i.get('type') != 'separator'])} applications"
    )

    class DockLauncher(AuroraView):
        """macOS-style dock launcher."""

        def __init__(self):
            # Calculate width based on items
            item_count = len([i for i in items if i.get("type") != "separator"])
            sep_count = len([i for i in items if i.get("type") == "separator"])
            width = item_count * 56 + sep_count * 18 + 48  # items + separators + padding

            super().__init__(
                html=DOCK_HTML,
                width=width,
                height=100,
                frame=False,
                transparent=True,
                undecorated_shadow=False,
                always_on_top=True,
                tool_window=True,
            )
            self.items = items

            # Bind API methods
            self.bind_call("get_items", self.get_items)
            self.bind_call("launch_item", self.launch_item)

        def get_items(self, *args, **kwargs):
            """Return the list of dock items."""
            return {"items": self.items}

        def launch_item(self, id: str = "", name: str = ""):
            """Launch an application by ID."""
            print(f"[DockLauncher] Launching: {name}")

            # Find the item
            item = next((i for i in self.items if i.get("id") == id), None)
            if not item or not item.get("path"):
                return {"ok": False, "error": "Application not found"}

            path = item["path"]
            if not os.path.exists(path):
                return {"ok": False, "error": f"Path not found: {path}"}

            try:
                if sys.platform == "win32":
                    os.startfile(path)
                else:
                    subprocess.Popen([path], start_new_session=True)

                # Mark as running
                item["running"] = True
                self.emit("item_running", {"id": id, "running": True})

                return {"ok": True}
            except Exception as e:
                return {"ok": False, "error": str(e)}

    print("\n" + "=" * 60)
    print("Dock Launcher Demo")
    print("=" * 60)
    print("\nFeatures:")
    print("  - macOS-style magnification on hover")
    print("  - Click to launch applications")
    print("  - Running indicator dots")
    print("  - Drag to reposition")
    print("  - Smooth GSAP animations")
    print("\nPress Ctrl+C to exit.")
    print("=" * 60 + "\n")

    dock = DockLauncher()
    dock.show()


if __name__ == "__main__":
    run_dock_launcher_demo()
