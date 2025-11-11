# Nuke 退出问题修复

## 问题描述

### 症状

1. **Nuke 退出后 WebView 窗口仍然存在**
   - 用户关闭 Nuke，但 QtWebView 窗口没有关闭
   - 窗口变成"孤儿窗口"，无法正常交互

2. **JavaScript 错误**
   ```
   Uncaught TypeError: Cannot read property 'on' of undefined
   ```
   - Nuke 退出后，`window.auroraview` 对象失效
   - JavaScript 代码尝试访问未定义的属性

3. **Qt 事件循环错误**
   ```
   QEventDispatcherWin32::wakeUp: Failed to post a message (无效的窗口句柄。)
   ```
   - Nuke 的事件循环已停止
   - WebView 仍在尝试发送消息到已失效的窗口句柄

## 根本原因

### 1. 窗口生命周期管理问题

```python
# 问题代码
webview = QtWebView(parent=None)  # 独立窗口
```

当 `parent=None` 时，QtWebView 是一个独立的顶级窗口：
- 不会随父应用（Nuke）退出而自动关闭
- Qt 不知道这个窗口应该在应用退出时关闭
- 窗口成为"孤儿窗口"

### 2. 缺少应用退出监听

- 没有监听 `QCoreApplication.aboutToQuit` 信号
- 应用退出时没有触发窗口清理
- WebView 继续运行，但事件循环已停止

### 3. JavaScript 对象未清理

- `window.auroraview` 对象在窗口关闭时仍然存在
- JavaScript 代码尝试访问已失效的 Qt 对象
- 导致 "Cannot read property of undefined" 错误

## 解决方案

### 1. 设置 WA_DeleteOnClose 属性

```python
def __init__(self, parent=None, ...):
    super().__init__(parent)
    
    # 设置窗口在关闭时自动删除
    from qtpy.QtCore import Qt
    self.setAttribute(Qt.WA_DeleteOnClose, True)
```

**作用**：
- 窗口关闭时自动调用 `deleteLater()`
- 确保 Qt 对象被正确清理
- 防止内存泄漏

### 2. 注册应用退出处理器

```python
def _register_app_quit_handler(self):
    """注册应用退出处理器"""
    try:
        from qtpy.QtCore import QCoreApplication
        
        app = QCoreApplication.instance()
        if app:
            # 连接到 aboutToQuit 信号
            app.aboutToQuit.connect(self._on_app_quit)
            logger.debug("Registered application quit handler")
    except Exception as e:
        logger.warning(f"Could not register app quit handler: {e}")

def _on_app_quit(self):
    """处理应用退出事件"""
    logger.info("Application quitting - closing QtWebView")
    try:
        # 立即关闭窗口
        if not self._is_closing:
            self.close()
    except Exception as e:
        logger.error(f"Error closing QtWebView on app quit: {e}")
```

**作用**：
- 监听应用退出信号
- 在 Nuke 退出时自动关闭 WebView
- 防止孤儿窗口

### 3. 清理 JavaScript 对象（改进版）

```python
def closeEvent(self, event):
    # Step 1: 先销毁 JavaScript 桥接
    try:
        cleanup_js = """
        (function() {
            if (window.auroraview && window.auroraview._destroy) {
                window.auroraview._destroy();  // 调用清理方法
            }
            window.auroraview = null;  // 清空对象
        })();
        """
        self.page().runJavaScript(cleanup_js)
    except Exception as e:
        logger.debug(f"Could not destroy JavaScript bridge: {e}")

    # Step 2: 清理 Python 桥接
    if hasattr(self, "_bridge") and self._bridge:
        self._bridge.cleanup()

    # ... 继续其他清理
```

**作用**：
- **先**销毁 JavaScript 桥接，再清理 Python 对象
- 调用 `_destroy()` 方法标记桥接为已销毁
- 防止 JavaScript 在清理过程中访问已失效的对象
- 避免 "Cannot read property of undefined" 错误

### 4. JavaScript 错误处理（改进版）

