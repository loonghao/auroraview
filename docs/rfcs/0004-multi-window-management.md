# RFC 0004: 多窗口管理与标签页支持

> **状态**: Draft
> **作者**: AuroraView Team
> **创建日期**: 2026-01-13
> **目标版本**: v0.6.0

## 摘要

本 RFC 提出一套完整的多窗口管理架构，包括全局窗口注册表（WindowManager）、生命周期事件系统（ReadyEvents）、多标签容器抽象（TabContainer）以及高级浏览器 API（Browser）。该设计旨在为 DCC 应用中的多面板工具、多标签浏览器、以及复杂的多窗口场景提供统一的底层支持。

## 动机

### 当前状态分析

AuroraView 目前的窗口管理存在以下限制：

1. **缺乏全局窗口追踪**：每个 WebView 实例独立存在，无法统一管理多个窗口
2. **竞态条件风险**：API 调用没有等待机制，可能在窗口未就绪时执行操作
3. **多标签场景支持不足**：需要在应用层手动管理标签状态和 WebView 实例
4. **DCC 多面板需求**：Maya/Houdini/Nuke 等 DCC 工具常需要多个可停靠面板

### 需求分析

1. **多窗口管理**
   - 全局窗口注册与查询
   - 活动窗口追踪
   - 窗口生命周期管理

2. **生命周期同步**
   - 等待窗口创建/显示/加载完成
   - 避免竞态条件
   - 装饰器简化 API 使用

3. **多标签支持**
   - 标签状态管理
   - 标签切换与导航
   - WebView 实例复用

4. **DCC 集成**
   - 多面板工具窗口
   - 统一的事件广播
   - 跨面板通信

### 目标用户场景

| 场景 | 描述 | 当前痛点 |
|------|------|----------|
| 多标签浏览器 | Agent Browser 等需要多标签 | 需手动管理所有状态 |
| DCC 工具面板 | Maya 多个停靠面板 | 面板间无法通信 |
| 调试工具 | 多窗口调试界面 | 无法追踪所有窗口 |
| 资源管理器 | 多视图文件浏览 | 视图同步困难 |

## 设计方案

### 架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│                        Application Layer                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Browser   │  │  DCC Panel  │  │   Custom Multi-Window   │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
│         │                │                      │                │
├─────────┼────────────────┼──────────────────────┼────────────────┤
│         │         Abstraction Layer             │                │
│         ▼                ▼                      ▼                │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    TabContainer                              ││
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        ││
│  │  │ TabState │ │ TabState │ │ TabState │ │ TabState │        ││
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘        ││
│  └───────┼────────────┼────────────┼────────────┼───────────────┘│
│          │            │            │            │                │
├──────────┼────────────┼────────────┼────────────┼────────────────┤
│          │      Core Layer         │            │                │
│          ▼            ▼            ▼            ▼                │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                   WindowManager (Singleton)                  ││
│  │  ┌──────────────────────────────────────────────────────┐   ││
│  │  │  _windows: Dict[str, weakref.ref[WebView]]           │   ││
│  │  │  _active_id: Optional[str]                           │   ││
│  │  │  _on_change_callbacks: List[Callable]                │   ││
│  │  └──────────────────────────────────────────────────────┘   ││
│  └─────────────────────────────────────────────────────────────┘│
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                     ReadyEvents                              ││
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌──────────────┐       ││
│  │  │ created │ │  shown  │ │ loaded  │ │ bridge_ready │       ││
│  │  └─────────┘ └─────────┘ └─────────┘ └──────────────┘       ││
│  └─────────────────────────────────────────────────────────────┘│
│                              │                                   │
├──────────────────────────────┼───────────────────────────────────┤
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                      WebView Core                            ││
│  │              (Existing Implementation)                       ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### 组件 1: WindowManager (全局窗口注册表)

#### 设计目标

- 单例模式，全局唯一
- 线程安全
- 弱引用避免内存泄漏
- 支持变更通知

#### API 设计

