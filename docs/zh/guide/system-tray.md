# 系统托盘

AuroraView 支持系统托盘集成，适用于后台应用程序。

## 基本用法

```python
from auroraview import run_desktop

run_desktop(
    title="我的后台应用",
    html=my_html,
    width=400,
    height=300,
    system_tray=True,      # 启用系统托盘
    hide_on_close=True,    # 关闭时最小化到托盘
)
```

## 功能特性

- **系统托盘图标** 带右键菜单
- **隐藏到托盘** - 关闭窗口时最小化
- **点击显示** - 从托盘恢复窗口
- **自定义托盘图标** 支持

## 配置选项

| 选项 | 描述 | 默认值 |
|------|------|--------|
| `system_tray` | 启用系统托盘 | `False` |
| `hide_on_close` | 关闭时最小化到托盘 | `False` |
| `tray_icon` | 自定义托盘图标路径 | 应用图标 |

## 示例：后台服务

```python
from auroraview import WebView

html = """
<!DOCTYPE html>
<html>
<body>
    <h1>后台服务</h1>
    <p>此应用在系统托盘中运行。</p>
    <button onclick="auroraview.send_event('minimize_to_tray')">
        最小化到托盘
    </button>
</body>
</html>
"""

webview = WebView.create(
    title="后台服务",
    html=html,
    system_tray=True,
    hide_on_close=True,
)

@webview.on("minimize_to_tray")
def handle_minimize(data):
    webview.hide()

webview.show()
```