```javascript
(function() {
    // 防止重复初始化
    if (window.auroraview) {
        console.log('[AuroraView] Bridge already initialized');
        return;
    }

    // 检查 QWebChannel 是否可用
    if (typeof QWebChannel === 'undefined') {
        console.error('[AuroraView] QWebChannel not available');
        return;
    }

    // 检查 qt.webChannelTransport 是否可用
    if (typeof qt === 'undefined' || !qt.webChannelTransport) {
        console.error('[AuroraView] qt.webChannelTransport not available');
        return;
    }

    try {
        new QWebChannel(qt.webChannelTransport, function(channel) {
            // 检查 bridge 对象是否存在
            if (!channel.objects || !channel.objects.auroraview_bridge) {
                console.error('[AuroraView] Bridge object not found');
                return;
            }

            var bridge = channel.objects.auroraview_bridge;
            var isDestroyed = false;

            // 监听窗口卸载事件
            window.addEventListener('beforeunload', function() {
                isDestroyed = true;
                console.log('[AuroraView] Window unloading - marking bridge as destroyed');
            });

            window.auroraview = {
                send_event: function(eventName, data) {
                    if (isDestroyed) {
                        console.warn('[AuroraView] Bridge destroyed - ignoring send_event');
                        return;
                    }
                    try {
                        if (!bridge || !bridge.js_to_python) {
                            console.error('[AuroraView] Bridge not available');
                            return;
                        }
                        var jsonData = JSON.stringify(data || {});
                        bridge.js_to_python(eventName, jsonData);
                    } catch (e) {
                        console.error('[AuroraView] Error sending event:', e);
                    }
                },
                on: function(eventName, callback) {
                    if (isDestroyed) {
                        console.warn('[AuroraView] Bridge destroyed - ignoring on');
                        return;
                    }
                    try {
                        if (!bridge || !bridge.python_to_js) {
                            console.error('[AuroraView] Bridge not available');
                            return;
                        }
                        bridge.python_to_js.connect(function(name, jsonData) {
                            if (isDestroyed) return;
                            if (name === eventName) {
                                try {
                                    var data = JSON.parse(jsonData);
                                    callback(data);
                                } catch (e) {
                                    console.error('[AuroraView] Error parsing event data:', e);
                                }
                            }
                        });
                    } catch (e) {
                        console.error('[AuroraView] Error registering handler:', e);
                    }
                },
                _destroy: function() {
                    isDestroyed = true;
                    console.log('[AuroraView] Bridge destroyed');
                }
            };
        });
    } catch (e) {
        console.error('[AuroraView] Error initializing QWebChannel:', e);
    }
})();
```

**关键改进**：
1. **防止重复初始化**：检查 `window.auroraview` 是否已存在
2. **销毁状态跟踪**：使用 `isDestroyed` 标志
3. **beforeunload 监听**：窗口卸载时自动标记为已销毁
4. **_destroy() 方法**：提供显式销毁方法供 Python 调用
5. **状态检查**：所有方法都检查 `isDestroyed` 状态
6. **更安全的对象访问**：检查 `bridge` 和方法是否存在

## 测试验证

### 在 Nuke 中测试

```python
# 在 Nuke Script Editor 中运行
import sys
from pathlib import Path

examples_dir = Path(r'C:\\path\\to\\dcc_webview\\examples')
sys.path.insert(0, str(examples_dir))

import nuke_examples.test_qt_lifecycle as example
example.run()
```

### 验证清单

- [ ] 创建节点后 UI 立即更新（无延迟）
- [ ] 关闭窗口无错误
- [ ] 退出 Nuke 时窗口自动关闭
- [ ] 无 "RuntimeError: Internal C++ object already deleted"
- [ ] 无 "Cannot read property 'on' of undefined"
- [ ] 无 "QEventDispatcherWin32::wakeUp: Failed to post a message"

## 相关文件

- `python/auroraview/qt_integration.py` - 主要修复
- `examples/nuke_examples/test_qt_lifecycle.py` - 测试示例
- `tests/test_qt_lifecycle.py` - 单元测试
- `docs/QT_LIFECYCLE_FIX.md` - 完整文档

