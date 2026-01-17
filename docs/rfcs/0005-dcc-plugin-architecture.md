# RFC 0005: DCC Plugin Architecture

- **Status**: Draft
- **Created**: 2026-01-15
- **Author**: AuroraView Team

## Summary

将 DCC（Digital Content Creation）集成代码从核心库中分离，采用插件化架构，使核心库保持轻量和职责单一。

## Motivation

### 当前问题

1. **违背单一职责原则**：核心库包含大量 DCC 特定代码（Maya、Houdini、Blender、Nuke 等）
2. **维护负担增加**：每个 DCC 版本更新都可能影响核心库发版
3. **依赖膨胀**：用户只使用一个 DCC，却需要下载所有 DCC 的集成代码
4. **测试复杂度高**：CI 需要模拟各种 DCC 环境
5. **版本耦合**：DCC 插件的 bug 修复需要等待核心库发版

### 目标

- 核心库专注于 WebView 能力，保持轻量
- DCC 集成作为独立包发布，按需安装
- 降低社区贡献门槛，DCC 插件可由社区维护
- 独立版本控制和发布周期

## Design

### 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                     DCC Applications                         │
├──────────┬──────────┬──────────┬──────────┬────────────────┤
│   Maya   │ Houdini  │ Blender  │   Nuke   │    Others      │
└────┬─────┴────┬─────┴────┬─────┴────┬─────┴───────┬────────┘
     │          │          │          │             │
     ▼          ▼          ▼          ▼             ▼
┌─────────┐┌─────────┐┌─────────┐┌─────────┐┌─────────────────┐
│auroraview││auroraview││auroraview││auroraview││ auroraview-xxx  │
│  -maya  ││-houdini ││-blender ││  -nuke  ││ (community)     │
└────┬────┘└────┬────┘└────┬────┘└────┬────┘└───────┬─────────┘
     │          │          │          │             │
     └──────────┴──────────┴──────────┴─────────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │      auroraview        │
              │      (core lib)        │
              ├────────────────────────┤
              │ • WebView Core API     │
              │ • Window Management    │
              │ • JS Bridge Protocol   │
              │ • Qt Integration Base  │
              │ • Plugin Discovery     │
              └────────────────────────┘
```

### 包结构

#### 核心库 (auroraview)

```
auroraview/
├── core/
│   ├── webview.py          # WebView 核心 API
│   ├── window_manager.py   # 窗口管理
│   └── config.py           # 配置
├── browser/
│   └── tab_container.py    # 标签页容器
├── integration/
│   └── qt/
│       └── _core.py        # Qt 集成基础（不依赖特定 DCC）
├── plugins/
│   ├── __init__.py         # 插件发现机制
│   ├── base.py             # 插件基类
│   └── registry.py         # 插件注册表
└── types.py                # 公共类型定义
```

#### DCC 插件包 (auroraview-maya)

```
auroraview-maya/
├── pyproject.toml
├── src/
│   └── auroraview_maya/
│       ├── __init__.py
│       ├── panel.py        # Maya 面板管理
│       ├── menu.py         # Maya 菜单集成
│       ├── workspace.py    # 工作区适配
│       └── plugin.py       # 插件入口
└── tests/
```

### 插件发现机制

#### Entry Points 方式

```toml
# auroraview-maya/pyproject.toml
[project.entry-points."auroraview.plugins"]
maya = "auroraview_maya:MayaPlugin"
```

```python
# auroraview/plugins/__init__.py
from importlib.metadata import entry_points

def discover_plugins() -> dict[str, type["DCCPlugin"]]:
    """发现所有已安装的 DCC 插件"""
    plugins = {}
    eps = entry_points(group="auroraview.plugins")
    for ep in eps:
        try:
            plugin_cls = ep.load()
            plugins[ep.name] = plugin_cls
        except Exception as e:
            logger.warning(f"Failed to load plugin {ep.name}: {e}")
    return plugins

def get_current_dcc_plugin() -> "DCCPlugin | None":
    """自动检测当前 DCC 环境并返回对应插件"""
    plugins = discover_plugins()
    for name, plugin_cls in plugins.items():
        if plugin_cls.detect():
            return plugin_cls()
    return None
```

### 插件基类

```python
# auroraview/plugins/base.py
from abc import ABC, abstractmethod
from typing import Any, TYPE_CHECKING

if TYPE_CHECKING:
    from auroraview.core import WebView

class DCCPlugin(ABC):
    """DCC 插件基类"""
    
    name: str = "unknown"
    version: str = "0.0.0"
    
    @classmethod
    @abstractmethod
    def detect(cls) -> bool:
        """检测当前是否在该 DCC 环境中运行"""
        ...
    
    @abstractmethod
    def get_main_window(self) -> Any:
        """获取 DCC 主窗口对象"""
        ...
    
    @abstractmethod
    def create_panel(
        self,
        webview: "WebView",
        title: str,
        **kwargs
    ) -> Any:
        """创建嵌入式面板"""
        ...
    
    @abstractmethod
    def create_dock(
        self,
        webview: "WebView",
        title: str,
        area: str = "right",
        **kwargs
    ) -> Any:
        """创建可停靠窗口"""
        ...
    
    def register_menu(self, menu_path: str, callback: callable) -> None:
        """注册菜单项（可选实现）"""
        raise NotImplementedError
    
    def save_workspace(self) -> dict:
        """保存工作区状态（可选实现）"""
        return {}
    
    def restore_workspace(self, state: dict) -> None:
        """恢复工作区状态（可选实现）"""
        pass