```python
# python/auroraview/core/window_manager.py

from __future__ import annotations
from typing import TYPE_CHECKING, Dict, List, Optional, Callable, Any
from threading import Lock
from uuid import uuid4
import weakref

if TYPE_CHECKING:
    from .webview import WebView

class WindowManager:
    """Global window registry for managing multiple WebView instances.
    
    Features:
    - Singleton pattern for global access
    - Thread-safe operations
    - Weak references to prevent memory leaks
    - Change notification callbacks
    
    Example:
        >>> wm = get_window_manager()
        >>> wm.register(webview)
        'wv_a1b2c3d4'
        >>> wm.get_active()
        <WebView object>
        >>> wm.get_all()
        [<WebView>, <WebView>]
    """
    
    _instance: Optional["WindowManager"] = None
    _lock = Lock()
    
    def __new__(cls) -> "WindowManager":
        if cls._instance is None:
            with cls._lock:
                if cls._instance is None:
                    cls._instance = super().__new__(cls)
                    cls._instance._init()
        return cls._instance
    
    def _init(self) -> None:
        """Initialize instance state."""
        self._windows: Dict[str, weakref.ref] = {}
        self._active_id: Optional[str] = None
        self._on_change_callbacks: List[Callable[[], None]] = []
        self._lock = Lock()
    
    def register(self, window: "WebView", uid: Optional[str] = None) -> str:
        """Register a window and return its unique ID.
        
        Args:
            window: The WebView instance to register
            uid: Optional custom unique ID. Auto-generated if not provided.
            
        Returns:
            The unique ID assigned to this window
        """
        with self._lock:
            if uid is None:
                uid = f"wv_{uuid4().hex[:8]}"
            
            self._windows[uid] = weakref.ref(
                window, 
                lambda _: self._on_window_gc(uid)
            )
            
            if self._active_id is None:
                self._active_id = uid
            
            self._notify_change()
            return uid
    
    def unregister(self, uid: str) -> bool:
        """Unregister a window by ID.
        
        Args:
            uid: The window's unique ID
            
        Returns:
            True if window was found and removed
        """
        with self._lock:
            if uid not in self._windows:
                return False
            
            del self._windows[uid]
            
            if self._active_id == uid:
                self._active_id = next(iter(self._windows.keys()), None)
            
            self._notify_change()
            return True
    
    def get(self, uid: str) -> Optional["WebView"]:
        """Get a window by ID.
        
        Args:
            uid: The window's unique ID
            
        Returns:
            The WebView instance, or None if not found
        """
        ref = self._windows.get(uid)
        return ref() if ref else None
    
    def get_active(self) -> Optional["WebView"]:
        """Get the currently active window."""
        if self._active_id:
            return self.get(self._active_id)
        return None
    
    def get_active_id(self) -> Optional[str]:
        """Get the ID of the currently active window."""
        return self._active_id
    
    def set_active(self, uid: str) -> bool:
        """Set the active window by ID.
        
        Args:
            uid: The window's unique ID
            
        Returns:
            True if window exists and was set as active
        """
        with self._lock:
            if uid not in self._windows:
                return False
            
            self._active_id = uid
            self._notify_change()
            return True
    
    def get_all(self) -> List["WebView"]:
        """Get all registered windows."""
        with self._lock:
            return [
                ref() for ref in self._windows.values() 
                if ref() is not None
            ]
    
    def get_all_ids(self) -> List[str]:
        """Get all registered window IDs."""
        with self._lock:
            return list(self._windows.keys())
    
    def count(self) -> int:
        """Get the number of registered windows."""
        with self._lock:
            return len(self._windows)
    
    def on_change(self, callback: Callable[[], None]) -> Callable[[], None]:
        """Register a callback for window changes.
        
        Args:
            callback: Function to call when windows change
            
        Returns:
            A function to unregister the callback
        """
        self._on_change_callbacks.append(callback)
        return lambda: self._on_change_callbacks.remove(callback)
    
    def broadcast(self, event: str, data: Any = None) -> None:
        """Broadcast an event to all windows.
        
        Args:
            event: Event name
            data: Event payload
        """
        for window in self.get_all():
            try:
                window.emit(event, data)
            except Exception:
                pass
    
    def _on_window_gc(self, uid: str) -> None:
        """Handle window being garbage collected."""
        with self._lock:
            if uid in self._windows:
                del self._windows[uid]
                if self._active_id == uid:
                    self._active_id = next(iter(self._windows.keys()), None)
    
    def _notify_change(self) -> None:
        """Notify all change callbacks."""
        for cb in self._on_change_callbacks:
            try:
                cb()
            except Exception:
                pass


# Global accessors
def get_window_manager() -> WindowManager:
    """Get the global WindowManager instance."""
    return WindowManager()

def get_windows() -> List["WebView"]:
    """Get all registered WebView windows."""
    return get_window_manager().get_all()

def get_active_window() -> Optional["WebView"]:
    """Get the currently active WebView window."""
    return get_window_manager().get_active()

def broadcast_event(event: str, data: Any = None) -> None:
    """Broadcast an event to all windows."""
    get_window_manager().broadcast(event, data)
```

#### 使用示例

```python
from auroraview import create_webview
from auroraview.core.window_manager import get_window_manager, get_windows

# 创建多个窗口
wv1 = create_webview(url="https://example.com", title="Window 1")
wv2 = create_webview(url="https://github.com", title="Window 2")

# 获取所有窗口
windows = get_windows()
print(f"Total windows: {len(windows)}")

# 获取活动窗口
wm = get_window_manager()
active = wm.get_active()

# 监听窗口变化
def on_windows_changed():
    print(f"Windows changed, count: {wm.count()}")

unsubscribe = wm.on_change(on_windows_changed)

# 广播事件到所有窗口
wm.broadcast("theme:changed", {"theme": "dark"})
```

### 组件 2: ReadyEvents (生命周期事件系统)

#### 设计目标

- 提供窗口生命周期的等待机制
- 避免 API 调用时的竞态条件
- 装饰器简化开发体验

#### API 设计

