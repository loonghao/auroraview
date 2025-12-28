# 高级用法

本指南涵盖了高级用户的进阶模式和技术。

## 多窗口应用

### 窗口管理器

创建和管理多个窗口：

```python
from auroraview import WebView

class WindowManager:
    def __init__(self):
        self.windows = {}
    
    def create_window(self, name: str, **kwargs) -> WebView:
        webview = WebView.create(**kwargs)
        self.windows[name] = webview
        return webview
    
    def get_window(self, name: str) -> WebView:
        return self.windows.get(name)
    
    def close_all(self):
        for webview in self.windows.values():
            webview.close()
        self.windows.clear()

# 使用示例
manager = WindowManager()
main = manager.create_window("main", title="主窗口", url="...")
settings = manager.create_window("settings", title="设置", url="...")
```

### 跨窗口通信

```python
# 在主窗口中
@main_webview.on("open_settings")
def open_settings(data):
    settings_webview.show()
    settings_webview.emit("init_settings", data)

# 在设置窗口中
@settings_webview.on("save_settings")
def save_settings(data):
    main_webview.emit("settings_changed", data)
```

## 动态 API 绑定

### 运行时注册

```python
webview = WebView.create("插件宿主", url="...")

# 动态加载插件
for plugin in discover_plugins():
    for method_name, method in plugin.get_methods():
        webview.bind_call(f"plugin.{plugin.name}.{method_name}", method)
```

### 条件绑定

```python
config = load_config()

if config.get("enable_export"):
    @webview.bind_call("api.export")
    def export(format: str):
        return do_export(format)

if config.get("enable_import"):
    @webview.bind_call("api.import")
    def import_data(path: str):
        return do_import(path)
```

## 自定义事件系统

### 事件中间件

```python
class EventMiddleware:
    def __init__(self, webview):
        self.webview = webview
        self.handlers = {}
        self.middleware = []
    
    def use(self, fn):
        """添加中间件函数。"""
        self.middleware.append(fn)
        return fn
    
    def on(self, event: str):
        """注册事件处理器。"""
        def decorator(fn):
            self.handlers.setdefault(event, []).append(fn)
            return fn
        return decorator
    
    def dispatch(self, event: str, data: dict):
        """通过中间件链分发事件。"""
        # 运行中间件
        for mw in self.middleware:
            data = mw(event, data)
            if data is None:
                return  # 中间件取消了事件
        
        # 运行处理器
        for handler in self.handlers.get(event, []):
            handler(data)

# 使用示例
events = EventMiddleware(webview)

@events.use
def log_events(event, data):
    print(f"事件: {event}, 数据: {data}")
    return data

@events.use
def validate_events(event, data):
    if not data.get("user_id"):
        return None  # 取消事件
    return data

@events.on("user_action")
def handle_action(data):
    process_action(data)
```

### 类型化事件

```python
from dataclasses import dataclass
from typing import TypeVar, Generic, Callable

T = TypeVar('T')

@dataclass
class SelectionEvent:
    items: list[str]
    source: str

class TypedEventEmitter(Generic[T]):
    def __init__(self):
        self.handlers: list[Callable[[T], None]] = []
    
    def on(self, handler: Callable[[T], None]):
        self.handlers.append(handler)
    
    def emit(self, event: T):
        for handler in self.handlers:
            handler(event)

# 使用示例
selection_changed = TypedEventEmitter[SelectionEvent]()

@selection_changed.on
def handle_selection(event: SelectionEvent):
    print(f"从 {event.source} 选择了 {len(event.items)} 个项目")

selection_changed.emit(SelectionEvent(
    items=["mesh1", "mesh2"],
    source="outliner"
))
```

## 状态管理

### 响应式状态

```python
from auroraview import WebView

class ReactiveState:
    def __init__(self, webview: WebView, initial: dict = None):
        self._webview = webview
        self._state = initial or {}
        self._watchers = {}
    
    def __getitem__(self, key):
        return self._state.get(key)
    
    def __setitem__(self, key, value):
        old_value = self._state.get(key)
        self._state[key] = value
        
        # 同步到 JavaScript
        self._webview.eval_js(f"""
            window.auroraview.state['{key}'] = {json.dumps(value)};
        """)
        
        # 通知观察者
        for watcher in self._watchers.get(key, []):
            watcher(value, old_value)
    
    def watch(self, key: str, callback):
        self._watchers.setdefault(key, []).append(callback)

# 使用示例
state = ReactiveState(webview, {"count": 0, "theme": "dark"})

@state.watch("theme")
def on_theme_change(new_value, old_value):
    print(f"主题从 {old_value} 变为 {new_value}")

state["theme"] = "light"  # 触发观察者并同步到 JS
```

