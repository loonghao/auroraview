# EventTimer 线程安全修复

## 问题描述

在初始实现中，EventTimer 使用后台线程来调用 `process_events()`，导致了 Rust PyO3 的线程安全检查失败：

```
pyo3_runtime.PanicException: assertion `left == right` failed: 
auroraview_core::webview::aurora_view::AuroraView is unsendable, 
but sent to another thread
  left: ThreadId(3)
  right: ThreadId(1)
```

### 根本原因

Rust 的 `AuroraView` 对象通过 PyO3 绑定到 Python，但它**不是线程安全的**（`!Send`）。这意味着：

1. `AuroraView` 对象只能在创建它的线程中使用
2. 不能将 `AuroraView` 对象传递到其他线程
3. 不能在其他线程中调用 `AuroraView` 的方法

当 EventTimer 在后台线程中调用 `webview.process_events()` 时，实际上是在线程 3 中调用了在线程 1（主线程）中创建的 `AuroraView` 对象，违反了 Rust 的线程安全规则。

## 解决方案

### 方案：使用主线程事件循环

不使用后台线程，而是利用宿主应用的事件循环来调用 EventTimer 的 tick 方法：

1. **Maya 环境**：使用 `cmds.scriptJob(event=["idle", callback])`
2. **Qt 环境**：使用 `QTimer` 的 `timeout` 信号

这样所有的事件处理都在主线程中进行，确保线程安全。

### 实现对比

#### ❌ 旧实现（后台线程，线程不安全）

```python
class EventTimer:
    def start(self):
        self._running = True
        # 创建后台线程 - 线程不安全！
        self._thread = threading.Thread(target=self._run_loop, daemon=True)
        self._thread.start()
    
    def _run_loop(self):
        """在后台线程中运行"""
        while self._running:
            # ❌ 在线程 3 中调用线程 1 的对象 - 违反线程安全！
            should_close = self._webview.process_events()
            time.sleep(self._interval_ms / 1000.0)
```

#### ✅ 新实现（主线程，线程安全）

```python
class EventTimer:
    def start(self):
        self._running = True
        
        # 优先使用 Maya scriptJob（主线程）
        if self._try_start_maya_timer():
            return
        
        # 回退到 Qt QTimer（主线程）
        if self._try_start_qt_timer():
            return
        
        raise RuntimeError("No timer backend available")
    
    def _try_start_maya_timer(self) -> bool:
        """使用 Maya 的 idle 事件（主线程）"""
        try:
            import maya.cmds as cmds
            # ✅ Maya 会在主线程的 idle 时调用 self._tick
            job_id = cmds.scriptJob(event=["idle", self._tick], protected=True)
            self._timer_impl = job_id
            return True
        except:
            return False
    
    def _try_start_qt_timer(self) -> bool:
        """使用 Qt QTimer（主线程）"""
        try:
            from PySide2.QtCore import QTimer
            timer = QTimer()
            timer.setInterval(self._interval_ms)
            # ✅ Qt 会在主线程中触发 timeout 信号
            timer.timeout.connect(self._tick)
            timer.start()
            self._timer_impl = timer
            return True
        except:
            return False
    
    def _tick(self):
        """✅ 在主线程中运行，线程安全"""
        if not self._running:
            return
        
        # ✅ 在主线程中调用，线程安全
        should_close = self._webview.process_events()
        
        if should_close:
            self.stop()
            for callback in self._close_callbacks:
                callback()
```

## 技术细节

### Maya scriptJob 的工作原理

```python
import maya.cmds as cmds

def my_callback():
    print("Called in main thread")

# Maya 会在主线程的 idle 时调用 my_callback
job_id = cmds.scriptJob(event=["idle", my_callback])
```

- `idle` 事件在 Maya 主线程空闲时触发
- 回调函数在主线程中执行
- 不会创建新线程

### Qt QTimer 的工作原理

```python
from PySide2.QtCore import QTimer

def my_callback():
    print("Called in main thread")

timer = QTimer()
timer.setInterval(16)  # 16ms
timer.timeout.connect(my_callback)
timer.start()
```

- `timeout` 信号在 Qt 主事件循环中触发
- 回调函数在主线程中执行
- 不会创建新线程

## 优势

### 1. 线程安全

- ✅ 所有操作都在主线程中进行
- ✅ 符合 Rust PyO3 的线程安全要求
- ✅ 避免了跨线程调用的问题

### 2. 性能

- ✅ 无需线程同步开销
- ✅ 无需 GIL（全局解释器锁）竞争
- ✅ 更低的 CPU 使用率

### 3. 兼容性

- ✅ 与 Maya 的事件循环完美集成
- ✅ 与 Qt 的事件循环完美集成
- ✅ 自动选择最佳后端

### 4. 简洁性

- ✅ 无需管理线程生命周期
- ✅ 无需处理线程异常
- ✅ 代码更简洁

## 测试验证

### 测试场景

1. **Maya 环境**：
   ```python
   # 在 Maya Script Editor 中
   from maya_integration import maya_outliner
   outliner = maya_outliner.main()
   # 观察控制台输出，应该看到：
   # [MayaOutliner] EventTimer started with Maya scriptJob
   # [MayaOutliner] EventTimer tick #1
   # [MayaOutliner] EventTimer tick #2
   # ...
   ```

2. **Qt 环境**（无 Maya）：
   ```python
   from auroraview import WebView, EventTimer
   from PySide2.QtWidgets import QApplication
   
   app = QApplication([])
   webview = WebView(title="Test", width=800, height=600)
   webview.show()
   
   timer = EventTimer(webview, interval_ms=16)
   timer.start()
   # 应该看到：
   # EventTimer started with Qt QTimer
   
   app.exec_()
   ```

3. **关闭检测**：
   ```python
   @timer.on_close
   def handle_close():
       print("Window closed - callback in main thread")
   
   # 点击窗口关闭按钮，应该看到回调被调用
   ```

### 预期结果

- ✅ 无线程安全错误
- ✅ 事件正常处理
- ✅ 关闭检测正常工作
- ✅ 无内存泄漏

## 总结

通过将 EventTimer 从后台线程模式改为主线程事件循环模式，我们成功解决了 Rust PyO3 的线程安全问题。新实现：

1. **线程安全**：所有操作都在主线程中进行
2. **性能更好**：无线程同步开销
3. **兼容性强**：与 Maya/Qt 事件循环完美集成
4. **代码简洁**：无需管理线程生命周期

这是一个典型的"用对的方式解决问题"的案例 - 不是绕过限制，而是遵循框架的设计理念。