```python
# python/auroraview/core/ready_events.py

from __future__ import annotations
from typing import TYPE_CHECKING, TypeVar, Callable, Any
from threading import Event as ThreadEvent
from functools import wraps

if TYPE_CHECKING:
    from .webview import WebView

F = TypeVar("F", bound=Callable[..., Any])

class ReadyEvents:
    """Event container for WebView lifecycle states.
    
    Provides thread-safe waiting mechanisms for various WebView states:
    - created: WebView instance created
    - shown: Window is visible
    - loaded: Page content loaded
    - bridge_ready: JS bridge is ready for communication
    
    Example:
        >>> events = ReadyEvents(webview)
        >>> events.wait_loaded(timeout=10)
        True
        >>> events.wait_bridge_ready()
        True
    """
    
    def __init__(self, window: "WebView"):
        self._window = window
        self.created = ThreadEvent()
        self.shown = ThreadEvent()
        self.loaded = ThreadEvent()
        self.bridge_ready = ThreadEvent()
    
    def wait_created(self, timeout: float = 20.0) -> bool:
        """Wait for WebView to be created.
        
        Args:
            timeout: Maximum time to wait in seconds
            
        Returns:
            True if event was set, False if timeout occurred
        """
        return self.created.wait(timeout)
    
    def wait_shown(self, timeout: float = 20.0) -> bool:
        """Wait for window to be shown.
        
        Args:
            timeout: Maximum time to wait in seconds
            
        Returns:
            True if event was set, False if timeout occurred
        """
        return self.shown.wait(timeout)
    
    def wait_loaded(self, timeout: float = 20.0) -> bool:
        """Wait for page to be loaded.
        
        Args:
            timeout: Maximum time to wait in seconds
            
        Returns:
            True if event was set, False if timeout occurred
        """
        return self.loaded.wait(timeout)
    
    def wait_bridge_ready(self, timeout: float = 20.0) -> bool:
        """Wait for JS bridge to be ready.
        
        Args:
            timeout: Maximum time to wait in seconds
            
        Returns:
            True if event was set, False if timeout occurred
        """
        return self.bridge_ready.wait(timeout)
    
    def wait_all(self, timeout: float = 30.0) -> bool:
        """Wait for all events (created, shown, loaded, bridge_ready).
        
        Args:
            timeout: Maximum total time to wait in seconds
            
        Returns:
            True if all events were set, False if timeout occurred
        """
        import time
        start = time.monotonic()
        remaining = timeout
        
        for event in [self.created, self.shown, self.loaded, self.bridge_ready]:
            if not event.wait(remaining):
                return False
            remaining = timeout - (time.monotonic() - start)
            if remaining <= 0:
                return False
        
        return True
    
    def reset(self) -> None:
        """Reset all events to unset state."""
        self.created.clear()
        self.shown.clear()
        self.loaded.clear()
        self.bridge_ready.clear()
    
    def is_ready(self) -> bool:
        """Check if all events are set."""
        return all([
            self.created.is_set(),
            self.shown.is_set(),
            self.loaded.is_set(),
            self.bridge_ready.is_set(),
        ])


# Decorators for automatic waiting

def require_created(func: F) -> F:
    """Decorator to ensure WebView is created before executing.
    
    Example:
        >>> class MyWebView(WebView):
        ...     @require_created
        ...     def custom_method(self):
        ...         pass
    """
    @wraps(func)
    def wrapper(self: "WebView", *args: Any, **kwargs: Any) -> Any:
        if not self._ready_events.wait_created(timeout=20):
            raise RuntimeError("WebView failed to create within timeout")
        return func(self, *args, **kwargs)
    return wrapper  # type: ignore


def require_shown(func: F) -> F:
    """Decorator to ensure window is shown before executing.
    
    Example:
        >>> class MyWebView(WebView):
        ...     @require_shown
        ...     def capture_screenshot(self):
        ...         pass
    """
    @wraps(func)
    def wrapper(self: "WebView", *args: Any, **kwargs: Any) -> Any:
        if not self._ready_events.wait_shown(timeout=20):
            raise RuntimeError("WebView failed to show within timeout")
        return func(self, *args, **kwargs)
    return wrapper  # type: ignore


def require_loaded(func: F) -> F:
    """Decorator to ensure page is loaded before executing.
    
    Example:
        >>> class MyWebView(WebView):
        ...     @require_loaded
        ...     def evaluate_js(self, script):
        ...         pass
    """
    @wraps(func)
    def wrapper(self: "WebView", *args: Any, **kwargs: Any) -> Any:
        if not self._ready_events.wait_loaded(timeout=20):
            raise RuntimeError("WebView failed to load within timeout")
        return func(self, *args, **kwargs)
    return wrapper  # type: ignore


def require_bridge_ready(func: F) -> F:
    """Decorator to ensure JS bridge is ready before executing.
    
    Example:
        >>> class MyWebView(WebView):
        ...     @require_bridge_ready
        ...     def call_js_api(self, method, params):
        ...         pass
    """
    @wraps(func)
    def wrapper(self: "WebView", *args: Any, **kwargs: Any) -> Any:
        if not self._ready_events.wait_bridge_ready(timeout=20):
            raise RuntimeError("JS bridge failed to initialize within timeout")
        return func(self, *args, **kwargs)
    return wrapper  # type: ignore
```

#### 使用示例

```python
from auroraview import create_webview
from auroraview.core.ready_events import require_loaded

# 自动等待
webview = create_webview(url="https://example.com")
webview.show()

# 手动等待
if webview._ready_events.wait_loaded(timeout=10):
    result = webview.evaluate_js("document.title")
    print(f"Page title: {result}")

# 使用装饰器
class MyWebView(WebView):
    @require_loaded
    def get_page_info(self):
        return self.evaluate_js("({title: document.title, url: location.href})")
```

### 组件 3: TabContainer (多标签容器)

#### 设计目标

- 抽象标签状态管理
- 与 WindowManager 集成
- 支持懒加载 WebView
- 提供事件回调

#### API 设计

