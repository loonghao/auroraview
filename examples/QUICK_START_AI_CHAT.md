# Quick Start: AI Chat Integration in DCC

## 问题解决方案

### 问题 1: WebView 阻塞 DCC 软件 ❌

**错误的做法**:
```python
webview.show()  # 这会阻塞!
```

**正确的做法** ✅:
```python
webview.show()  # 当前实现已经是非阻塞的(在后台线程运行)
```

AuroraView 的 `show()` 方法已经在后台线程运行,不会阻塞 DCC 主线程。

---

### 问题 2: JavaScript 注入没有生效 ❌

**错误的做法**:
```python
webview.load_url("https://example.com")
time.sleep(2)
webview.eval_js(script)  # 可能页面还没加载完
webview.show()
```

**正确的做法** ✅:
```python
# 方法 1: 使用延迟注入
def inject_after_load(webview, script, delay=3.0):
    def _inject():
        time.sleep(delay)
        webview.eval_js(script)
    
    import threading
    threading.Thread(target=_inject, daemon=True).start()

webview.load_url("https://example.com")
webview.show()
inject_after_load(webview, script, delay=3.0)
```

---

## 完整示例代码

### 在 DCC 中使用 (Maya, Houdini, etc.)

```python
import time
import threading
from auroraview import WebView

# JavaScript 注入脚本
INJECTION_SCRIPT = """
(function() {
    console.log('[DCC] Injection starting...');
    
    // 添加自定义按钮
    const btn = document.createElement('button');
    btn.textContent = '🎨 Get DCC Selection';
    btn.style.cssText = `
        position: fixed;
        top: 10px;
        right: 10px;
        z-index: 999999;
        padding: 10px 20px;
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        color: white;
        border: none;
        border-radius: 6px;
        cursor: pointer;
        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    `;
    btn.onclick = () => {
        window.dispatchEvent(new CustomEvent('get_scene_info'));
    };
    document.body.appendChild(btn);
    
    // 监听来自 DCC 的响应
    window.addEventListener('scene_info_response', (e) => {
        console.log('DCC Selection:', e.detail);
        alert(JSON.stringify(e.detail, null, 2));
    });
    
    console.log('[DCC] Injection complete');
})();
"""

# 延迟注入函数
def inject_script_delayed(webview, script, delay=3.0):
    """在延迟后注入 JavaScript"""
    def _inject():
        time.sleep(delay)
        print(f"[INFO] Injecting JavaScript after {delay}s delay...")
        try:
            webview.eval_js(script)
            print("[OK] JavaScript injected successfully")
        except Exception as e:
            print(f"[ERROR] Failed to inject: {e}")
    
    thread = threading.Thread(target=_inject, daemon=True)
    thread.start()

# 创建 WebView
webview = WebView(
    title="AI Chat Integration",
    width=1200,
    height=800,
    dev_tools=True  # 开启 DevTools 以便调试
)

# 注册事件处理器 (在 show() 之前)
@webview.on("get_scene_info")
def handle_get_scene_info(data):
    print("[RECV] Website requested scene info")
    
    # 获取 DCC 场景数据
    # 在 Maya 中: selection = cmds.ls(selection=True)
    # 在 Houdini 中: selection = hou.selectedNodes()
    selection = ["pCube1", "pSphere1", "camera1"]
    
    # 发送回网页
    webview.emit("scene_info_response", {
        "selection": selection,
        "count": len(selection)
    })
    print(f"[SEND] Sent {len(selection)} objects")

# 加载 AI 聊天网站
webview.load_url("https://knot.woa.com/chat?web_key=1c2a6b4568f24e00a58999c1b7cb0f6e")

# 显示 WebView (非阻塞)
webview.show()
print("[OK] WebView opened (non-blocking)")

# 延迟注入 JavaScript (等待页面加载)
inject_script_delayed(webview, INJECTION_SCRIPT, delay=3.0)

# 重要: 保持引用以防止被垃圾回收
# 在 Maya 中: __main__.ai_chat = webview
# 在 Houdini 中: hou.session.ai_chat = webview

print("[OK] Setup complete! DCC should remain responsive.")
```

