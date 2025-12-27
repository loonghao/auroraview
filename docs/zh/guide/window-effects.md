# 窗口特效

AuroraView 提供高级窗口特效，用于创建现代、视觉吸引力的应用：

- **点击穿透**：允许鼠标事件穿透透明区域
- **背景模糊**：应用原生 Windows 模糊效果（Acrylic、Mica）

## 点击穿透窗口

点击穿透模式允许你创建透明覆盖窗口，鼠标事件会穿透到下层窗口，同时保持特定的交互区域活跃。

### 使用场景

- 浮动覆盖面板
- HUD 显示
- 标注工具
- 桌面小部件

### 基本用法

```python
from auroraview import AuroraView

# 创建透明窗口
webview = AuroraView(
    html="""
    <html>
    <body style="background: transparent;">
        <button data-interactive style="position: fixed; top: 10px; left: 10px;">
            点击我
        </button>
        <div style="position: fixed; bottom: 10px; right: 10px; opacity: 0.5;">
            此区域点击会穿透
        </div>
    </body>
    </html>
    """,
    transparent=True,
    decorations=False,
)

# 启用点击穿透模式
webview.enable_click_through()

# 定义交互区域（接收鼠标事件的区域）
webview.update_interactive_regions([
    {"x": 10, "y": 10, "width": 100, "height": 40}  # 按钮区域
])

webview.show()
```

### JavaScript 集成

`data-interactive` 属性自动跟踪元素位置：

```html
<!-- 这些元素将是交互式的 -->
<button data-interactive>按钮 1</button>
<div data-interactive class="toolbar">工具栏</div>

<!-- 这些元素将穿透点击 -->
<div class="overlay">透明覆盖层</div>
```

```javascript
// SDK 自动监控 data-interactive 元素
// 并将其位置发送到原生层

// 手动更新（如需要）
window.auroraview.interactive.update();

// 获取当前区域
const regions = window.auroraview.interactive.getRegions();

// 禁用跟踪
window.auroraview.interactive.setEnabled(false);
```

### Python API

```python
# 启用点击穿透
webview.enable_click_through()

# 禁用点击穿透
webview.disable_click_through()

# 检查是否启用
is_enabled = webview.is_click_through_enabled()

# 手动更新交互区域
webview.update_interactive_regions([
    {"x": 0, "y": 0, "width": 200, "height": 50},
    {"x": 300, "y": 100, "width": 150, "height": 80},
])

# 获取当前区域
regions = webview.get_interactive_regions()
```

## 背景模糊（毛玻璃效果）

AuroraView 支持 Windows 原生背景模糊效果：

| 效果 | Windows 10 | Windows 11 | 描述 |
|------|------------|------------|------|
| Blur | ✅ 1809+ | ✅ | 基本窗口背后模糊 |
| Acrylic | ✅ 1809+ | ✅ | 带噪点的半透明模糊 |
| Mica | ❌ | ✅ 22000+ | 桌面壁纸采样 |
| Mica Alt | ❌ | ✅ 22523+ | 更强的 Mica 变体 |

### 基本用法

```python
from auroraview import AuroraView

webview = AuroraView(
    html="""
    <html>
    <body style="background: rgba(30, 30, 30, 0.5);">
        <h1>模糊背景</h1>
    </body>
    </html>
    """,
    transparent=True,
    decorations=False,
)

# 应用模糊效果
webview.apply_blur()

# 或使用自定义着色颜色（RGBA）
webview.apply_blur((30, 30, 30, 200))

webview.show()
```

### 效果类型

#### Blur

基本模糊效果，适用于 Windows 10 1809+ 和 Windows 11。

```python
# 应用模糊
webview.apply_blur()

# 带深色着色的模糊
webview.apply_blur((30, 30, 30, 200))

# 清除模糊
webview.clear_blur()
```

#### Acrylic

带噪点纹理的半透明模糊，类似 Windows Fluent Design。

```python
# 应用 acrylic
webview.apply_acrylic()

# 带自定义颜色的 acrylic
webview.apply_acrylic((30, 30, 30, 150))

# 清除 acrylic
webview.clear_acrylic()
```

::: warning
在某些 Windows 版本上，Acrylic 在窗口调整大小时可能有性能问题。
:::

#### Mica

Windows 11 材质，采样桌面壁纸创建个性化背景。