```python
# python/auroraview/browser/tab_container.py

from __future__ import annotations
from typing import TYPE_CHECKING, Dict, List, Optional, Callable, Any
from dataclasses import dataclass, field
from threading import Lock
from uuid import uuid4

if TYPE_CHECKING:
    from auroraview import WebView

from auroraview.core.window_manager import get_window_manager


@dataclass
class TabState:
    """State for a single tab.
    
    Attributes:
        id: Unique tab identifier
        title: Tab title (displayed in tab bar)
        url: Current URL
        favicon: Favicon URL or data URI
        is_loading: Whether the page is loading
        can_go_back: Whether back navigation is available
        can_go_forward: Whether forward navigation is available
        webview_id: Reference to WindowManager (None if not loaded)
        metadata: Custom metadata storage
    """
    id: str
    title: str = "New Tab"
    url: str = ""
    favicon: str = ""
    is_loading: bool = False
    can_go_back: bool = False
    can_go_forward: bool = False
    webview_id: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            "id": self.id,
            "title": self.title,
            "url": self.url,
            "favicon": self.favicon,
            "isLoading": self.is_loading,
            "canGoBack": self.can_go_back,
            "canGoForward": self.can_go_forward,
            "metadata": self.metadata,
        }


class TabContainer:
    """Container for managing multiple tabs with WebView windows.
    
    This provides a foundation for building tabbed browsers, multi-panel
    DCC tools, and other multi-webview applications.
    
    Features:
    - Tab state management (create, close, activate)
    - Lazy WebView loading
    - Navigation controls
    - Event callbacks for UI updates
    
    Example:
        >>> container = TabContainer(
        ...     on_tabs_update=lambda tabs: print(f"Tabs: {len(tabs)}"),
        ...     default_url="https://example.com"
        ... )
        >>> tab = container.create_tab("https://github.com")
        >>> container.navigate("https://google.com")
        >>> container.close_tab(tab.id)
    """
    
    def __init__(
        self,
        on_tab_change: Optional[Callable[[TabState], None]] = None,
        on_tabs_update: Optional[Callable[[List[TabState]], None]] = None,
        default_url: str = "",
        webview_factory: Optional[Callable[..., "WebView"]] = None,
        webview_options: Optional[Dict[str, Any]] = None,
    ):
        """Initialize TabContainer.
        
        Args:
            on_tab_change: Callback when active tab changes
            on_tabs_update: Callback when tab list changes
            default_url: Default URL for new tabs
            webview_factory: Custom factory for creating WebViews
            webview_options: Options passed to WebView creation
        """
        self._tabs: Dict[str, TabState] = {}
        self._tab_order: List[str] = []  # Maintain insertion order
        self._active_tab_id: Optional[str] = None
        self._on_tab_change = on_tab_change
        self._on_tabs_update = on_tabs_update
        self._default_url = default_url
        self._webview_factory = webview_factory
        self._webview_options = webview_options or {}
        self._lock = Lock()
        self._wm = get_window_manager()
    
    def create_tab(
        self, 
        url: str = "", 
        title: str = "New Tab",
        activate: bool = True,
        load_immediately: bool = True,
    ) -> TabState:
        """Create a new tab.
        
        Args:
            url: Initial URL (uses default_url if empty)
            title: Initial tab title
            activate: Whether to activate the new tab
            load_immediately: Whether to create WebView immediately
            
        Returns:
            The created TabState
        """
        tab_id = f"tab_{uuid4().hex[:8]}"
        tab = TabState(
            id=tab_id, 
            url=url or self._default_url,
            title=title,
        )
        
        with self._lock:
            self._tabs[tab_id] = tab
            self._tab_order.append(tab_id)
            
            if activate or self._active_tab_id is None:
                self._active_tab_id = tab_id
        
        if load_immediately and tab.url:
            self._load_tab_webview(tab)
        
        self._notify_tabs_update()
        
        if activate and self._on_tab_change:
            self._on_tab_change(tab)
        
        return tab
    
    def close_tab(self, tab_id: str) -> Optional[str]:
        """Close a tab.
        
        Args:
            tab_id: ID of the tab to close
            
        Returns:
            ID of the new active tab, or None if no tabs remain
        """
        with self._lock:
            if tab_id not in self._tabs:
                return self._active_tab_id
            
            tab = self._tabs.pop(tab_id)
            self._tab_order.remove(tab_id)
            
            # Close WebView if exists
            if tab.webview_id:
                webview = self._wm.get(tab.webview_id)
                if webview:
                    try:
                        webview.close()
                    except Exception:
                        pass
            
            # Select new active tab
            if self._active_tab_id == tab_id:
                if self._tab_order:
                    # Try to select adjacent tab
                    self._active_tab_id = self._tab_order[-1]
                else:
                    self._active_tab_id = None
        
        self._notify_tabs_update()
        
        if self._active_tab_id and self._on_tab_change:
            self._on_tab_change(self._tabs[self._active_tab_id])
        
        return self._active_tab_id
    
    def activate_tab(self, tab_id: str) -> bool:
        """Activate a tab.
        
        Args:
            tab_id: ID of the tab to activate
            
        Returns:
            True if tab was found and activated
        """
        with self._lock:
            if tab_id not in self._tabs:
                return False
            
            if self._active_tab_id == tab_id:
                return True
            
            old_tab = self._tabs.get(self._active_tab_id)
            new_tab = self._tabs[tab_id]
            
            # Hide old tab's webview
            if old_tab and old_tab.webview_id:
                webview = self._wm.get(old_tab.webview_id)
                if webview:
                    webview.hide()
            
            # Show/load new tab's webview
            if new_tab.webview_id:
                webview = self._wm.get(new_tab.webview_id)
                if webview:
                    webview.show(wait=False)
            elif new_tab.url:
                self._load_tab_webview(new_tab)
            
            self._active_tab_id = tab_id
        
        if self._on_tab_change:
            self._on_tab_change(new_tab)
        
        self._notify_tabs_update()
        return True
    
    def navigate(self, url: str, tab_id: Optional[str] = None) -> bool:
        """Navigate a tab to a URL.
        
        Args:
            url: URL to navigate to
            tab_id: Tab ID (uses active tab if None)
            
        Returns:
            True if navigation was initiated
        """
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return False
        
        with self._lock:
            tab = self._tabs.get(tab_id)
            if not tab:
                return False
            
            tab.url = url
            tab.is_loading = True
            
            if tab.webview_id:
                webview = self._wm.get(tab.webview_id)
                if webview:
                    webview.load_url(url)
            else:
                self._load_tab_webview(tab)
        
        self._notify_tabs_update()
        return True
    
    def go_back(self, tab_id: Optional[str] = None) -> bool:
        """Go back in the specified tab."""
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return False
        
        tab = self._tabs.get(tab_id)
        if not tab or not tab.webview_id:
            return False
        
        webview = self._wm.get(tab.webview_id)
        if webview and tab.can_go_back:
            webview.go_back()
            return True
        return False
    
    def go_forward(self, tab_id: Optional[str] = None) -> bool:
        """Go forward in the specified tab."""
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return False
        
        tab = self._tabs.get(tab_id)
        if not tab or not tab.webview_id:
            return False
        
        webview = self._wm.get(tab.webview_id)
        if webview and tab.can_go_forward:
            webview.go_forward()
            return True
        return False
    
    def reload(self, tab_id: Optional[str] = None) -> bool:
        """Reload the specified tab."""
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return False
        
        tab = self._tabs.get(tab_id)
        if not tab or not tab.webview_id:
            return False
        
        webview = self._wm.get(tab.webview_id)
        if webview:
            webview.reload()
            tab.is_loading = True
            self._notify_tabs_update()
            return True
        return False
    
    def update_tab(self, tab_id: str, **kwargs: Any) -> bool:
        """Update tab properties.
        
        Args:
            tab_id: Tab ID
            **kwargs: Properties to update (title, favicon, metadata, etc.)
            
        Returns:
            True if tab was found and updated
        """
        with self._lock:
            tab = self._tabs.get(tab_id)
            if not tab:
                return False
            
            for key, value in kwargs.items():
                if hasattr(tab, key):
                    setattr(tab, key, value)
        
        self._notify_tabs_update()
        return True
    
    def get_tab(self, tab_id: str) -> Optional[TabState]:
        """Get a tab by ID."""
        return self._tabs.get(tab_id)
    
    def get_active_tab(self) -> Optional[TabState]:
        """Get the active tab."""
        if self._active_tab_id:
            return self._tabs.get(self._active_tab_id)
        return None
    
    def get_active_tab_id(self) -> Optional[str]:
        """Get the active tab ID."""
        return self._active_tab_id
    
    def get_all_tabs(self) -> List[TabState]:
        """Get all tabs in order."""
        return [self._tabs[tid] for tid in self._tab_order if tid in self._tabs]
    
    def get_tab_count(self) -> int:
        """Get the number of tabs."""
        return len(self._tabs)
    
    def get_webview(self, tab_id: Optional[str] = None) -> Optional["WebView"]:
        """Get the WebView for a tab.
        
        Args:
            tab_id: Tab ID (uses active tab if None)
            
        Returns:
            The WebView instance, or None
        """
        tab_id = tab_id or self._active_tab_id
        if not tab_id:
            return None
        
        tab = self._tabs.get(tab_id)
        if not tab or not tab.webview_id:
            return None
        
        return self._wm.get(tab.webview_id)
    
    def _load_tab_webview(self, tab: TabState) -> None:
        """Create and load a WebView for a tab."""
        from auroraview import create_webview
        
        factory = self._webview_factory or create_webview
        webview = factory(
            url=tab.url,
            auto_show=False,
            **self._webview_options
        )
        
        tab.webview_id = self._wm.register(webview)
        self._setup_webview_events(tab, webview)
        
        # Show if this is the active tab
        if self._active_tab_id == tab.id:
            webview.show(wait=False)
    
    def _setup_webview_events(self, tab: TabState, webview: "WebView") -> None:
        """Set up event handlers for a tab's WebView."""
        
        @webview.on("page:load_start")
        def on_load_start(data: Any) -> None:
            tab.is_loading = True
            self._notify_tabs_update()
        
        @webview.on("page:load_finish")
        def on_load_finish(data: Any) -> None:
            tab.is_loading = False
            if isinstance(data, dict):
                if data.get("url"):
                    tab.url = data["url"]
                if data.get("title"):
                    tab.title = data["title"]
                tab.can_go_back = data.get("canGoBack", False)
                tab.can_go_forward = data.get("canGoForward", False)
            self._notify_tabs_update()
        
        @webview.on("page:title_changed")
        def on_title_changed(data: Any) -> None:
            if isinstance(data, dict) and data.get("title"):
                tab.title = data["title"]
                self._notify_tabs_update()
        
        @webview.on("page:favicon_changed")
        def on_favicon_changed(data: Any) -> None:
            if isinstance(data, dict) and data.get("favicon"):
                tab.favicon = data["favicon"]
                self._notify_tabs_update()
        
        @webview.on("navigation:state_changed")
        def on_nav_state(data: Any) -> None:
            if isinstance(data, dict):
                tab.can_go_back = data.get("canGoBack", tab.can_go_back)
                tab.can_go_forward = data.get("canGoForward", tab.can_go_forward)
                self._notify_tabs_update()
        
        @webview.on("closing")
        def on_closing(data: Any) -> None:
            self.close_tab(tab.id)
    
    def _notify_tabs_update(self) -> None:
        """Notify tabs update callback."""
        if self._on_tabs_update:
            try:
                self._on_tabs_update(self.get_all_tabs())
            except Exception:
                pass
```

