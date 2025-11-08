# Singleton Mode Guide

## 概述

AuroraView现在支持单例模式，允许你控制WebView实例的创建行为：
- **单例模式**: 同一时间只允许一个实例存在（适合工具窗口）
- **多实例模式**: 允许创建多个独立的窗口实例（适合文档窗口）

## 为什么需要单例模式？

在DCC应用（如Maya、Houdini）中，工具窗口通常应该是单例的：
- 避免用户意外打开多个相同的工具窗口
- 确保工具状态的一致性
- 简化窗口管理和资源清理

## 使用方法

### 1. WebView.create() API

```python
from auroraview import WebView

# 单例模式 - 使用singleton参数
webview1 = WebView.create(
    "My Tool",
    url="http://localhost:3000",
    singleton="my_tool"  # 单例键
)

# 再次调用返回相同实例
webview2 = WebView.create(
    "My Tool",
    url="http://localhost:3000",
    singleton="my_tool"  # 相同的键
)

assert webview1 is webview2  # True - 同一个实例
```

### 2. Maya Outliner示例

```python
from maya_integration import maya_outliner

# 单例模式（默认）- 只允许一个Outliner窗口
outliner1 = maya_outliner.main()
outliner2 = maya_outliner.main()  # 返回相同实例
assert outliner1 is outliner2

# 多实例模式 - 允许多个Outliner窗口
outliner1 = maya_outliner.main(singleton=False)
outliner2 = maya_outliner.main(singleton=False)  # 创建新实例
assert outliner1 is not outliner2
```

## 单例键（Singleton Key）

单例键是一个字符串，用于标识单例实例：
- 相同的键 → 返回相同的实例
- 不同的键 → 创建不同的实例

```python
# 不同的工具使用不同的单例键
outliner = WebView.create("Outliner", singleton="maya_outliner")
asset_browser = WebView.create("Assets", singleton="asset_browser")

# 这两个是不同的实例
assert outliner is not asset_browser
```

## 窗口关闭和清理

### 自动清理

当窗口关闭时，会自动从单例注册表中移除：

```python
# 创建单例实例
webview1 = WebView.create("Tool", singleton="my_tool")

# 关闭窗口
webview1.close()

# 现在可以创建新实例
webview2 = WebView.create("Tool", singleton="my_tool")
assert webview1 is not webview2  # 不同的实例
```

### 手动关闭

```python
# 方式1: 调用close()方法
webview.close()

# 方式2: 使用上下文管理器
with WebView.create("Tool", singleton="my_tool") as webview:
    webview.show()
# 自动关闭
```

## Maya Outliner改进

### 窗口关闭问题修复

我们修复了Maya Outliner的窗口关闭问题：

1. **EventTimer正确停止**: 在关闭前停止事件处理
2. **Maya回调清理**: 移除所有注册的回调
3. **防止重入**: 使用`_is_closing`标志防止重复关闭
4. **单例注册清理**: 从注册表中移除实例

### 关闭流程

```python
outliner = maya_outliner.main()

# 关闭窗口 - 执行以下步骤：
# 1. 停止EventTimer
# 2. 移除Maya callbacks
# 3. 关闭WebView窗口
# 4. 从单例注册表移除
# 5. 清理引用
outliner.close()
```

## 最佳实践

### 1. 工具窗口使用单例模式

```python
# 推荐：工具窗口使用单例
outliner = WebView.create(
    "Maya Outliner",
    url="http://localhost:3000",
    singleton="maya_outliner"
)
```

### 2. 文档窗口使用多实例模式

```python
# 推荐：文档窗口允许多实例
def open_document(file_path):
    return WebView.create(
        f"Document: {file_path}",
        url=f"file://{file_path}",
        # 不使用singleton参数
    )
```

### 3. 使用有意义的单例键

```python
# 好的命名
singleton="maya_outliner"
singleton="asset_browser"
singleton="render_settings"

# 避免通用名称
singleton="tool"  # 太通用
singleton="window"  # 不明确
```

### 4. 在DCC集成中默认使用单例

```python
def create_maya_tool():
    """创建Maya工具窗口（单例）"""
    return WebView.maya(
        "My Tool",
        url="http://localhost:3000",
        singleton="my_maya_tool"  # 默认单例
    )
```

## 测试

运行测试脚本验证功能：

```bash
# 测试所有功能
python examples/maya-outliner/test_singleton.py

# 只测试单例模式
python examples/maya-outliner/test_singleton.py --test singleton

# 只测试多实例模式
python examples/maya-outliner/test_singleton.py --test multi

# 只测试窗口关闭
python examples/maya-outliner/test_singleton.py --test close
```

## API参考

### WebView.create()

```python
WebView.create(
    title: str = "AuroraView",
    *,
    url: Optional[str] = None,
    html: Optional[str] = None,
    width: int = 800,
    height: int = 600,
    singleton: Optional[str] = None,  # 单例键
    # ... 其他参数
) -> WebView
```

### maya_outliner.main()

```python
maya_outliner.main(
    url: Optional[str] = None,
    use_local: bool = False,
    singleton: bool = True  # 默认启用单例模式
) -> MayaOutliner
```

## 常见问题

**Q: 单例模式会影响性能吗？**

A: 不会。单例检查非常快速（字典查找），对性能影响可忽略不计。

**Q: 如何强制创建新实例？**

A: 先关闭旧实例，或使用不同的单例键，或不使用singleton参数。

**Q: 单例实例会自动关闭旧窗口吗？**

A: 不会。如果实例已存在，会直接返回现有实例。如果需要替换，请先手动关闭旧实例。

**Q: 单例注册表会导致内存泄漏吗？**

A: 不会。当窗口关闭时，会自动从注册表中移除。

## 总结

- ✅ 单例模式适合工具窗口
- ✅ 多实例模式适合文档窗口
- ✅ 使用有意义的单例键
- ✅ 窗口关闭会自动清理
- ✅ Maya Outliner默认使用单例模式

