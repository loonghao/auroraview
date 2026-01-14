---
outline: deep
---

# Agent Browser - 多标签浏览器演示

本指南演示如何使用 AuroraView WebView 创建一个类似浏览器的多标签应用。

## 概述

Agent Browser 演示展示了：

- **标签管理**：创建、关闭和切换标签
- **导航控制**：后退、前进、刷新和主页按钮
- **URL 栏**：智能 URL/搜索检测
- **文件拖放**：拖拽打开 PDF、图片和文本文件
- **键盘快捷键**：Ctrl+T、Ctrl+W、Ctrl+L、F5 等
- **状态同步**：Python 和 JavaScript 之间的标签状态同步

## 运行浏览器

```bash
# 作为模块运行
python -m examples.agent_browser

# 或直接运行
python examples/agent_browser/browser.py
```

## 核心概念

### 使用统一 API

Agent Browser 使用 AuroraView 的统一 `create_webview()` API：

```python
from auroraview import create_webview

# 创建独立浏览器窗口
webview = create_webview(
    title="Agent Browser",
    html=browser_html,
    width=1280,
    height=900,
    debug=True,
    allow_file_protocol=True,  # 启用 file:// 协议
    context_menu=True,
)
```

### 标签状态管理

演示使用 `TabManager` 类来管理标签：

```python
from dataclasses import dataclass, field
from typing import Dict, List, Optional
import threading

@dataclass
class Tab:
    """表示一个浏览器标签。"""
    id: str
    title: str = "New Tab"
    url: str = ""
    is_loading: bool = False
    can_go_back: bool = False
    can_go_forward: bool = False
    history: List[str] = field(default_factory=list)
    history_index: int = -1

class TabManager:
    """线程安全的标签管理器，支持历史记录。"""
    
    def __init__(self):
        self.tabs: Dict[str, Tab] = {}
        self.tab_order: List[str] = []
        self.active_tab_id: Optional[str] = None
        self._lock = threading.Lock()

    def navigate(self, tab_id: str, url: str) -> None:
        """导航到 URL，更新历史记录。"""
        with self._lock:
            if tab_id not in self.tabs:
                return
            tab = self.tabs[tab_id]
            
            # 截断前进历史
            if tab.history_index < len(tab.history) - 1:
                tab.history = tab.history[:tab.history_index + 1]
            
            tab.history.append(url)
            tab.history_index = len(tab.history) - 1
            tab.url = url
            tab.can_go_back = tab.history_index > 0
            tab.can_go_forward = False
```

### Python-JavaScript 通信

浏览器使用 AuroraView 的事件系统进行双向通信：

**Python → JavaScript（事件）**：
```python
# 广播标签更新到 UI
def broadcast_tabs_update():
    webview.emit("tabs:update", {
        "tabs": tab_manager.get_tabs_info(),
        "active_tab_id": tab_manager.active_tab_id,
    })
```

**JavaScript → Python（API 调用）**：
```javascript
// 创建新标签
auroraview.api.create_tab({ url: "https://example.com" });

// 关闭标签
auroraview.api.close_tab({ tab_id: "tab-123" });

// 导航
auroraview.api.navigate({ url: "https://github.com" });
```

### 文件拖放

浏览器支持通过拖拽文件到窗口来打开文件：

```python
from auroraview.core.events import WindowEvent

@webview.on(WindowEvent.FILE_DROP)
def on_file_drop(data):
    """处理文件拖放事件。"""
    paths = data.get("paths", [])
    if paths:
        result = handle_file_open(paths[0])
        if result.get("success"):
            webview.emit("file:opened", result)
```

支持的文件类型：
- **PDF**：直接在浏览器中打开
- **图片**：PNG、JPG、GIF、WebP、SVG 等
- **文本**：TXT、JSON、XML、MD、CSS、JS
- **HTML**：作为网页打开

## 实现细节

### 创建浏览器

```python
from auroraview import create_webview
from auroraview.core.events import WindowEvent

class AgentBrowser:
    def __init__(self):
        self.tab_manager = TabManager()
        self.webview = None

    def run(self):
        self.webview = create_webview(
            title="Agent Browser",
            html=self._load_html(),
            width=1280,
            height=900,
            debug=True,
            allow_file_protocol=True,
        )
        
        # 注册 API 处理器
        @self.webview.bind_call("api.create_tab")
        def create_tab(url: str = "") -> dict:
            tab_id = self.tab_manager.create_tab(url=url)
            self._broadcast_tabs_update()
            if url:
                self.webview.load_url(url)
            return {"tab_id": tab_id, "success": True}
        
        @self.webview.bind_call("api.navigate")
        def navigate(url: str) -> dict:
            final_url = self._process_url(url)
            self.tab_manager.navigate(
                self.tab_manager.active_tab_id, 
                final_url
            )
            self._broadcast_tabs_update()
            self.webview.load_url(final_url)
            return {"success": True, "url": final_url}
        
        # 创建初始标签
        self.tab_manager.create_tab()
        self.webview.show()
```

### 在 JavaScript 中处理标签事件

```javascript
window.addEventListener('auroraviewready', () => {
    // 监听来自 Python 的标签更新
    auroraview.on('tabs:update', (data) => {
        tabs = data.tabs;
        activeTabId = data.active_tab_id;
        renderTabs();
        updateNavButtons();
    });
    
    // 获取初始标签状态
    auroraview.api.get_tabs().then(data => {
        tabs = data.tabs || [];
        activeTabId = data.active_tab_id;
        renderTabs();
    });
});

// 键盘快捷键
document.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 't') { e.preventDefault(); createNewTab(); }
    if (e.ctrlKey && e.key === 'w') { e.preventDefault(); closeTab(activeTabId); }
    if (e.ctrlKey && e.key === 'l') { e.preventDefault(); focusUrlBar(); }
    if (e.key === 'F5') { e.preventDefault(); reloadPage(); }
});
```

## 键盘快捷键

| 快捷键 | 操作 |
|--------|------|
| `Ctrl+T` | 新建标签 |
| `Ctrl+W` | 关闭标签 |
| `Ctrl+L` | 聚焦 URL 栏 |
| `F5` | 刷新页面 |
| `Alt+←` | 后退 |
| `Alt+→` | 前进 |
| `F12` | 开发者工具 |

## 项目结构

```
examples/agent_browser/
├── __init__.py      # 包导出
├── __main__.py      # -m 入口点
├── browser.py       # 主浏览器实现
└── ui.html          # 浏览器 UI 模板
```

## 另请参阅

- [通信](./communication.md) - Python ↔ JavaScript 通信
- [子窗口](./child-windows.md) - 管理子窗口
- [示例](./examples.md) - 更多示例应用