### 组件 4: Browser (高级浏览器 API)

#### 设计目标

- 简化多标签浏览器开发
- 提供开箱即用的 UI 控制器
- 支持自定义扩展

#### API 设计

```python
# python/auroraview/browser/__init__.py

from __future__ import annotations
from typing import TYPE_CHECKING, Optional, List, Dict, Any, Callable
from pathlib import Path

if TYPE_CHECKING:
    from auroraview import WebView

from .tab_container import TabContainer, TabState


class Browser:
    """High-level browser API with multi-tab support.
    
    Provides a complete tabbed browser experience with:
    - Tab management (create, close, switch)
    - Navigation controls (back, forward, reload)
    - URL bar and search
    - Customizable UI
    
    Example:
        >>> browser = Browser(title="My Browser")
        >>> browser.new_tab("https://google.com")
        >>> browser.new_tab("https://github.com")
        >>> browser.run()  # Blocking
    
    For DCC integration:
        >>> browser = Browser(parent=maya_widget)
        >>> browser.new_tab("https://docs.autodesk.com")
        >>> browser.show()  # Non-blocking
    """
    
    def __init__(
        self,
        title: str = "AuroraView Browser",
        width: int = 1200,
        height: int = 800,
        debug: bool = False,
        parent: Any = None,
        default_url: str = "",
        controller_html: Optional[str] = None,
        controller_height: int = 60,
    ):
        """Initialize Browser.
        
        Args:
            title: Window title
            width: Window width
            height: Window height
            debug: Enable developer tools
            parent: Parent widget for DCC integration
            default_url: Default URL for new tabs
            controller_html: Custom HTML for tab bar UI
            controller_height: Height of the controller/tab bar
        """
        self.title = title
        self.width = width
        self.height = height
        self.debug = debug
        self.parent = parent
        self.default_url = default_url
        self._controller_html = controller_html
        self._controller_height = controller_height
        
        self._tabs = TabContainer(
            on_tabs_update=self._on_tabs_update,
            on_tab_change=self._on_tab_change,
            default_url=default_url,
            webview_options={
                "width": width,
                "height": height - controller_height,
                "debug": debug,
                "parent": parent,
            }
        )
        
        self._controller: Optional["WebView"] = None
        self._running = False
        self._on_ready_callbacks: List[Callable[["Browser"], None]] = []
    
    def new_tab(self, url: str = "", title: str = "New Tab") -> TabState:
        """Create a new tab.
        
        Args:
            url: Initial URL
            title: Tab title
            
        Returns:
            The created TabState
        """
        return self._tabs.create_tab(url=url, title=title)
    
    def close_tab(self, tab_id: Optional[str] = None) -> None:
        """Close a tab.
        
        Args:
            tab_id: Tab ID (closes active tab if None)
        """
        tab_id = tab_id or self._tabs.get_active_tab_id()
        if tab_id:
            self._tabs.close_tab(tab_id)
    
    def activate_tab(self, tab_id: str) -> None:
        """Activate a tab by ID."""
        self._tabs.activate_tab(tab_id)
    
    def navigate(self, url: str, tab_id: Optional[str] = None) -> None:
        """Navigate a tab to a URL.
        
        Args:
            url: URL to navigate to
            tab_id: Tab ID (uses active tab if None)
        """
        self._tabs.navigate(url, tab_id)
    
    def go_back(self) -> None:
        """Go back in the active tab."""
        self._tabs.go_back()
    
    def go_forward(self) -> None:
        """Go forward in the active tab."""
        self._tabs.go_forward()
    
    def reload(self) -> None:
        """Reload the active tab."""
        self._tabs.reload()
    
    def get_tabs(self) -> List[TabState]:
        """Get all tabs."""
        return self._tabs.get_all_tabs()
    
    def get_active_tab(self) -> Optional[TabState]:
        """Get the active tab."""
        return self._tabs.get_active_tab()
    
    def on_ready(self, callback: Callable[["Browser"], None]) -> None:
        """Register a callback for when browser is ready.
        
        Args:
            callback: Function to call when browser is ready
        """
        if self._running:
            callback(self)
        else:
            self._on_ready_callbacks.append(callback)
    
    def run(self) -> None:
        """Run the browser (blocking).
        
        This creates the controller window and starts the event loop.
        """
        self._create_controller()
        self._running = True
        
        for callback in self._on_ready_callbacks:
            try:
                callback(self)
            except Exception:
                pass
        
        self._controller.show()
    
    def show(self) -> None:
        """Show the browser (non-blocking).
        
        Use this for DCC integration where the host manages the event loop.
        """
        self._create_controller()
        self._running = True
        
        for callback in self._on_ready_callbacks:
            try:
                callback(self)
            except Exception:
                pass
        
        self._controller.show(wait=False)
    
    def close(self) -> None:
        """Close the browser and all tabs."""
        # Close all tabs
        for tab in self._tabs.get_all_tabs():
            self._tabs.close_tab(tab.id)
        
        # Close controller
        if self._controller:
            self._controller.close()
        
        self._running = False
    
    def _create_controller(self) -> None:
        """Create the controller/tab bar window."""
        from auroraview import create_webview
        
        self._controller = create_webview(
            title=self.title,
            html=self._controller_html or self._get_default_controller_html(),
            width=self.width,
            height=self._controller_height,
            debug=self.debug,
            parent=self.parent,
            auto_show=False,
        )
        
        self._setup_controller_api()
    
    def _setup_controller_api(self) -> None:
        """Set up API bindings for the controller."""
        
        @self._controller.bind_call("browser.new_tab")
        def new_tab(url: str = "", title: str = "New Tab") -> Dict[str, Any]:
            tab = self.new_tab(url, title)
            return {"tabId": tab.id, "success": True}
        
        @self._controller.bind_call("browser.close_tab")
        def close_tab(tabId: str = "") -> Dict[str, Any]:
            self.close_tab(tabId or None)
            return {"success": True}
        
        @self._controller.bind_call("browser.activate_tab")
        def activate_tab(tabId: str) -> Dict[str, Any]:
            self.activate_tab(tabId)
            return {"success": True}
        
        @self._controller.bind_call("browser.navigate")
        def navigate(url: str, tabId: str = "") -> Dict[str, Any]:
            self.navigate(url, tabId or None)
            return {"success": True}
        
        @self._controller.bind_call("browser.go_back")
        def go_back() -> Dict[str, Any]:
            self.go_back()
            return {"success": True}
        
        @self._controller.bind_call("browser.go_forward")
        def go_forward() -> Dict[str, Any]:
            self.go_forward()
            return {"success": True}
        
        @self._controller.bind_call("browser.reload")
        def reload() -> Dict[str, Any]:
            self.reload()
            return {"success": True}
        
        @self._controller.bind_call("browser.get_tabs")
        def get_tabs() -> Dict[str, Any]:
            tabs = self.get_tabs()
            active = self._tabs.get_active_tab_id()
            return {
                "tabs": [t.to_dict() for t in tabs],
                "activeTabId": active,
            }
    
    def _on_tabs_update(self, tabs: List[TabState]) -> None:
        """Handle tabs update."""
        if self._controller:
            self._controller.emit("browser:tabs_update", {
                "tabs": [t.to_dict() for t in tabs],
                "activeTabId": self._tabs.get_active_tab_id(),
            })
    
    def _on_tab_change(self, tab: TabState) -> None:
        """Handle active tab change."""
        if self._controller:
            self._controller.emit("browser:tab_changed", tab.to_dict())
    
    def _get_default_controller_html(self) -> str:
        """Get default controller HTML."""
        return """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: #2d2d2d;
            color: #fff;
            height: 100vh;
            display: flex;
            flex-direction: column;
        }
        .tab-bar {
            display: flex;
            background: #3d3d3d;
            padding: 4px;
            gap: 2px;
            overflow-x: auto;
        }
        .tab {
            display: flex;
            align-items: center;
            padding: 6px 12px;
            background: #4d4d4d;
            border-radius: 4px;
            cursor: pointer;
            min-width: 120px;
            max-width: 200px;
        }
        .tab.active { background: #5d5d5d; }
        .tab:hover { background: #5d5d5d; }
        .tab-title {
            flex: 1;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            font-size: 12px;
        }
        .tab-close {
            margin-left: 8px;
            opacity: 0.6;
            cursor: pointer;
        }
        .tab-close:hover { opacity: 1; }
        .new-tab {
            padding: 6px 12px;
            cursor: pointer;
            opacity: 0.6;
        }
        .new-tab:hover { opacity: 1; }
        .toolbar {
            display: flex;
            padding: 4px;
            gap: 4px;
            background: #2d2d2d;
        }
        .nav-btn {
            padding: 4px 8px;
            background: #4d4d4d;
            border: none;
            border-radius: 4px;
            color: #fff;
            cursor: pointer;
        }
        .nav-btn:hover { background: #5d5d5d; }
        .nav-btn:disabled { opacity: 0.3; cursor: not-allowed; }
        .url-bar {
            flex: 1;
            padding: 4px 8px;
            background: #1d1d1d;
            border: none;
            border-radius: 4px;
            color: #fff;
            font-size: 13px;
        }
        .url-bar:focus { outline: 1px solid #0078d4; }
    </style>
</head>
<body>
    <div class="tab-bar" id="tabBar">
        <div class="new-tab" onclick="newTab()">+</div>
    </div>
    <div class="toolbar">
        <button class="nav-btn" id="backBtn" onclick="goBack()">←</button>
        <button class="nav-btn" id="forwardBtn" onclick="goForward()">→</button>
        <button class="nav-btn" onclick="reload()">↻</button>
        <input class="url-bar" id="urlBar" placeholder="Enter URL..." 
               onkeypress="if(event.key==='Enter')navigate()">
    </div>
    <script>
        let tabs = [];
        let activeTabId = null;
        
        window.addEventListener('auroraviewready', () => {
            auroraview.on('browser:tabs_update', updateTabs);
            auroraview.on('browser:tab_changed', onTabChanged);
            auroraview.call('browser.get_tabs').then(updateTabs);
        });
        
        function updateTabs(data) {
            tabs = data.tabs || [];
            activeTabId = data.activeTabId;
            renderTabs();
            updateNavState();
        }
        
        function onTabChanged(tab) {
            document.getElementById('urlBar').value = tab.url || '';
            updateNavState();
        }
        
        function renderTabs() {
            const bar = document.getElementById('tabBar');
            bar.innerHTML = tabs.map(t => `
                <div class="tab ${t.id === activeTabId ? 'active' : ''}" 
                     onclick="activateTab('${t.id}')">
                    <span class="tab-title">${t.title || 'New Tab'}</span>
                    <span class="tab-close" onclick="event.stopPropagation();closeTab('${t.id}')">×</span>
                </div>
            `).join('') + '<div class="new-tab" onclick="newTab()">+</div>';
        }
        
        function updateNavState() {
            const active = tabs.find(t => t.id === activeTabId);
            document.getElementById('backBtn').disabled = !active?.canGoBack;
            document.getElementById('forwardBtn').disabled = !active?.canGoForward;
        }
        
        function newTab() { auroraview.call('browser.new_tab', {url: ''}); }
        function closeTab(id) { auroraview.call('browser.close_tab', {tabId: id}); }
        function activateTab(id) { auroraview.call('browser.activate_tab', {tabId: id}); }
        function goBack() { auroraview.call('browser.go_back'); }
        function goForward() { auroraview.call('browser.go_forward'); }
        function reload() { auroraview.call('browser.reload'); }
        function navigate() {
            const url = document.getElementById('urlBar').value;
            if (url) auroraview.call('browser.navigate', {url});
        }
    </script>
</body>
</html>
"""
```

