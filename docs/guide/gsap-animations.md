# GSAP Animations

AuroraView integrates seamlessly with [GSAP](https://greensock.com/gsap/) (GreenSock Animation Platform) to create smooth, professional animations for your web-based UI.

## Overview

GSAP is a powerful JavaScript animation library that provides:
- Smooth, high-performance animations
- Timeline-based sequencing
- Elastic, bounce, and spring effects
- Cross-browser compatibility

## Gallery Integration

The AuroraView Gallery uses GSAP for various animations:

### Animated Sidebar

```tsx
import gsap from 'gsap';

// Icon hover animation
const handleMouseEnter = () => {
  gsap.to(iconRef.current, {
    scale: 1.15,
    rotate: 5,
    duration: 0.3,
    ease: 'back.out(1.7)',
  });
};

// Active indicator slide
gsap.to(indicator, {
  scaleY: 1,
  opacity: 1,
  duration: 0.3,
  ease: 'power2.out',
});
```

### Card Animations

```tsx
// Entrance animation
gsap.fromTo(card, 
  { opacity: 0, y: 30, scale: 0.95 },
  { opacity: 1, y: 0, scale: 1, duration: 0.6, ease: 'power3.out' }
);

// Hover effect with glow
gsap.to(card, {
  scale: 1.02,
  y: -4,
  duration: 0.3,
  ease: 'power2.out',
});
```

### Page Transitions

```tsx
// Page enter
gsap.fromTo(container,
  { opacity: 0, y: 20 },
  { opacity: 1, y: 0, duration: 0.5, ease: 'power3.out' }
);

// Page exit
gsap.to(container, {
  opacity: 0,
  y: -20,
  duration: 0.3,
  ease: 'power3.in',
});
```

## Floating Toolbar Examples

### Expandable Toolbar

Create a toolbar that expands from a trigger button:

```python
from auroraview import AuroraView

TOOLBAR_HTML = """
<!DOCTYPE html>
<html>
<head>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/gsap.min.js"></script>
    <style>
        .toolbar { display: flex; gap: 8px; }
        .tool-item { opacity: 0; transform: scale(0.5); }
    </style>
</head>
<body>
    <button onclick="toggleToolbar()">+</button>
    <div class="toolbar" id="toolbar"></div>
    
    <script>
        function toggleToolbar() {
            const items = document.querySelectorAll('.tool-item');
            gsap.to(items, {
                opacity: 1,
                scale: 1,
                duration: 0.3,
                stagger: 0.05,
                ease: 'back.out(1.7)'
            });
        }
    </script>
</body>
</html>
"""

class FloatingToolbar(AuroraView):
    def __init__(self):
        super().__init__(
            html=TOOLBAR_HTML,
            width=64,
            height=64,
            frame=False,
            transparent=True,
            always_on_top=True,
            tool_window=True,
        )
```

### Radial Menu

Create a circular menu with elastic animations:

```javascript
function toggleMenu() {
    const items = document.querySelectorAll('.menu-item');
    const RADIUS = 85;
    
    items.forEach((item, index) => {
        const angle = -Math.PI / 2 + index * (2 * Math.PI / items.length);
        const x = Math.cos(angle) * RADIUS;
        const y = Math.sin(angle) * RADIUS;
        
        gsap.fromTo(item,
            { opacity: 0, scale: 0, x: -x, y: -y },
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
}
```

### Dock Magnification

macOS-style dock with magnification effect:

```javascript
const MAGNIFICATION = 1.5;
const MAGNIFICATION_RANGE = 100;

document.addEventListener('mousemove', (e) => {
    dockItems.forEach(item => {
        const rect = item.getBoundingClientRect();
        const centerX = rect.left + rect.width / 2;
        const distance = Math.abs(e.clientX - centerX);
        const scale = Math.max(1, MAGNIFICATION - (distance / MAGNIFICATION_RANGE) * (MAGNIFICATION - 1));
        
        gsap.to(item, {
            scale: scale,
            y: (scale - 1) * -20,
            duration: 0.1,
            ease: 'power2.out'
        });
    });
});
```

## Animation Hooks (React)

The Gallery provides reusable animation hooks:

### useStaggerEntrance

```tsx
import { useStaggerEntrance } from './hooks/useGsapAnimations';

function CardGrid() {
  const containerRef = useStaggerEntrance('.card', {
    duration: 0.6,
    stagger: 0.08,
    y: 30,
  });
  
  return (
    <div ref={containerRef}>
      <div className="card">Card 1</div>
      <div className="card">Card 2</div>
    </div>
  );
}
```

### useHoverScale

```tsx
import { useHoverScale } from './hooks/useGsapAnimations';

function Button() {
  const buttonRef = useHoverScale(1.05);
  return <button ref={buttonRef}>Hover Me</button>;
}
```

### useFloatingAnimation

```tsx
import { useFloatingAnimation } from './hooks/useGsapAnimations';

function Logo() {
  const logoRef = useFloatingAnimation({
    amplitude: 10,
    duration: 3,
  });
  return <img ref={logoRef} src="logo.png" />;
}
```

## Best Practices

1. **Performance**: Use `will-change: transform` for animated elements
2. **Easing**: Choose appropriate easing functions:
   - `power2.out` - Smooth deceleration
   - `back.out(1.7)` - Overshoot effect
   - `elastic.out(1, 0.5)` - Bouncy effect
3. **Stagger**: Use stagger for lists to create wave effects
4. **Cleanup**: Kill animations on component unmount

```tsx
useEffect(() => {
  const animation = gsap.to(element, {...});
  return () => animation.kill();
}, []);
```

## Examples

See the complete examples:
- `examples/floating_toolbar_demo.py` - Expandable toolbar
- `examples/radial_menu_demo.py` - Circular menu
- `examples/dock_launcher_demo.py` - macOS-style dock
