# GSAP 动画

AuroraView 与 [GSAP](https://greensock.com/gsap/)（GreenSock 动画平台）无缝集成，为您的 Web UI 创建流畅、专业的动画效果。

## 概述

GSAP 是一个强大的 JavaScript 动画库，提供：
- 流畅、高性能的动画
- 基于时间线的序列动画
- 弹性、弹跳和弹簧效果
- 跨浏览器兼容性

## Gallery 集成

AuroraView Gallery 使用 GSAP 实现各种动画效果：

### 动画侧边栏

```tsx
import gsap from 'gsap';

// 图标悬停动画
const handleMouseEnter = () => {
  gsap.to(iconRef.current, {
    scale: 1.15,
    rotate: 5,
    duration: 0.3,
    ease: 'back.out(1.7)',
  });
};

// 激活指示器滑动
gsap.to(indicator, {
  scaleY: 1,
  opacity: 1,
  duration: 0.3,
  ease: 'power2.out',
});
```

### 卡片动画

```tsx
// 入场动画
gsap.fromTo(card, 
  { opacity: 0, y: 30, scale: 0.95 },
  { opacity: 1, y: 0, scale: 1, duration: 0.6, ease: 'power3.out' }
);

// 悬停效果带发光
gsap.to(card, {
  scale: 1.02,
  y: -4,
  duration: 0.3,
  ease: 'power2.out',
});
```

### 页面过渡

```tsx
// 页面进入
gsap.fromTo(container,
  { opacity: 0, y: 20 },
  { opacity: 1, y: 0, duration: 0.5, ease: 'power3.out' }
);

// 页面退出
gsap.to(container, {
  opacity: 0,
  y: -20,
  duration: 0.3,
  ease: 'power3.in',
});
```

## 浮窗工具架示例

### 可展开工具栏

创建从触发按钮展开的工具栏：

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

### 圆形菜单

创建带弹性动画的圆形菜单：

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

### Dock 放大效果

macOS 风格的 Dock 放大效果：

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

## 动画 Hooks（React）

Gallery 提供可复用的动画 Hooks：

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
      <div className="card">卡片 1</div>
      <div className="card">卡片 2</div>
    </div>
  );
}
```

### useHoverScale

```tsx
import { useHoverScale } from './hooks/useGsapAnimations';

function Button() {
  const buttonRef = useHoverScale(1.05);
  return <button ref={buttonRef}>悬停我</button>;
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

## 最佳实践

1. **性能**：为动画元素使用 `will-change: transform`
2. **缓动函数**：选择合适的缓动函数：
   - `power2.out` - 平滑减速
   - `back.out(1.7)` - 超出效果
   - `elastic.out(1, 0.5)` - 弹性效果
3. **交错动画**：对列表使用 stagger 创建波浪效果
4. **清理**：在组件卸载时终止动画

```tsx
useEffect(() => {
  const animation = gsap.to(element, {...});
  return () => animation.kill();
}, []);
```

## 示例

查看完整示例：
- `examples/floating_toolbar_demo.py` - 可展开工具栏
- `examples/radial_menu_demo.py` - 圆形菜单
- `examples/dock_launcher_demo.py` - macOS 风格 Dock