## 向后兼容性

### 兼容策略

1. **新增 API，不修改现有 API**
   - `WindowManager` 是新增模块，不影响现有代码
   - `ReadyEvents` 作为可选增强，不改变现有行为
   - `TabContainer` 和 `Browser` 是新增高级 API

2. **渐进式集成**
   - 现有 `WebView` 自动注册到 `WindowManager`
   - 不使用新功能的代码无需任何修改

3. **可选启用**
   - 装饰器 (`require_loaded` 等) 是可选的
   - `Browser` API 是独立的高级封装

### 迁移路径

```python
# 现有代码 - 无需修改，继续工作
from auroraview import create_webview
wv = create_webview(url="https://example.com")
wv.show()

# 新代码 - 使用 WindowManager
from auroraview import create_webview
from auroraview.core.window_manager import get_windows

wv1 = create_webview(url="https://example.com")
wv2 = create_webview(url="https://github.com")

# 获取所有窗口
all_windows = get_windows()

# 新代码 - 使用 Browser API
from auroraview.browser import Browser

browser = Browser()
browser.new_tab("https://google.com")
browser.run()
```

## 实现计划

### Phase 1: 核心基础 (v0.6.0)

- [ ] **WindowManager** 实现
  - [ ] 单例模式
  - [ ] 窗口注册/注销
  - [ ] 活动窗口管理
  - [ ] 变更通知回调
  - [ ] 广播事件

