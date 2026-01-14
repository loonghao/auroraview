---
outline: deep
---

# 多标签页浏览器示例

本指南演示如何使用 AuroraView WebView 创建一个类似浏览器的多标签页应用程序。

## 概述

多标签页浏览器示例展示了：

- **标签页管理**：创建、关闭和切换标签页
- **导航控件**：后退、前进、刷新和主页按钮
- **URL 栏**：智能 URL/搜索检测
- **新窗口处理**：使用 `new_window_mode` 拦截链接
- **状态同步**：在 Python 和 JavaScript 之间保持标签页状态同步

## 核心概念

### 新窗口模式

AuroraView 提供三种处理 `window.open()` 和 `target="_blank"` 链接的模式：

```python
from auroraview import WebView

# 模式 1: Deny（默认）- 阻止所有新窗口请求
webview = WebView(new_window_mode="deny")

# 模式 2: System Browser - 在默认浏览器中打开链接
webview = WebView(new_window_mode="system_browser")

# 模式 3: Child WebView - 创建新的 WebView 窗口
webview = WebView(new_window_mode="child_webview")
```

对于多标签页浏览器，`child_webview` 模式最为合适，因为它允许你拦截新窗口请求并在应用程序内处理它们。

### 标签页状态管理

示例使用 `BrowserState` 类来管理标签页：

```python
from dataclasses import dataclass, field
from typing import Dict, List, Optional
import threading

@dataclass
class Tab:
    """表示一个浏览器标签页。"""
    id: str
    title: str = "New Tab"
    url: str = ""
    is_loading: bool = False
    can_go_back: bool = False
    can_go_forward: bool = False

@dataclass
class BrowserState:
    """线程安全的浏览器状态管理器。"""
    tabs: Dict[str, Tab] = field(default_factory=dict)
    active_tab_id: Optional[str] = None
    tab_order: List[str] = field(default_factory=list)
    _lock: threading.Lock = field(default_factory=threading.Lock)

    def add_tab(self, tab: Tab) -> None:
        with self._lock:
            self.tabs[tab.id] = tab
            self.tab_order.append(tab.id)

    def remove_tab(self, tab_id: str) -> Optional[str]:
        """移除标签页并返回下一个活动标签页 ID。"""
        with self._lock:
            if tab_id not in self.tabs:
                return self.active_tab_id
            
            idx = self.tab_order.index(tab_id)
            del self.tabs[tab_id]
            self.tab_order.remove(tab_id)
            
            if not self.tab_order:
                return None
            
            new_idx = min(idx, len(self.tab_order) - 1)
            return self.tab_order[new_idx]
```

### Python-JavaScript 通信

浏览器使用 AuroraView 的事件系统进行双向通信：

**Python → JavaScript（事件）**：
```python
# 向 UI 广播标签页更新
def broadcast_tabs_update():
    main_window.emit("tabs:update", {
        "tabs": browser_state.get_tabs_info(),
        "active_tab_id": browser_state.active_tab_id,
    })
```

**JavaScript → Python（API 调用）**：
```javascript
// 创建新标签页
auroraview.api.create_tab({ url: "https://example.com" });

// 关闭标签页
auroraview.api.close_tab({ tab_id: "tab-123" });

// 导航
auroraview.api.navigate({ url: "https://github.com" });
```

## 运行示例

### 多标签页浏览器示例

```bash
python examples/multi_tab_browser_demo.py
```

此示例提供完整的浏览器式 UI，包括：
- 带有创建/关闭/切换功能的标签栏
- 带有后退/前进/刷新/主页按钮的导航栏
- 带有智能 URL 检测的 URL 栏
- 常用网站的快速链接

### 标签页 WebView 示例

```bash
python examples/tabbed_webview_demo.py
```

一个更简单的版本，专注于标签页管理概念和 `new_window_mode` 的使用。

## 实现细节

### 创建主窗口

```python
from auroraview import WebView

def create_browser_window() -> WebView:
    view = WebView.create(
        title="AuroraView Browser",
        html=BROWSER_HTML,
        width=1200,
        height=800,
        debug=True,
        # 启用 child WebView 模式以处理 window.open()
        new_window_mode="child_webview",
    )
    
    # 注册 API 处理器
    @view.bind_call("api.create_tab")
    def create_tab(url: str = "") -> dict:
        tab_id = str(uuid.uuid4())[:8]
        tab = Tab(id=tab_id, title="New Tab", url=url)
        browser_state.add_tab(tab)
        browser_state.active_tab_id = tab_id
        broadcast_tabs_update()
        return {"tab_id": tab_id}
    
    @view.bind_call("api.navigate")
    def navigate(url: str) -> dict:
        if browser_state.active_tab_id:
            browser_state.update_tab(
                browser_state.active_tab_id,
                url=url,
                title=get_title_from_url(url),
            )
            broadcast_tabs_update()
            return {"success": True}
        return {"success": False}
    
    return view
```

### 在 JavaScript 中处理标签页事件

```javascript
window.addEventListener('auroraviewready', () => {
    // 监听来自 Python 的标签页更新
    auroraview.on('tabs:update', (data) => {
        tabs = data.tabs;
        activeTabId = data.active_tab_id;
        renderTabs();
    });
    
    // 创建初始标签页
    auroraview.api.create_tab();
});

function renderTabs() {
    const container = document.getElementById('tabs-container');
    container.innerHTML = tabs.map(tab => `
        <div class="tab ${tab.is_active ? 'active' : ''}"
             onclick="switchTab('${tab.id}')">
            <span class="tab-title">${tab.title}</span>
            <span class="tab-close" 
                  onclick="event.stopPropagation(); closeTab('${tab.id}')">×</span>
        </div>
    `).join('');
}
```

## 高级主题

### 真正的多 WebView 实现

对于生产级浏览器，每个标签页都会有自己的 WebView 实例。架构如下：

```python
class TabManager:
    def __init__(self):
        self.tab_webviews: Dict[str, WebView] = {}
    
    def create_tab(self, url: str = "") -> str:
        tab_id = str(uuid.uuid4())[:8]
        
        # 为此标签页创建新的 WebView
        webview = WebView.create(
            title=f"Tab {tab_id}",
            url=url,
            new_window_mode="child_webview",
        )
        
        # 处理来自此标签页的导航事件
        @webview.on("navigation")
        def on_navigate(data):
            self.update_tab_url(tab_id, data["url"])
        
        self.tab_webviews[tab_id] = webview
        return tab_id
    
    def close_tab(self, tab_id: str):
        if tab_id in self.tab_webviews:
            self.tab_webviews[tab_id].close()
            del self.tab_webviews[tab_id]
```

### 线程安全

`BrowserState` 类使用锁来保证线程安全，这在以下情况下很重要：
- 多个标签页可能同时触发事件
- 后台任务更新标签页状态
- 主窗口需要在标签页被修改时读取状态

```python
def update_tab(self, tab_id: str, **kwargs) -> None:
    with self._lock:
        if tab_id in self.tabs:
            for key, value in kwargs.items():
                if hasattr(self.tabs[tab_id], key):
                    setattr(self.tabs[tab_id], key, value)
```

## 另请参阅

- [WebView 基础](./webview-basics.md) - 核心 WebView 概念
- [通信](./communication.md) - Python ↔ JavaScript 通信
- [子窗口](./child-windows.md) - 管理子窗口
- [示例](./examples.md) - 更多示例应用程序
