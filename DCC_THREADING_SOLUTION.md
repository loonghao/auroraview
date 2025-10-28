# DCC 线程通信解决方案

## 🎯 问题分析

当前的实现中，WebView 运行在后台线程中，但当用户点击 WebView 中的按钮时，Maya 会卡住。这是因为：

1. **WebView 事件循环在后台线程** - WebView 的事件循环运行在独立的后台线程中
2. **JavaScript 回调直接调用 Python** - 当用户点击按钮时，JavaScript 调用 Python 函数
3. **Python 函数尝试调用 Maya API** - Python 函数需要调用 Maya 命令（如 `cmds.select()`）
4. **Maya API 不是线程安全的** - Maya 的 Python API 只能在主线程中调用
5. **结果：Maya 卡住** - 后台线程尝试从主线程调用 Maya API，导致死锁

## ✅ 解决方案：消息队列模式

使用 **线程安全的消息队列** 来实现线程间通信：

```
WebView (后台线程)
    ↓
JavaScript 事件
    ↓
Python 回调函数
    ↓
消息队列 (queue.Queue)
    ↓
Maya 主线程
    ↓
Maya API 调用
```

### 核心原理

1. **WebView 在后台线程运行** - 保持 Maya 主线程响应
2. **事件通过队列传递** - JavaScript 事件被放入线程安全的队列
3. **Maya 主线程轮询队列** - 定期检查队列中的消息
4. **在主线程执行 Maya 命令** - 所有 Maya API 调用都在主线程中进行

## 📋 实现步骤

### 步骤 1：创建消息队列管理器

```python
import queue
import threading
from typing import Callable, Any

class DCCEventQueue:
    """Thread-safe event queue for DCC integration."""
    
    def __init__(self):
        self._queue = queue.Queue()
        self._callbacks = {}
    
    def register_callback(self, event_name: str, callback: Callable):
        """Register a callback for an event."""
        self._callbacks[event_name] = callback
    
    def post_event(self, event_name: str, *args, **kwargs):
        """Post an event to the queue (thread-safe)."""
        self._queue.put((event_name, args, kwargs))
    
    def process_events(self):
        """Process all pending events (call from main thread)."""
        while not self._queue.empty():
            try:
                event_name, args, kwargs = self._queue.get_nowait()
                if event_name in self._callbacks:
                    self._callbacks[event_name](*args, **kwargs)
            except queue.Empty:
                break
```

### 步骤 2：修改 WebView 回调

```python
# 在 WebView 中注册事件处理器
event_queue = DCCEventQueue()

# 注册 Maya 命令回调
def on_select_object(obj_name):
    """This will be called from the main thread."""
    import maya.cmds as cmds
    cmds.select(obj_name)

event_queue.register_callback("select_object", on_select_object)

# JavaScript 调用时，发送事件到队列
def handle_js_event(event_data):
    """Called from WebView (background thread)."""
    event_queue.post_event("select_object", event_data["object"])
```

### 步骤 3：在 Maya 主线程中轮询队列

```python
import maya.cmds as cmds

def process_dcc_events():
    """Process DCC events from the queue."""
    event_queue.process_events()

# 使用 Maya 的定时器来定期处理事件
def setup_event_processing():
    """Setup periodic event processing in Maya."""
    # 使用 scriptJob 定期调用
    cmds.scriptJob(
        event=["idle", process_dcc_events],
        permanent=True
    )
```

## 🔄 完整工作流

```
1. 用户在 WebView 中点击按钮
   ↓
2. JavaScript 事件触发
   ↓
3. JavaScript 调用 Python 函数（在后台线程中）
   ↓
4. Python 函数将事件放入队列（线程安全）
   ↓
5. Maya 主线程定期检查队列
   ↓
6. 主线程从队列中取出事件
   ↓
7. 主线程执行 Maya API 调用
   ↓
8. Maya 保持响应，没有卡住
```

## 💡 关键优势

✓ **线程安全** - `queue.Queue` 是线程安全的
✓ **不阻塞 Maya** - WebView 事件循环在后台线程
✓ **Maya API 兼容** - 所有 Maya 调用都在主线程
✓ **可扩展** - 支持多个事件类型
✓ **低延迟** - 事件处理延迟很小（毫秒级）

## 📚 参考资源

- Python `queue.Queue` 文档：https://docs.python.org/3/library/queue.html
- Maya 脚本作业：https://help.autodesk.com/view/MAYAUL/2022/ENU/
- PyQt 线程通信：https://doc.qt.io/qt-6/qthread.html
- Flet 架构：https://github.com/flet-dev/flet

## 🎯 下一步

1. 实现 `DCCEventQueue` 类
2. 修改 WebView 回调以使用队列
3. 在 Maya 中设置事件处理循环
4. 测试线程安全性
5. 添加错误处理和日志记录

## ⚠️ 注意事项

- 不要在后台线程中调用 Maya API
- 始终使用队列进行线程间通信
- 定期处理队列中的事件
- 考虑事件处理的延迟
- 添加超时机制防止队列溢出