- [ ] **ReadyEvents** 实现
  - [ ] 生命周期事件
  - [ ] 等待方法
  - [ ] 装饰器

- [ ] **WebView 集成**
  - [ ] 自动注册到 WindowManager
  - [ ] 触发 ReadyEvents

- [ ] **单元测试**
  - [ ] WindowManager 测试
  - [ ] ReadyEvents 测试
  - [ ] 集成测试

### Phase 2: 多标签支持 (v0.6.1)

- [ ] **TabContainer** 实现
  - [ ] 标签状态管理
  - [ ] 懒加载 WebView
  - [ ] 导航控制
  - [ ] 事件回调

- [ ] **Browser** 实现
  - [ ] 高级 API
  - [ ] 默认 UI
  - [ ] API 绑定

- [ ] **文档更新**
  - [ ] API 文档
  - [ ] 使用指南
  - [ ] 示例代码

### Phase 3: DCC 集成 (v0.7.0)

- [ ] **Maya 多面板**
  - [ ] 面板管理器
  - [ ] 停靠支持

- [ ] **Houdini 多面板**
  - [ ] 面板管理器
  - [ ] 工具架集成

- [ ] **跨面板通信**
  - [ ] 事件广播
  - [ ] 状态同步

## 测试计划

### 单元测试

