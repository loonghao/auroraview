# 浮窗面板

创建浮动工具窗口，用于 AI 助手、工具面板或覆盖层界面。

## 基础浮窗面板

```python
from auroraview import WebView

webview = WebView.create(
    title="AI 助手",
    html=panel_html,
    width=320,
    height=400,
    frame=False,         # 无边框窗口
    transparent=True,    # 透明背景
    always_on_top=True,  # 保持置顶
)
webview.show()
```

## 工具窗口模式

从任务栏和 Alt+Tab 中隐藏：

```python
webview = WebView.create(
    title="工具面板",
    html=palette_html,
    width=200,
    height=600,
    frame=False,
    tool_window=True,    # 从任务栏/Alt+Tab 隐藏 (WS_EX_TOOLWINDOW)
)
```

## 所有者模式

窗口跟随父窗口最小化/还原：

```python
webview = WebView.create(
    title="浮动工具",
    html=tool_html,
    parent=parent_hwnd,  # 父窗口句柄
    mode="owner",        # 跟随父窗口最小化/还原
    frame=False,
    always_on_top=True,
)
```

## 透明浮动按钮

创建真正透明的圆形按钮：

```python
from auroraview import AuroraView

class TriggerButton(AuroraView):
    def __init__(self):
        super().__init__(
            html=BUTTON_HTML,
            width=48,
            height=48,
            frame=False,
            transparent=True,
            undecorated_shadow=False,  # 默认：无阴影（如需阴影请设为 True）

            always_on_top=True,
            tool_window=True,
        )
```

**透明窗口的关键参数：**
- `frame=False` - 无边框窗口
- `transparent=True` - 透明背景
- `undecorated_shadow=False` - 默认：无边框窗口**无阴影**；如需阴影请显式设为 `True`

- `tool_window=True` - 从任务栏/Alt+Tab 隐藏

## 带 GSAP 动画的浮动工具栏

创建带平滑动画的可展开工具栏：

```python
from auroraview import AuroraView

TOOLBAR_HTML = """
<!DOCTYPE html>
<html>
<head>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/gsap.min.js"></script>
    <style>
        .toolbar { display: flex; gap: 8px; }
        .tool-item { 
            width: 40px; height: 40px;
            opacity: 0; transform: scale(0.5);
        }
    </style>
</head>
<body>
    <button class="trigger" onclick="toggleToolbar()">+</button>
    <div class="toolbar" id="toolbar">
        <div class="tool-item">Maya</div>
        <div class="tool-item">Blender</div>
    </div>
    <script>
        let expanded = false;
        function toggleToolbar() {
            expanded = !expanded;
            const items = document.querySelectorAll('.tool-item');
            if (expanded) {
                gsap.to(items, {
                    opacity: 1,
                    scale: 1,
                    duration: 0.3,
                    stagger: 0.05,
                    ease: 'back.out(1.7)'
                });
            } else {
                gsap.to(items, {
                    opacity: 0,
                    scale: 0.5,
                    duration: 0.2,
                    stagger: 0.02,
                });
            }
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
            undecorated_shadow=False,
            always_on_top=True,
            tool_window=True,
        )
```

## 完整示例

```python
from auroraview import WebView

panel_html = """
<!DOCTYPE html>
<html>
<head>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            background: rgba(30, 30, 30, 0.95);
            color: white;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            border-radius: 12px;
            overflow: hidden;
        }
        .titlebar {
            height: 32px;
            background: rgba(0, 0, 0, 0.3);
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 12px;
            -webkit-app-region: drag;
        }
        .titlebar button {
            -webkit-app-region: no-drag;
            background: none;
            border: none;
            color: white;
            cursor: pointer;
            padding: 4px 8px;
        }
        .content {
            padding: 16px;
        }
    </style>
</head>
<body>
    <div class="titlebar">
        <span>AI 助手</span>
        <button onclick="auroraview.send_event('close')">✕</button>
    </div>
    <div class="content">
        <p>问我任何问题...</p>
    </div>
</body>
</html>
"""

webview = WebView.create(
    title="AI 助手",
    html=panel_html,
    width=320,
    height=400,
    frame=False,
    transparent=True,
    always_on_top=True,
    tool_window=True,
)

@webview.on("close")
def handle_close(data):
    webview.close()

webview.show()
```

## 配置选项

| 选项 | 描述 | 效果 |
|------|------|------|
| `frame=False` | 移除窗口边框 | 无边框窗口 |
| `transparent=True` | 启用透明 | 透明背景 |
| `undecorated_shadow=False` | 禁用阴影 | 真正透明的窗口 |
| `always_on_top=True` | 保持在其他窗口上方 | 始终可见 |
| `tool_window=True` | 工具窗口样式 | 从任务栏隐藏 |
| `mode="owner"` | 所有者关系 | 跟随父窗口 |

## 自定义拖拽

对于无边框窗口，使用 CSS `-webkit-app-region`：

```css
/* 使元素可拖拽 */
.titlebar {
    -webkit-app-region: drag;
}

/* 排除交互元素 */
.titlebar button {
    -webkit-app-region: no-drag;
}
```

## 相关示例

- `examples/floating_panel_demo.py` - 基础浮窗面板
- `examples/floating_toolbar_demo.py` - 带 GSAP 的可展开工具栏
- `examples/radial_menu_demo.py` - 圆形菜单
- `examples/dock_launcher_demo.py` - macOS 风格 Dock
- `examples/logo_button_demo.py` - Logo 按钮触发器