### 持久化状态

```python
import json
from pathlib import Path

class PersistentState:
    def __init__(self, path: Path):
        self.path = path
        self._state = self._load()
    
    def _load(self) -> dict:
        if self.path.exists():
            return json.loads(self.path.read_text())
        return {}
    
    def _save(self):
        self.path.write_text(json.dumps(self._state, indent=2))
    
    def get(self, key, default=None):
        return self._state.get(key, default)
    
    def set(self, key, value):
        self._state[key] = value
        self._save()

# 使用示例
state = PersistentState(Path.home() / ".myapp" / "state.json")
state.set("last_project", "/path/to/project")
```

## 错误处理

### 全局错误处理器

```python
import traceback

class ErrorHandler:
    def __init__(self, webview: WebView):
        self.webview = webview
    
    def wrap(self, fn):
        """捕获和报告错误的装饰器。"""
        def wrapper(*args, **kwargs):
            try:
                return fn(*args, **kwargs)
            except Exception as e:
                self.report_error(e)
                raise
        return wrapper
    
    def report_error(self, error: Exception):
        """将错误发送到前端。"""
        self.webview.emit("error", {
            "type": type(error).__name__,
            "message": str(error),
            "traceback": traceback.format_exc()
        })

# 使用示例
error_handler = ErrorHandler(webview)

@webview.bind_call("api.risky_operation")
@error_handler.wrap
def risky_operation():
    # 如果抛出异常，错误会发送到前端
    do_something_risky()
```

### 重试逻辑

```python
import time
from functools import wraps

def retry(max_attempts=3, delay=1.0, backoff=2.0):
    def decorator(fn):
        @wraps(fn)
        def wrapper(*args, **kwargs):
            last_error = None
            current_delay = delay
            
            for attempt in range(max_attempts):
                try:
                    return fn(*args, **kwargs)
                except Exception as e:
                    last_error = e
                    if attempt < max_attempts - 1:
                        time.sleep(current_delay)
                        current_delay *= backoff
            
            raise last_error
        return wrapper
    return decorator

@webview.bind_call("api.fetch_data")
@retry(max_attempts=3, delay=0.5)
def fetch_data(url: str):
    return requests.get(url).json()
```

## 性能优化

### 事件防抖

```python
import threading
from functools import wraps

def debounce(wait: float):
    def decorator(fn):
        timer = None
        
        @wraps(fn)
        def wrapper(*args, **kwargs):
            nonlocal timer
            
            def call_fn():
                fn(*args, **kwargs)
            
            if timer:
                timer.cancel()
            timer = threading.Timer(wait, call_fn)
            timer.start()
        
        return wrapper
    return decorator

@webview.on("search_input")
@debounce(0.3)  # 最后一次输入后等待 300ms
def handle_search(data):
    results = search(data["query"])
    webview.emit("search_results", results)
```

### 节流

```python
import time
from functools import wraps

def throttle(interval: float):
    def decorator(fn):
        last_call = 0
        
        @wraps(fn)
        def wrapper(*args, **kwargs):
            nonlocal last_call
            now = time.time()
            
            if now - last_call >= interval:
                last_call = now
                return fn(*args, **kwargs)
        
        return wrapper
    return decorator

@webview.on("viewport_update")
@throttle(0.016)  # 最大 60fps
def handle_viewport_update(data):
    update_viewport(data)
```

### 延迟加载

```python
class LazyLoader:
    def __init__(self, webview: WebView):
        self.webview = webview
        self.loaded_modules = set()
    
    def load_module(self, module_name: str):
        if module_name in self.loaded_modules:
            return
        
        # 加载 Python 模块
        module = importlib.import_module(f"plugins.{module_name}")
        
        # 注册 API
        for name, method in inspect.getmembers(module, inspect.isfunction):
            if hasattr(method, "_api_method"):
                self.webview.bind_call(f"{module_name}.{name}", method)
        
        self.loaded_modules.add(module_name)
        self.webview.emit("module_loaded", {"name": module_name})

# 使用示例
loader = LazyLoader(webview)

@webview.on("load_module")
def on_load_module(data):
    loader.load_module(data["name"])
```

## 测试模式