```python
# tests/python/unit/test_window_manager.py

def test_singleton():
    """WindowManager should be singleton."""
    wm1 = get_window_manager()
    wm2 = get_window_manager()
    assert wm1 is wm2

def test_register_unregister():
    """Test window registration."""
    wm = get_window_manager()
    webview = Mock()
    
    uid = wm.register(webview)
    assert wm.get(uid) is webview
    
    wm.unregister(uid)
    assert wm.get(uid) is None

def test_active_window():
    """Test active window tracking."""
    wm = get_window_manager()
    wv1 = Mock()
    wv2 = Mock()
    
    uid1 = wm.register(wv1)
    uid2 = wm.register(wv2)
    
    assert wm.get_active() is wv1  # First registered is active
    
    wm.set_active(uid2)
    assert wm.get_active() is wv2

def test_broadcast():
    """Test event broadcasting."""
    wm = get_window_manager()
    wv1 = Mock()
    wv2 = Mock()
    
    wm.register(wv1)
    wm.register(wv2)
    
    wm.broadcast("test:event", {"data": 123})
    
    wv1.emit.assert_called_with("test:event", {"data": 123})
    wv2.emit.assert_called_with("test:event", {"data": 123})
```

### 集成测试

```python
# tests/python/integration/test_browser.py

def test_browser_tabs():
    """Test browser tab management."""
    browser = Browser()
    
    tab1 = browser.new_tab("https://example.com")
    tab2 = browser.new_tab("https://github.com")
    
    assert browser.get_tabs() == [tab1, tab2]
    assert browser.get_active_tab() == tab2
    
    browser.close_tab(tab2.id)
    assert browser.get_active_tab() == tab1
```

## 参考资料

- [AuroraView 架构文档](../guide/architecture.md)
- [WebView API 参考](../api/webview.md)
- [DCC 集成指南](../dcc/index.md)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-01-13 | Draft | 初始草案 |