---

## 关键要点

### 1. 非阻塞显示 ✅
```python
webview.show()  # 已经是非阻塞的,在后台线程运行
```

### 2. 延迟注入 JavaScript ✅
```python
# 不要立即注入
webview.load_url("https://example.com")
time.sleep(2)  # ❌ 不够可靠
webview.eval_js(script)

# 使用延迟注入
webview.load_url("https://example.com")
webview.show()
inject_script_delayed(webview, script, delay=3.0)  # ✅ 可靠
```

### 3. 事件处理器在 show() 之前注册 ✅
```python
# 正确顺序
@webview.on("event_name")
def handler(data):
    pass

webview.show()  # ✅
```

### 4. 保持引用 ✅
```python
# 在 DCC 中,保存到全局变量
__main__.webview = webview  # Maya
hou.session.webview = webview  # Houdini
```

---

## 调试技巧

### 1. 开启 DevTools
```python
webview = WebView(dev_tools=True)
```
按 F12 打开开发者工具,查看:
- Console 日志
- Network 请求
- Elements 结构

### 2. 检查注入是否成功
在 DevTools Console 中输入:
```javascript
window.DCCIntegration  // 应该显示对象
```

### 3. 手动测试事件
在 DevTools Console 中:
```javascript
// 触发事件到 Python
window.dispatchEvent(new CustomEvent('get_scene_info'));

// 检查是否收到响应
window.addEventListener('scene_info_response', (e) => {
    console.log('Received:', e.detail);
});
```

---

## 常见问题

### Q1: 为什么注入的按钮没有出现?
**A**: 页面可能还没加载完。增加延迟时间:
```python
inject_script_delayed(webview, script, delay=5.0)  # 增加到 5 秒
```

### Q2: 为什么 DCC 还是卡住了?
**A**: 确保使用的是 `show()` 而不是 `show_blocking()`:
```python
webview.show()  # ✅ 非阻塞
# webview.show_blocking()  # ❌ 阻塞
```

### Q3: 如何知道页面加载完成?
**A**: 在注入脚本中检查:
```javascript
if (document.readyState === 'complete') {
    console.log('Page fully loaded');
} else {
    window.addEventListener('load', () => {
        console.log('Page loaded');
    });
}
```

### Q4: 如何在 Maya 中使用?
**A**: 
```python
# 在 Maya Script Editor 中
exec(open('path/to/example.py').read())

# 或者直接粘贴代码
# ... (上面的完整示例代码)

# 保存引用
import __main__
__main__.ai_chat_webview = webview
```

---

## 完整工作流程

1. **创建 WebView**
   ```python
   webview = WebView(title="AI Chat", dev_tools=True)
   ```

2. **注册事件处理器**
   ```python
   @webview.on("event_name")
   def handler(data):
       pass
   ```

3. **加载网页**
   ```python
   webview.load_url("https://example.com")
   ```

4. **显示 WebView**
   ```python
   webview.show()  # 非阻塞
   ```

5. **延迟注入 JavaScript**
   ```python
   inject_script_delayed(webview, script, delay=3.0)
   ```

6. **保持引用**
   ```python
   __main__.webview = webview
   ```

---

## 参考示例

- `examples/07_ai_chat_non_blocking.py` - 完整的非阻塞示例
- `examples/05_third_party_site_injection.py` - JavaScript 注入基础
- `examples/06_ai_chat_integration.py` - AI 聊天集成模式

---

## 下一步

1. 运行示例: `python examples/07_ai_chat_non_blocking.py`
2. 在 DCC 中测试
3. 根据实际网站调整选择器和延迟时间
4. 添加更多自定义功能

祝你成功! 🎉