```python
# 浅色模式 mica
webview.apply_mica(dark=False)

# 深色模式 mica
webview.apply_mica(dark=True)

# 清除 mica
webview.clear_mica()
```

#### Mica Alt

Mica 的更强变体，通常用于标签页窗口。

```python
# 应用 mica alt
webview.apply_mica_alt(dark=True)

# 清除 mica alt
webview.clear_mica_alt()
```

### CSS 集成

对于局部区域模糊，使用 CSS `backdrop-filter`：

```css
.blurred-panel {
    background: rgba(30, 30, 30, 0.5);
    backdrop-filter: blur(20px);
    border-radius: 10px;
}
```

## 平台支持

| 功能 | Windows 10 | Windows 11 | macOS | Linux |
|------|------------|------------|-------|-------|
| 点击穿透 | ✅ | ✅ | ❌ | ❌ |
| Blur | ✅ 1809+ | ✅ | ❌ | ❌ |
| Acrylic | ✅ 1809+ | ✅ | ❌ | ❌ |
| Mica | ❌ | ✅ 22000+ | ❌ | ❌ |
| Mica Alt | ❌ | ✅ 22523+ | ❌ | ❌ |

## 故障排除

### 点击穿透不工作

1. **检查是否启用了点击穿透**：
   ```python
   print(webview.is_click_through_enabled())  # 应该是 True
   ```

2. **验证交互区域已设置**：
   ```python
   regions = webview.get_interactive_regions()
   print(f"交互区域: {regions}")
   ```

3. **确保窗口是透明的**：
   ```python
   webview = WebView(transparent=True, ...)
   ```

### 模糊效果不可见

1. **窗口必须是透明的**：
   ```python
   webview = WebView(transparent=True, ...)
   ```

2. **HTML 背景必须是半透明的**：
   ```html
   <body style="background: rgba(30, 30, 30, 0.5);">
   ```

3. **检查 Windows 版本兼容性**（见平台支持表）

### "HWND not available" 错误

此错误发生在窗口完全初始化之前尝试使用窗口特效时。在 `show()` 之后调用特效方法：

```python
webview = WebView(...)
webview.show()  # 窗口现在已初始化
webview.apply_blur()  # 现在可以安全调用
```

### JavaScript 中 API 调用返回错误

通过 `auroraview.api.*` 调用窗口特效 API 时，使用对象参数：

```javascript
// 正确 - 使用对象参数
await auroraview.api.apply_blur({color: [30, 30, 30, 200]});
await auroraview.api.apply_mica({dark: true});
await auroraview.api.update_interactive_regions({regions: [...]});

// 错误 - 数组参数会被展开
await auroraview.api.apply_blur([30, 30, 30, 200]);  // 错误！
```

## 已知限制

1. **仅限 Windows**：点击穿透和毛玻璃效果仅在 Windows 10/11 上支持。不支持 macOS 和 Linux。

2. **Mica 需要 Windows 11**：Mica 和 Mica Alt 效果仅在 Windows 11 build 22000+ 上工作。

3. **Acrylic 性能**：在某些 Windows 版本上，Acrylic 效果在窗口调整大小时可能导致性能问题。

4. **点击穿透和 DCC 集成**：在 DCC 宿主（Maya、3ds Max 等）中使用点击穿透时，当窗口移动或调整大小时必须手动更新交互区域。

5. **透明窗口伪影**：在某些系统上，透明窗口可能显示渲染伪影。使用 `extend_frame_into_client_area` 获得更好的效果。

## DCC 集成说明

在 DCC 应用程序（Maya、3ds Max、Houdini 等）中使用窗口特效时，有一些重要注意事项。

### 集成模式

AuroraView 提供三种集成模式适用于不同场景：

| 模式 | 类 | 描述 | 适用场景 |
|------|-----|------|----------|
| **桌面模式** | `WebView` + `show()` | 独立窗口，拥有自己的事件循环 | 独立工具、桌面应用 |
| **原生模式 (HWND)** | `WebView` + `parent=hwnd` | 通过 HWND 嵌入，不依赖 Qt | Blender、Unreal Engine、非 Qt 应用 |
| **Qt 模式** | `QtWebView` | 作为 Qt widget 子窗口嵌入 | Maya、Houdini、Nuke、3ds Max |

### 各模式特效支持