### Mock WebView

```python
class MockWebView:
    def __init__(self):
        self.events = []
        self.js_calls = []
        self.handlers = {}
    
    def emit(self, event: str, data: dict):
        self.events.append((event, data))
    
    def eval_js(self, script: str):
        self.js_calls.append(script)
    
    def on(self, event: str):
        def decorator(fn):
            self.handlers[event] = fn
            return fn
        return decorator
    
    def simulate_event(self, event: str, data: dict):
        if event in self.handlers:
            self.handlers[event](data)

# 测试
def test_my_tool():
    mock = MockWebView()
    tool = MyTool(mock)
    
    mock.simulate_event("button_click", {"id": "save"})
    
    assert ("save_complete", {"success": True}) in mock.events
```

### 集成测试

```python
import pytest
from auroraview.testing import HeadlessWebView

@pytest.fixture
def webview():
    with HeadlessWebView.playwright() as wv:
        yield wv

def test_api_call(webview):
    webview.load_html("""
        <script>
            async function test() {
                const result = await auroraview.api.get_data();
                document.body.textContent = JSON.stringify(result);
            }
            test();
        </script>
    """)
    
    webview.wait_for("body")
    assert webview.text("body") == '{"items":[1,2,3]}'
```

## 安全最佳实践

### 输入验证

```python
from pydantic import BaseModel, validator

class ExportRequest(BaseModel):
    path: str
    format: str
    
    @validator("path")
    def validate_path(cls, v):
        # 防止目录遍历
        if ".." in v:
            raise ValueError("无效路径")
        return v
    
    @validator("format")
    def validate_format(cls, v):
        allowed = ["fbx", "obj", "gltf"]
        if v not in allowed:
            raise ValueError(f"格式必须是以下之一: {allowed}")
        return v

@webview.bind_call("api.export")
def export(path: str = "", format: str = ""):
    request = ExportRequest(path=path, format=format)
    return do_export(request.path, request.format)
```

### 沙箱执行

```python
import ast

class SafeEval:
    ALLOWED_NODES = {
        ast.Expression, ast.Num, ast.Str, ast.List, ast.Dict,
        ast.BinOp, ast.Add, ast.Sub, ast.Mult, ast.Div,
    }
    
    def __init__(self):
        pass
    
    def eval(self, code: str):
        tree = ast.parse(code, mode='eval')
        self._validate(tree)
        return eval(compile(tree, '<string>', 'eval'))
    
    def _validate(self, node):
        if type(node) not in self.ALLOWED_NODES:
            raise ValueError(f"不允许的节点: {type(node).__name__}")
        for child in ast.iter_child_nodes(node):
            self._validate(child)

# 使用示例
safe_eval = SafeEval()
result = safe_eval.eval("1 + 2 * 3")  # OK
result = safe_eval.eval("__import__('os')")  # 抛出 ValueError
```

## 部署模式

### 配置管理

```python
import os
from pathlib import Path
from dataclasses import dataclass

@dataclass
class AppConfig:
    debug: bool = False
    api_url: str = "http://localhost:3000"
    log_level: str = "INFO"
    
    @classmethod
    def from_env(cls):
        return cls(
            debug=os.getenv("APP_DEBUG", "false").lower() == "true",
            api_url=os.getenv("APP_API_URL", "http://localhost:3000"),
            log_level=os.getenv("APP_LOG_LEVEL", "INFO"),
        )
    
    @classmethod
    def from_file(cls, path: Path):
        import tomllib
        data = tomllib.loads(path.read_text())
        return cls(**data)

# 使用示例
config = AppConfig.from_env()
webview = WebView.create(
    title="我的应用",
    url=config.api_url,
    debug=config.debug,
)
```

### 日志记录

```python
import logging
from datetime import datetime

def setup_logging(webview: WebView, level: str = "INFO"):
    # Python 日志
    logging.basicConfig(
        level=getattr(logging, level),
        format="%(asctime)s [%(levelname)s] %(message)s"
    )
    logger = logging.getLogger("myapp")
    
    # 转发到前端
    class WebViewHandler(logging.Handler):
        def emit(self, record):
            webview.emit("log", {
                "level": record.levelname,
                "message": record.getMessage(),
                "timestamp": datetime.now().isoformat()
            })
    
    logger.addHandler(WebViewHandler())
    return logger

# 使用示例
logger = setup_logging(webview, "DEBUG")
logger.info("应用已启动")
```
