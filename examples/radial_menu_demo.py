#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Radial Menu Demo - Circular floating menu with GSAP animations.

This example demonstrates how to create a radial/pie menu that expands
from a central button with smooth GSAP animations.

Features demonstrated:
- Transparent, frameless circular window
- Radial menu layout with items arranged in a circle
- GSAP-powered animations (elastic, spring effects)
- Hover effects with magnetic cursor
- Sub-menu support
- Tool window style (hide from taskbar/Alt+Tab)

Use cases:
- Quick action menu in DCC applications
- Context menu replacement
- Tool palette with categories
- Marking menu style interface

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

# HTML for the radial menu with GSAP animations
RADIAL_MENU_HTML = """
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
            align-items: center;
            justify-content: center;
            position: relative;
        }

        /* Center button */
        .center-btn {
            width: 56px;
            height: 56px;
            border-radius: 50%;
            background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
            border: none;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            box-shadow: 0 4px 20px rgba(99, 102, 241, 0.5);
            position: relative;
            z-index: 100;
            transition: transform 0.2s;
        }

        .center-btn:hover {
            transform: scale(1.1);
        }

        .center-btn:active {
            transform: scale(0.95);
        }

        .center-btn svg {
            width: 28px;
            height: 28px;
            fill: white;
            transition: transform 0.3s ease;
        }

        .center-btn.expanded svg {
            transform: rotate(45deg);
        }

        /* Radial menu items */
        .menu-item {
            position: absolute;
            width: 48px;
            height: 48px;
            border-radius: 50%;
            background: rgba(30, 30, 46, 0.95);
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255, 255, 255, 0.1);
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            opacity: 0;
            transform: scale(0);
            box-shadow: 0 4px 15px rgba(0, 0, 0, 0.3);
            transition: background 0.2s, border-color 0.2s;
        }

        .menu-item:hover {
            background: rgba(99, 102, 241, 0.3);
            border-color: #6366f1;
        }

        .menu-item svg {
            width: 22px;
            height: 22px;
            fill: white;
        }

        .menu-item .icon-text {
            font-size: 14px;
            font-weight: 600;
            color: white;
        }

        /* Tooltip */
        .menu-item::after {
            content: attr(data-tooltip);
            position: absolute;
            white-space: nowrap;
            background: rgba(0, 0, 0, 0.9);
            color: white;
            padding: 6px 12px;
            border-radius: 6px;
            font-size: 12px;
            opacity: 0;
            pointer-events: none;
            transition: opacity 0.2s;
            z-index: 1000;
        }

        /* Position tooltips based on item position */
        .menu-item.top::after { bottom: 100%; margin-bottom: 8px; left: 50%; transform: translateX(-50%); }
        .menu-item.bottom::after { top: 100%; margin-top: 8px; left: 50%; transform: translateX(-50%); }
        .menu-item.left::after { right: 100%; margin-right: 8px; top: 50%; transform: translateY(-50%); }
        .menu-item.right::after { left: 100%; margin-left: 8px; top: 50%; transform: translateY(-50%); }

        .menu-item:hover::after {
            opacity: 1;
        }

        /* Ring decoration */
        .ring {
            position: absolute;
            border-radius: 50%;
            border: 1px solid rgba(99, 102, 241, 0.2);
            pointer-events: none;
            opacity: 0;
        }

        .ring-1 { width: 120px; height: 120px; }
        .ring-2 { width: 180px; height: 180px; }
        .ring-3 { width: 240px; height: 240px; }

        /* Particle effects */
        .particle {
            position: absolute;
            width: 4px;
            height: 4px;
            border-radius: 50%;
            background: rgba(99, 102, 241, 0.6);
            pointer-events: none;
        }

        /* Color variants for menu items */
        .menu-item.color-1 { background: linear-gradient(135deg, rgba(244, 63, 94, 0.2), rgba(236, 72, 153, 0.2)); }
        .menu-item.color-2 { background: linear-gradient(135deg, rgba(249, 115, 22, 0.2), rgba(245, 158, 11, 0.2)); }
        .menu-item.color-3 { background: linear-gradient(135deg, rgba(34, 197, 94, 0.2), rgba(16, 185, 129, 0.2)); }
        .menu-item.color-4 { background: linear-gradient(135deg, rgba(59, 130, 246, 0.2), rgba(99, 102, 241, 0.2)); }
        .menu-item.color-5 { background: linear-gradient(135deg, rgba(139, 92, 246, 0.2), rgba(168, 85, 247, 0.2)); }
        .menu-item.color-6 { background: linear-gradient(135deg, rgba(6, 182, 212, 0.2), rgba(14, 165, 233, 0.2)); }

        .menu-item.color-1:hover { background: linear-gradient(135deg, rgba(244, 63, 94, 0.4), rgba(236, 72, 153, 0.4)); border-color: #f43f5e; }
        .menu-item.color-2:hover { background: linear-gradient(135deg, rgba(249, 115, 22, 0.4), rgba(245, 158, 11, 0.4)); border-color: #f97316; }
        .menu-item.color-3:hover { background: linear-gradient(135deg, rgba(34, 197, 94, 0.4), rgba(16, 185, 129, 0.4)); border-color: #22c55e; }
        .menu-item.color-4:hover { background: linear-gradient(135deg, rgba(59, 130, 246, 0.4), rgba(99, 102, 241, 0.4)); border-color: #3b82f6; }
        .menu-item.color-5:hover { background: linear-gradient(135deg, rgba(139, 92, 246, 0.4), rgba(168, 85, 247, 0.4)); border-color: #8b5cf6; }
        .menu-item.color-6:hover { background: linear-gradient(135deg, rgba(6, 182, 212, 0.4), rgba(14, 165, 233, 0.4)); border-color: #06b6d4; }

        /* Drag handle for frameless window */
        .drag-area {
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            -webkit-app-region: drag;
        }

        .no-drag {
            -webkit-app-region: no-drag;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="drag-area"></div>

        <!-- Decorative rings -->
        <div class="ring ring-1"></div>
        <div class="ring ring-2"></div>
        <div class="ring ring-3"></div>

        <!-- Menu items container -->
        <div id="menuItems"></div>

        <!-- Center button -->
        <button class="center-btn no-drag" id="centerBtn" onclick="toggleMenu()">
            <svg viewBox="0 0 24 24">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
            </svg>
        </button>
    </div>

    <script>
        let isExpanded = false;
        const RADIUS = 85; // Distance from center to menu items
        const menuItems = [
            { id: 'maya', name: 'Maya', icon: 'M', color: 1 },
            { id: 'max', name: '3ds Max', icon: '3D', color: 2 },
            { id: 'houdini', name: 'Houdini', icon: 'H', color: 3 },
            { id: 'blender', name: 'Blender', icon: 'B', color: 4 },
            { id: 'photoshop', name: 'Photoshop', icon: 'Ps', color: 5 },
            { id: 'vscode', name: 'VS Code', icon: '<>', color: 6 },
        ];

        // Create menu items
        function createMenuItems() {
            const container = document.getElementById('menuItems');
            container.innerHTML = '';

            const itemCount = menuItems.length;
            const angleStep = (2 * Math.PI) / itemCount;
            const startAngle = -Math.PI / 2; // Start from top

            menuItems.forEach((item, index) => {
                const angle = startAngle + index * angleStep;
                const x = Math.cos(angle) * RADIUS;
                const y = Math.sin(angle) * RADIUS;

                const element = document.createElement('div');
                element.className = `menu-item no-drag color-${item.color}`;
                element.setAttribute('data-tooltip', item.name);
                element.setAttribute('data-id', item.id);

                // Add position class for tooltip
                if (y < -20) element.classList.add('top');
                else if (y > 20) element.classList.add('bottom');
                else if (x < 0) element.classList.add('left');
                else element.classList.add('right');

                element.innerHTML = `<span class="icon-text">${item.icon}</span>`;
                element.style.left = `calc(50% + ${x}px - 24px)`;
                element.style.top = `calc(50% + ${y}px - 24px)`;

                element.onclick = () => handleItemClick(item);

                // Add hover effect with GSAP
                element.onmouseenter = () => {
                    gsap.to(element, {
                        scale: 1.2,
                        duration: 0.3,
                        ease: 'back.out(1.7)'
                    });
                };

                element.onmouseleave = () => {
                    gsap.to(element, {
                        scale: 1,
                        duration: 0.3,
                        ease: 'power2.out'
                    });
                };

                container.appendChild(element);
            });
        }

        function toggleMenu() {
            isExpanded = !isExpanded;
            const btn = document.getElementById('centerBtn');
            const items = document.querySelectorAll('.menu-item');
            const rings = document.querySelectorAll('.ring');

            if (isExpanded) {
                btn.classList.add('expanded');

                // Animate rings
                rings.forEach((ring, i) => {
                    gsap.to(ring, {
                        opacity: 1,
                        scale: 1,
                        duration: 0.5,
                        delay: i * 0.1,
                        ease: 'power2.out'
                    });
                });

                // Animate menu items with stagger
                const itemCount = items.length;
                items.forEach((item, index) => {
                    const angle = -Math.PI / 2 + index * (2 * Math.PI / itemCount);
                    const x = Math.cos(angle) * RADIUS;
                    const y = Math.sin(angle) * RADIUS;

                    // Start from center
                    gsap.fromTo(item,
                        {
                            opacity: 0,
                            scale: 0,
                            x: -x,
                            y: -y
                        },
                        {
                            opacity: 1,
                            scale: 1,
                            x: 0,
                            y: 0,
                            duration: 0.5,
                            delay: index * 0.05,
                            ease: 'elastic.out(1, 0.5)'
                        }
                    );
                });

                // Create particle burst
                createParticleBurst();

                // Notify backend
                if (window.auroraview && window.auroraview.call) {
                    window.auroraview.call('on_expand', { expanded: true });
                }
            } else {
                btn.classList.remove('expanded');

                // Animate rings out
                rings.forEach((ring, i) => {
                    gsap.to(ring, {
                        opacity: 0,
                        scale: 0.8,
                        duration: 0.3,
                        delay: (rings.length - i - 1) * 0.05,
                        ease: 'power2.in'
                    });
                });

                // Animate menu items back to center
                const itemCount = items.length;
                items.forEach((item, index) => {
                    const angle = -Math.PI / 2 + index * (2 * Math.PI / itemCount);
                    const x = Math.cos(angle) * RADIUS;
                    const y = Math.sin(angle) * RADIUS;

                    gsap.to(item, {
                        opacity: 0,
                        scale: 0,
                        x: -x,
                        y: -y,
                        duration: 0.3,
                        delay: (items.length - index - 1) * 0.03,
                        ease: 'power2.in'
                    });
                });

                // Notify backend
                if (window.auroraview && window.auroraview.call) {
                    window.auroraview.call('on_expand', { expanded: false });
                }
            }
        }

        function createParticleBurst() {
            const container = document.querySelector('.container');
            const particleCount = 12;

            for (let i = 0; i < particleCount; i++) {
                const particle = document.createElement('div');
                particle.className = 'particle';
                particle.style.left = '50%';
                particle.style.top = '50%';
                container.appendChild(particle);

                const angle = (i / particleCount) * Math.PI * 2;
                const distance = 80 + Math.random() * 40;

                gsap.to(particle, {
                    x: Math.cos(angle) * distance,
                    y: Math.sin(angle) * distance,
                    opacity: 0,
                    duration: 0.8,
                    ease: 'power2.out',
                    onComplete: () => particle.remove()
                });
            }
        }

        async function handleItemClick(item) {
            console.log('[RadialMenu] Clicked:', item.name);

            // Click animation
            const element = document.querySelector(`[data-id="${item.id}"]`);
            if (element) {
                gsap.to(element, {
                    scale: 0.8,
                    duration: 0.1,
                    yoyo: true,
                    repeat: 1,
                    ease: 'power2.inOut'
                });
            }

            try {
                if (window.auroraview && window.auroraview.call) {
                    await window.auroraview.call('on_item_click', { id: item.id, name: item.name });
                }
            } catch (e) {
                console.error('[RadialMenu] Error:', e);
            }
        }

        // Initialize
        createMenuItems();

        // Listen for tool updates
        window.addEventListener('auroraviewready', () => {
            console.log('[RadialMenu] AuroraView ready');
            if (window.auroraview && window.auroraview.on) {
                window.auroraview.on('update_items', (data) => {
                    if (data && data.items) {
                        menuItems.length = 0;
                        menuItems.push(...data.items);
                        createMenuItems();
                    }
                });
            }
        });
    </script>
</body>
</html>
"""


