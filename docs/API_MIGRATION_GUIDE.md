# API Migration Guide

## WebView API Parameter Changes

为了提供更清晰、更符合Python习惯的API，我们更新了`WebView`类的参数名称。

### 参数名称变更

| 旧参数名 | 新参数名 | 说明 |
|---------|---------|------|
| `dev_tools` | `debug` | 启用开发者工具 |
| `parent_hwnd` | `parent` | 父窗口句柄 |
| `parent_mode` | `mode` | 嵌入模式 |

### 迁移示例

#### Before (旧API)

```python
from auroraview import WebView

webview = WebView(
    title="My Tool",
    width=800,
    height=600,
    dev_tools=True,
    parent_hwnd=maya_hwnd,
    parent_mode="owner"
)
```

#### After (新API)

```python
from auroraview import WebView

webview = WebView(
    title="My Tool",
    width=800,
    height=600,
    debug=True,      # dev_tools -> debug
    parent=maya_hwnd,  # parent_hwnd -> parent
    mode="owner"       # parent_mode -> mode
)
```

### 推荐的现代API

我们强烈推荐使用新的工厂方法API，它提供了更简洁的语法：

```python
from auroraview import WebView

# 通用方式
webview = WebView.create(
    "My Tool",
    url="http://localhost:3000",
    width=800,
    height=600,
    debug=True
)

# Maya集成（嵌入到主窗口）
import maya.OpenMayaUI as omui
maya_hwnd = int(omui.MQtUtil.mainWindow())
webview = WebView.create(
    "Maya Tool",
    url="http://localhost:3000",
    parent=maya_hwnd,
    mode="owner",
)

# Houdini集成（嵌入到主窗口）
import hou
hou_hwnd = int(hou.qt.mainWindow().winId())
webview = WebView.create(
    "Houdini Tool",
    url="http://localhost:3000",
    parent=hou_hwnd,
    mode="owner",
)

# Blender（独立窗口）
webview = WebView.create(
    "Blender Tool",
    url="http://localhost:3000"
)
```

### 自动迁移工具

我们提供了自动迁移脚本来更新你的代码：

```bash
python scripts/update_api_params.py
```

这个脚本会：
- 扫描所有Python文件
- 自动替换旧参数名为新参数名
- 生成迁移报告

### 向后兼容性

**重要**: 旧的参数名已被移除，不再支持。请使用新的参数名。

如果你看到以下错误：

```
TypeError: WebView.__init__() got an unexpected keyword argument 'dev_tools'
```

这意味着你的代码使用了旧的参数名，请按照上述指南更新。

### 完整的参数列表

#### WebView.__init__()

```python
WebView(
    title: str = "AuroraView",
    width: int = 800,
    height: int = 600,
    url: Optional[str] = None,
    html: Optional[str] = None,
    debug: bool = True,        # 启用开发者工具
    resizable: bool = True,    # 窗口可调整大小
    frame: bool = True,        # 显示窗口边框
    parent: Optional[int] = None,  # 父窗口句柄
    mode: Optional[str] = None,    # 嵌入模式: "owner" 或 "child"
)
```

#### WebView.create() (推荐)

```python
WebView.create(
    title: str = "AuroraView",
    # Content
    url: Optional[str] = None,
    html: Optional[str] = None,
    # Window properties
    width: int = 800,
    height: int = 600,
    resizable: bool = True,
    frame: bool = True,
    # DCC integration
    parent: Optional[int] = None,
    mode: Literal["auto", "owner", "child"] = "auto",
    # Development options
    debug: bool = True,
    # Automation
    auto_show: bool = False,
    auto_timer: bool = True,
)
```

### 常见问题

**Q: 为什么要改变参数名？**

A: 新的参数名更简洁、更符合Python习惯，同时与其他Python GUI库保持一致。

**Q: 我的旧代码还能用吗？**

A: 不能。旧的参数名已被移除。请使用迁移脚本或手动更新代码。

**Q: 如何快速迁移大量代码？**

A: 使用我们提供的自动迁移脚本：`python scripts/update_api_params.py`

**Q: 新API有什么优势？**

A: 
- 更简洁的参数名
- 工厂方法提供更好的默认值
- 自动DCC检测和配置
- 更好的类型提示

### 更新日志

- **2025-01-04**: 参数名称更新
  - `dev_tools` → `debug`
  - `parent_hwnd` → `parent`
  - `parent_mode` → `mode`
- **2025-01-04**: 添加工厂方法API
  - `WebView.create()`（统一入口，取代 `maya()/houdini()/blender()/for_dcc()`）

