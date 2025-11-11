# 🚀 Simplified API - No More Bridge Waiting!

## 问题解决

### ❌ 旧方式的问题

1. **随机延迟**: 需要等待 `window.auroraview` 初始化
2. **复杂代码**: 需要手动编写等待逻辑
3. **外部依赖**: 需要加载 `auroraview-bridge.js`
4. **学习成本**: 需要理解桥接机制

```javascript
// ❌ 旧方式 - 需要等待
let retryCount = 0;
function waitForBridge() {
    retryCount++;
    if (window.auroraview && window.auroraview.send_event) {
        // 终于可以用了!
        window.auroraview.send_event('test', {});
    } else {
        setTimeout(waitForBridge, 100); // 继续等待...
    }
}
waitForBridge();
```

### ✅ 新方式的优势

1. **零延迟**: 桥接在初始化脚本中立即可用
2. **简单API**: 直接使用 `window.aurora`
3. **无需等待**: 不需要任何等待代码
4. **Qt风格**: 熟悉的 `emit()` / `on()` 语法

```javascript
// ✅ 新方式 - 立即可用!
window.aurora.emit('test', {});  // 直接使用,无需等待!
```

## 三种API层级

### 1. 高级API (推荐) - `window.aurora`

**最简单的方式,开箱即用!**

```javascript
// 发送事件到Python (JavaScript → Python)
window.aurora.emit('create_node', { type: 'Grade' });

// 接收Python事件 (Python → JavaScript)
window.aurora.on('node_created', (data) => {
    console.log('Node:', data.name);
});
```

**特点:**
- ✓ 立即可用,无需等待
- ✓ Qt风格API
- ✓ 链式调用支持
- ✓ 自动错误处理

### 2. 类API - `new AuroraView()`

**适合需要多个实例的场景**

```javascript
const bridge = new AuroraView();

bridge
    .on('event1', handler1)
    .on('event2', handler2)
    .emit('ready', {});
```

### 3. 低级API - `window.auroraview`

**底层API,通常不需要直接使用**

```javascript
window.auroraview.send_event('test', {});
window.auroraview.on('response', callback);
```

## 完整示例

### Python端

```python
from auroraview import WebView

webview = WebView.create(
    title="My Tool",
    width=800,
    height=600
)

@webview.on("create_node")
def handle_create_node(data):
    node_type = data.get("type", "Grade")
    # ... 创建节点 ...
    webview.emit("node_created", {"name": "Grade1"})

html = """
<!DOCTYPE html>
<html>
<body>
    <button onclick="createNode()">Create Node</button>
    <script>
        // ✓ 立即可用,无需等待!
        window.aurora.on('node_created', (data) => {
            console.log('Created:', data.name);
        });
        
        function createNode() {
            window.aurora.emit('create_node', { type: 'Grade' });
        }
    </script>
</body>
</html>
"""

webview.load_html(html)
webview.show()
```

### JavaScript端

```javascript
// 1. 注册事件监听器
window.aurora.on('node_created', (data) => {
    console.log('✓ Node created:', data.name);
});

window.aurora.on('error', (data) => {
    console.error('✗ Error:', data.message);
});

// 2. 发送事件到Python
function createGrade() {
    window.aurora.emit('create_node', { type: 'Grade' });
}

function createBlur() {
    window.aurora.emit('create_node', { type: 'Blur' });
}
```

## 迁移指南

### 从旧API迁移

```javascript
// ❌ 旧方式
function waitForBridge() {
    if (window.auroraview) {
        window.auroraview.send_event('test', {});
        window.auroraview.on('response', handler);
    } else {
        setTimeout(waitForBridge, 100);
    }
}
waitForBridge();

// ✅ 新方式
window.aurora.emit('test', {});
window.aurora.on('response', handler);
```

### 从 auroraview-bridge.js 迁移

```javascript
// ❌ 旧方式 - 需要加载外部文件
const bridge = new AuroraViewBridge();
bridge.emit('test', {});
bridge.connect('response', handler);

// ✅ 新方式 - 内置,无需加载
window.aurora.emit('test', {});
window.aurora.on('response', handler);
```

## 运行示例

```bash
# 简化版示例 (推荐)
python examples/nuke_examples/test_simplified.py

# 在Nuke中
import sys
sys.path.insert(0, r'C:\path\to\dcc_webview\examples')
from nuke_examples import test_simplified
test_simplified.run()
```

## 技术细节

### 初始化时机

桥接在 `with_initialization_script()` 中注入,在页面加载时立即执行:

```rust
// src/webview/standalone.rs
let event_bridge_script = r#"
(function() {
    // 创建 window.auroraview (低级API)
    window.auroraview = { ... };
    
    // 创建 window.AuroraView 类
    window.AuroraView = class { ... };
    
    // 创建默认实例
    window.aurora = new window.AuroraView();
})();
"#;
```

### 为什么没有延迟?

1. **初始化脚本**: 在页面加载前执行
2. **同步创建**: 对象立即可用
3. **无异步**: 不需要等待任何异步操作

## 常见问题

**Q: 还需要 `auroraview-bridge.js` 吗?**  
A: 不需要!所有功能已内置。

**Q: 旧代码还能用吗?**  
A: 可以!`window.auroraview` 仍然可用,但推荐使用 `window.aurora`。

**Q: 如何检查桥接是否ready?**  
A: 不需要检查!`window.aurora` 在脚本执行时就已经ready。

**Q: 支持哪些DCC?**  
A: Nuke, Maya, Houdini, Blender等所有支持的DCC。

## 总结

✅ **使用 `window.aurora` - 简单、快速、可靠!**

- 无需等待
- 无需外部文件
- Qt风格API
- 立即可用

🎉 **享受零延迟的开发体验!**