def run_radial_menu_demo():
    """Run the radial menu demo."""
    from auroraview import AuroraView

    class RadialMenu(AuroraView):
        """Radial menu with circular tool layout."""

        def __init__(self):
            # Size to accommodate the expanded menu
            super().__init__(
                html=RADIAL_MENU_HTML,
                width=280,
                height=280,
                frame=False,
                transparent=True,
                undecorated_shadow=False,
                always_on_top=True,
                tool_window=True,
            )
            self.is_expanded = False

            # Bind API methods
            self.bind_call("on_expand", self.on_expand)
            self.bind_call("on_item_click", self.on_item_click)

        def on_expand(self, expanded: bool = False):
            """Handle menu expansion/collapse."""
            self.is_expanded = expanded
            print(f"[RadialMenu] Expanded: {expanded}")

        def on_item_click(self, id: str = "", name: str = ""):
            """Handle menu item click."""
            print(f"[RadialMenu] Item clicked: {name} (id: {id})")

            # Here you would launch the corresponding application
            # For demo purposes, just log the action
            return {"ok": True, "message": f"Clicked: {name}"}

    print("\n" + "=" * 60)
    print("Radial Menu Demo")
    print("=" * 60)
    print("\nFeatures:")
    print("  - Click the center button to expand/collapse")
    print("  - Hover over items for tooltips")
    print("  - Click items to trigger actions")
    print("  - Smooth GSAP animations with elastic effects")
    print("\nPress Ctrl+C to exit.")
    print("=" * 60 + "\n")

    menu = RadialMenu()
    menu.show()


if __name__ == "__main__":
    run_radial_menu_demo()