```

### 使用示例

#### 自动检测模式

```python
from auroraview import WebView
from auroraview.plugins import get_current_dcc_plugin

# 自动检测 DCC 环境
plugin = get_current_dcc_plugin()
if plugin:
    webview = WebView(url="http://localhost:3000")
    panel = plugin.create_dock(webview, title="My Tool", area="right")
else:
    # Standalone 模式
    webview = WebView(url="http://localhost:3000")
    webview.show()
```

#### 显式指定模式

```python
from auroraview import WebView
from auroraview_maya import MayaPlugin

plugin = MayaPlugin()
webview = WebView(url="http://localhost:3000")
panel = plugin.create_dock(webview, title="My Tool")
```

### 核心库保留的 Qt 集成

核心库仍保留基础的 Qt 集成能力，但不包含 DCC 特定逻辑：

```python
# auroraview/integration/qt/_core.py

class QtWebView:
    """基础 Qt WebView 容器，不依赖任何 DCC"""
    
    def __init__(self, parent: QWidget | None = None):
        self._container = QWidget(parent)
        self._webview = WebView()
        # ... 基础嵌入逻辑
    
    def widget(self) -> QWidget:
        """返回可嵌入的 QWidget"""
        return self._container
```

### DCC 插件实现示例

#### Maya 插件

```python
# auroraview-maya/src/auroraview_maya/plugin.py
from auroraview.plugins import DCCPlugin

class MayaPlugin(DCCPlugin):
    name = "maya"
    version = "0.1.0"
    
    @classmethod
    def detect(cls) -> bool:
        try:
            import maya.cmds
            return True
        except ImportError:
            return False
    
    def get_main_window(self):
        from maya import OpenMayaUI
        from shiboken2 import wrapInstance
        ptr = OpenMayaUI.MQtUtil.mainWindow()
        return wrapInstance(int(ptr), QMainWindow)
    
    def create_panel(self, webview, title, **kwargs):
        from .panel import MayaWebViewPanel
        return MayaWebViewPanel(webview, title, **kwargs)
    
    def create_dock(self, webview, title, area="right", **kwargs):
        from .dock import MayaWebViewDock
        return MayaWebViewDock(webview, title, area, **kwargs)
```

## Migration Plan

### Phase 1: 核心库重构 (Week 1-2)

1. 创建 `auroraview.plugins` 模块
2. 定义 `DCCPlugin` 基类和发现机制
3. 重构现有 Qt 集成代码，移除 DCC 特定逻辑
4. 添加插件系统单元测试

### Phase 2: 首个插件包 (Week 3-4)

1. 创建 `auroraview-maya` 独立仓库/包
2. 迁移 Maya 特定代码
3. 编写 Maya 环境集成测试
4. 发布 beta 版本

### Phase 3: 其他 DCC 插件 (Week 5-8)

1. 创建 `auroraview-houdini`
2. 创建 `auroraview-blender`
3. 创建 `auroraview-nuke`
4. 完善文档和示例

### Phase 4: 社区化 (Ongoing)

1. 发布插件开发指南
2. 创建插件模板仓库
3. 建立插件审核流程
4. 社区插件目录

## Package Structure Options

### Option A: Monorepo with Multiple Packages

```
auroraview/
├── packages/
│   ├── auroraview/           # 核心库
│   ├── auroraview-maya/
│   ├── auroraview-houdini/
│   └── auroraview-blender/
└── pyproject.toml            # workspace 配置
```

**优点**: 统一版本管理，便于跨包修改
**缺点**: 发布流程复杂

### Option B: Separate Repositories

```
github.com/AuroraView/
├── auroraview              # 核心库
├── auroraview-maya
├── auroraview-houdini
└── auroraview-blender
```

**优点**: 独立版本控制，社区可以 fork 维护
**缺点**: 跨仓库协调困难

### 推荐: Option A (Monorepo)

初期使用 Monorepo 便于开发和测试，成熟后可考虑拆分。

## Compatibility

### Python 版本

- 核心库: Python 3.7+
- DCC 插件: 根据各 DCC 的 Python 版本要求

### DCC 版本矩阵

| 插件包 | 支持的 DCC 版本 | Python 版本 |
|-------|---------------|------------|
| auroraview-maya | Maya 2022+ | 3.7-3.10 |
| auroraview-houdini | Houdini 19.5+ | 3.7-3.10 |
| auroraview-blender | Blender 3.0+ | 3.10+ |
| auroraview-nuke | Nuke 14+ | 3.7-3.9 |

## Testing Strategy

### 核心库测试

- 单元测试: 不依赖任何 DCC
- 集成测试: 使用 Mock DCC 环境

### DCC 插件测试

- 每个插件包独立 CI
- 使用 DCC 提供的 mayapy/hython 等命令行工具
- 可选: 使用 Docker 容器化 DCC 环境

## Documentation

### 核心库文档

- 快速开始 (Standalone 模式)
- API 参考
- 插件开发指南

### DCC 插件文档

- 安装指南
- DCC 特定配置
- 工作区集成示例

## Open Questions

1. **插件版本兼容性**: 如何处理核心库升级后插件不兼容的情况？
2. **依赖管理**: DCC 插件是否应该固定核心库版本？
3. **社区插件审核**: 如何确保第三方插件质量？
4. **许可证**: 社区插件的许可证要求？

## References

- [Python Entry Points](https://packaging.python.org/en/latest/specifications/entry-points/)
- [pywebview Plugin Architecture](https://github.com/nicegui-org/nicegui)
- [Flet DCC Integration](https://flet.dev)
