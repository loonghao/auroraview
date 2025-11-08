# 循环引用问题修复

## 🐛 问题描述

在Python中，`EventTimer` 持有对 `WebView` 的引用，而 `MayaOutliner` 同时持有 `EventTimer` 和 `WebView` 的引用，这可能导致循环引用问题：

```
MayaOutliner
    ├─> self.webview (WebView)
    └─> self._event_timer (EventTimer)
            └─> self._webview (WebView)  # 循环引用！
```

### 问题表现

1. **内存泄漏**: 对象无法被垃圾回收
2. **悬垂引用**: EventTimer可能访问已删除的WebView
3. **清理不完整**: 关闭窗口后资源未释放

---

## 🦀 Rust的解决方案

在Rust中，这个问题通过**所有权系统**和**生命周期**自动解决：

```rust
struct EventTimer<'a> {
    webview: &'a WebView,  // 借用，不是所有权
}

// 编译器保证：
// 1. EventTimer不能比WebView活得更久
// 2. WebView被drop时，EventTimer必须已经被drop
// 3. 不会有悬垂引用
```

**Rust优势**:
- ✅ 编译时防止循环引用
- ✅ 编译时防止悬垂引用
- ✅ 自动按正确顺序清理

---

## 🐍 Python的解决方案

在Python中，我们需要**手动管理引用**：

### 修复1: EventTimer.cleanup()

添加 `cleanup()` 方法来清理所有引用：

```python
class EventTimer:
    def cleanup(self) -> None:
        """Cleanup all resources and references.
        
        This method should be called when the EventTimer is no longer needed.
        It stops the timer and clears all references to prevent memory leaks.
        """
        self.stop()
        
        # Clear all callbacks
        self._close_callbacks.clear()
        self._tick_callbacks.clear()
        
        logger.info("EventTimer cleanup complete")
    
    def stop(self) -> None:
        """Stop the timer and cleanup resources."""
        if not self._running:
            return

        self._running = False
        self._stop_timer_impl()
        
        # Clear webview reference to prevent circular references
        self._webview = None
        
        logger.info("EventTimer stopped and cleaned up")
```

### 修复2: MayaOutliner使用cleanup()

更新 `MayaOutliner` 使用新的 `cleanup()` 方法：

```python
class MayaOutliner:
    def _stop_event_processing(self):
        """Stop automatic event processing (EventTimer) and cleanup resources.
        
        Uses EventTimer.cleanup() to properly clear all references including
        the webview reference to prevent circular references.
        """
        if self._event_timer is not None:
            try:
                # Use cleanup() instead of stop() to clear webview reference
                self._event_timer.cleanup()
                print("[MayaOutliner] EventTimer stopped and cleaned up")
                self._event_timer = None
            except Exception as e:
                print(f"[MayaOutliner] Failed to stop EventTimer: {e}")
```

---

## 📊 对比总结

| 方面 | Python (修复前) | Python (修复后) | Rust |
|------|----------------|----------------|------|
| 循环引用 | ❌ 可能发生 | ✅ 手动清理 | ✅ 编译时防止 |
| 悬垂引用 | ❌ 可能发生 | ✅ 手动清理 | ✅ 编译时防止 |
| 内存泄漏 | ❌ 可能发生 | ✅ 手动清理 | ✅ 自动防止 |
| 清理顺序 | ❌ 不确定 | ✅ 手动管理 | ✅ 编译时确定 |
| 开发负担 | ❌ 容易忘记 | ⚠️ 需要记住调用cleanup() | ✅ 编译器强制 |

---

## 🎯 关键教训

### 1. **Python需要手动管理引用**

```python
# 不好的做法
def close(self):
    self._event_timer.stop()  # 只停止，不清理引用

# 好的做法
def close(self):
    self._event_timer.cleanup()  # 停止并清理所有引用
```

### 2. **Rust自动管理引用**

```rust
// Rust - 编译器自动管理
impl Drop for EventTimer {
    fn drop(&mut self) {
        // 所有字段自动按声明逆序drop
        // 不需要手动清理引用
    }
}
```

### 3. **使用弱引用（可选方案）**

Python也可以使用 `weakref` 来避免循环引用：

```python
import weakref

class EventTimer:
    def __init__(self, webview):
        self._webview = weakref.ref(webview)  # 弱引用
    
    def _tick(self):
        webview = self._webview()  # 解引用
        if webview is None:
            # WebView已被删除
            self.stop()
            return
        # 使用webview...
```

**但是**：弱引用增加了复杂性，需要每次检查是否为None。

---

## ✅ 修复验证

### 测试步骤

1. **创建Maya Outliner**:
   ```python
   from maya_integration import maya_outliner
   outliner = maya_outliner.main()
   ```

2. **关闭窗口**:
   ```python
   outliner.close()
   ```

3. **检查清理**:
   ```python
   # 应该看到：
   # [MayaOutliner] Step 1: Stopping EventTimer...
   # EventTimer stopped and cleaned up
   # [MayaOutliner] EventTimer stopped and cleaned up
   # [MayaOutliner] Step 2: Removing Maya callbacks...
   # [MayaOutliner] Step 3: Closing WebView window...
   # [MayaOutliner] Step 4: Cleared WebView reference
   # [MayaOutliner] Step 5: Removed from singleton registry
   ```

4. **重新创建**:
   ```python
   outliner2 = maya_outliner.main()
   # 应该成功创建新实例
   ```

---

## 📚 相关文档

- `docs/RUST_SOLVES_DCC_WINDOW_PROBLEMS.md` - Rust如何解决这些问题
- `docs/SINGLETON_MODE.md` - 单例模式实现
- `python/auroraview/event_timer.py` - EventTimer实现

---

**总结**: 在Python中需要手动管理引用来防止循环引用，而Rust通过所有权系统在编译时自动解决这个问题。这是Rust在系统编程中的一个重要优势。