| 功能 | 桌面模式 | 原生模式 (HWND) | Qt 模式 |
|------|----------|-----------------|---------|
| 点击穿透 | ✅ 完全支持 | ✅ 完全支持 | ⚠️ 有限 |
| Blur/Acrylic | ✅ 完全支持 | ✅ 完全支持 | ⚠️ 可能冲突 |
| Mica | ✅ 完全支持 | ✅ 完全支持 | ❌ 不推荐 |
| 透明窗口 | ✅ 完全支持 | ✅ 完全支持 | ⚠️ 依赖 Qt |

::: tip 模式选择
- **桌面模式**：最适合独立应用和开发/测试
- **原生模式 (HWND)**：最适合非 Qt 的 DCC（Blender、Unreal），完全支持特效
- **Qt 模式**：最适合基于 Qt 的 DCC（Maya、Houdini、Nuke），需要停靠功能时使用
:::

### Qt 模式限制

通过 Qt 将 AuroraView 嵌入 DCC 应用程序时：

1. **点击穿透**：可以工作，但当父 Qt widget 移动或调整大小时需要仔细管理交互区域。

2. **毛玻璃效果**：可能与 Qt 自身的窗口合成冲突。在目标 DCC 中彻底测试。

3. **窗口句柄访问**：特效需要直接访问 HWND，可以通过 `webview._core` 获取，但在 Qt widget 生命周期中可能有时序问题。

### DCC 中的推荐用法

```python
from auroraview import WebView
from auroraview.qt import QtWebView

# 桌面模式 - 独立窗口，完全支持特效
def create_desktop_panel():
    webview = WebView(
        title="桌面面板",
        transparent=True,
        decorations=False,
        always_on_top=True,
        tool_window=True,
    )
    webview.enable_click_through()
    webview.apply_acrylic((30, 30, 30, 180))
    webview.show()  # 阻塞调用，拥有事件循环
    return webview

# 原生模式 (HWND) - 用于 Blender、Unreal 等。完全支持特效
def create_native_panel(parent_hwnd):
    webview = WebView(
        title="原生面板",
        parent=parent_hwnd,  # 来自非 Qt 应用的 HWND
        transparent=True,
        decorations=False,
    )
    webview.enable_click_through()
    webview.apply_acrylic((30, 30, 30, 180))
    return webview

# Qt 模式 - 用于 Maya、Houdini、Nuke。特效支持有限
def create_qt_panel(parent_widget):
    qt_webview = QtWebView(parent=parent_widget)
    # 注意：特效在 Qt 模式下可能无法按预期工作
    # 使用 CSS backdrop-filter 代替模糊效果
    return qt_webview
```

### DCC 最佳实践

1. **对需要点击穿透或毛玻璃效果的覆盖面板使用桌面或原生模式**。

2. **对停靠面板使用 Qt 模式**，视觉效果不太重要但需要 Qt 集成。

3. **在目标 DCC 版本上测试**，因为不同 DCC 的 Qt 版本不同。

4. **提供后备样式**，以应对特效不工作的情况。

5. **处理窗口生命周期事件**，在面板关闭时正确清理特效。

## 最佳实践

1. 使用模糊效果时**始终设置 `transparent=True`**
2. 对于自定义形状窗口**使用 `decorations=False`**
3. **结合 `tool_window=True`** 从任务栏隐藏
4. **在目标 Windows 版本上测试**，因为效果各不相同
5. **为不支持的平台提供后备样式**
6. 从 JavaScript 调用 API 时**使用对象参数**

## 示例：浮动面板

```python
from auroraview import AuroraView

html = """
<!DOCTYPE html>
<html>
<head>
    <style>
        body {
            margin: 0;
            background: transparent;
            font-family: 'Segoe UI', sans-serif;
        }
        .panel {
            background: rgba(30, 30, 30, 0.7);
            border-radius: 12px;
            padding: 20px;
            margin: 10px;
            color: white;
        }
        button {
            background: #0078d4;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 6px;
            cursor: pointer;
        }
    </style>
</head>
<body>
    <div class="panel" data-interactive>
        <h2>浮动面板</h2>
        <p>此面板有模糊背景</p>
        <button data-interactive>操作</button>
    </div>
</body>
</html>
"""

webview = AuroraView(
    html=html,
    width=300,
    height=200,
    transparent=True,
    decorations=False,
    always_on_top=True,
    tool_window=True,
)

# 为透明区域启用点击穿透
webview.enable_click_through()

# 应用 acrylic 模糊
webview.apply_acrylic((30, 30, 30, 180))

webview.show()
```
