# 本地资源加载方案对比

## 📋 问题

在 WebView 中加载本地资源（图片、CSS、JS、字体等）有多种方式，每种都有优缺点。

---

## 🎯 方案对比

### 方案 1: File URL（`file://`）

**使用方式**:
```python
from auroraview import WebView

webview = WebView(
    title="Local Resources",
    html="""
    <html>
        <head>
            <link rel="stylesheet" href="file:///C:/projects/my_app/style.css">
        </head>
        <body>
            <img src="file:///C:/projects/my_app/logo.png">
            <script src="file:///C:/projects/my_app/app.js"></script>
        </body>
    </html>
    """
)
```

**优点**:
- ✅ 简单直接，不需要额外配置
- ✅ 所有平台都支持
- ✅ 可以直接访问文件系统

**缺点**:
- ❌ **CORS 限制** - 无法使用 fetch/XHR 加载其他本地文件
- ❌ **安全限制** - 现代浏览器限制 file:// 的功能
- ❌ **路径问题** - 绝对路径在不同环境下不一致
- ❌ **跨平台** - Windows (`C:\`) vs Unix (`/home/`)

**CORS 问题示例**:
```javascript
// ❌ 这会失败！
fetch('file:///C:/data/config.json')
    .then(r => r.json())
    .catch(e => console.error('CORS error:', e));
```

---

### 方案 2: Data URL（Base64 编码）

**使用方式**:
```python
import base64

# 读取图片并编码
with open('logo.png', 'rb') as f:
    logo_data = base64.b64encode(f.read()).decode()

webview = WebView(
    html=f"""
    <html>
        <body>
            <img src="data:image/png;base64,{logo_data}">
        </body>
    </html>
    """
)
```

**优点**:
- ✅ 无 CORS 限制
- ✅ 资源嵌入 HTML，单文件分发
- ✅ 跨平台一致

**缺点**:
- ❌ **体积增加 33%** - Base64 编码开销
- ❌ **HTML 文件巨大** - 大量资源时不可行
- ❌ **无缓存** - 每次都重新加载
- ❌ **不适合大文件** - 视频、大图片等

---

### 方案 3: 本地 HTTP 服务器

**使用方式**:
```python
from auroraview import WebView
import http.server
import threading
import os

# 启动本地服务器
def start_server():
    os.chdir('/path/to/resources')
    server = http.server.HTTPServer(('localhost', 8080), 
                                     http.server.SimpleHTTPRequestHandler)
    server.serve_forever()

threading.Thread(target=start_server, daemon=True).start()

# 使用 http:// 加载资源
webview = WebView(
    html="""
    <html>
        <head>
            <link rel="stylesheet" href="http://localhost:8080/style.css">
        </head>
        <body>
            <img src="http://localhost:8080/logo.png">
            <script src="http://localhost:8080/app.js"></script>
        </body>
    </html>
    """
)
```

**优点**:
- ✅ **无 CORS 限制** - 可以自由使用 fetch/XHR
- ✅ **完整的 HTTP 功能** - 缓存、压缩、范围请求
- ✅ **开发体验好** - 类似 Web 开发

**缺点**:
- ❌ **需要额外进程** - 管理服务器生命周期
- ❌ **端口冲突** - 需要动态分配端口
- ❌ **安全风险** - 其他进程可能访问
- ❌ **复杂度高** - 需要处理启动、停止、错误

---

### 方案 4: Custom Protocol（`asset://`, `dcc://`）⭐ 推荐

**使用方式**:
```python
from auroraview import WebView

webview = WebView(
    title="Custom Protocol",
    html="""
    <html>
        <head>
            <link rel="stylesheet" href="asset://style.css">
        </head>
        <body>
            <img src="asset://images/logo.png">
            <script src="asset://js/app.js"></script>
        </body>
    </html>
    """,
    # 配置资源根目录
    asset_root="/path/to/resources"
)
```

**Rust 实现**:
```rust
// 在 backend/native.rs 中
webview_builder.with_custom_protocol("asset".into(), move |_id, request| {
    let path = request.uri().path().trim_start_matches('/');
    let full_path = asset_root.join(path);
    
    match std::fs::read(&full_path) {
        Ok(data) => {
            let mime = mime_guess::from_path(&full_path)
                .first_or_octet_stream();
            
            http::Response::builder()
                .header("Content-Type", mime.as_ref())
                .body(data.into())
                .unwrap()
        }
        Err(_) => {
            http::Response::builder()
                .status(404)
                .body(b"Not Found".to_vec().into())
                .unwrap()
        }
    }
})
```

**优点**:
- ✅ **无 CORS 限制** - 自定义协议被视为同源
- ✅ **安全** - 只能访问指定目录
- ✅ **简洁的 URL** - `asset://logo.png` vs `file:///C:/long/path/logo.png`
- ✅ **跨平台** - 路径处理在 Rust 端统一
- ✅ **灵活** - 可以从内存、数据库、网络加载
- ✅ **性能好** - 直接文件读取，无 HTTP 开销

**缺点**:
- ⚠️ **需要实现** - 当前 protocol.rs 未集成
- ⚠️ **一次性配置** - 在 WebView 创建时注册

---

## 📊 方案对比表

| 特性 | file:// | Data URL | HTTP Server | Custom Protocol |
|------|---------|----------|-------------|-----------------|
| **CORS 限制** | ❌ 有 | ✅ 无 | ✅ 无 | ✅ 无 |
| **实现复杂度** | ✅ 简单 | ✅ 简单 | ❌ 复杂 | ⚠️ 中等 |
| **性能** | ✅ 好 | ⚠️ 中等 | ⚠️ 中等 | ✅ 好 |
| **安全性** | ⚠️ 低 | ✅ 高 | ❌ 低 | ✅ 高 |
| **大文件支持** | ✅ 好 | ❌ 差 | ✅ 好 | ✅ 好 |
| **跨平台** | ⚠️ 路径问题 | ✅ 好 | ✅ 好 | ✅ 好 |
| **开发体验** | ⚠️ 中等 | ❌ 差 | ✅ 好 | ✅ 好 |

---

## 🎯 实际使用建议

### 场景 1: 简单应用（少量资源）

**推荐**: Data URL
```python
# 适合：几个小图标、小 CSS 文件
webview = WebView(html=f"""
    <style>{css_content}</style>
    <img src="data:image/png;base64,{logo_base64}">
""")
```

---

### 场景 2: 开发阶段

**推荐**: 本地 HTTP 服务器
```python
# 使用 Python 内置服务器
# 优点：热重载、调试方便
```

---

### 场景 3: 生产环境（DCC 集成）⭐

**推荐**: Custom Protocol
```python
# Maya/Houdini 插件
webview = WebView(
    html="""
    <link rel="stylesheet" href="dcc://ui/style.css">
    <img src="dcc://icons/tool.png">
    """,
    asset_root=os.path.join(os.path.dirname(__file__), "resources")
)
```

**为什么**:
- ✅ 无 CORS 问题 - 可以自由使用 fetch
- ✅ 安全 - 只能访问插件目录
- ✅ 简洁 - URL 不暴露文件系统路径
- ✅ 灵活 - 可以从 Maya 场景、数据库加载

---

## 🚀 结论

**如果不实现 Custom Protocol**:

1. **开发阶段**: 使用本地 HTTP 服务器
   ```python
   # 简单但需要管理服务器
   ```

2. **生产环境**: 
   - **小应用**: Data URL（嵌入资源）
   - **大应用**: 本地 HTTP 服务器（复杂但可行）

**如果实现 Custom Protocol** ⭐:
- 所有场景都使用 `asset://` 协议
- 最佳的开发和生产体验
- 这就是 Tauri、Electron 等框架的做法

---

## 💡 我的建议

**保留并实现 `protocol.rs`**，因为：

1. ✅ **解决 CORS 问题** - 这是 file:// 的最大痛点
2. ✅ **简化部署** - 不需要 HTTP 服务器
3. ✅ **提升安全** - 限制访问范围
4. ✅ **行业标准** - Tauri、Electron 都这样做
5. ✅ **代码已完成** - 只需集成到 Wry

**实现优先级**: 高 🔥
**实现难度**: 中等（约 50 行代码集成）

---

## 📝 实际案例对比

### 案例: Maya 插件 UI

**需求**: 加载插件的 HTML、CSS、JS、图片资源

#### ❌ 使用 file:// （有问题）

```python
import os

plugin_dir = os.path.dirname(__file__)

webview = WebView(
    html=f"""
    <html>
        <head>
            <link rel="stylesheet" href="file:///{plugin_dir}/ui/style.css">
        </head>
        <body>
            <img src="file:///{plugin_dir}/icons/logo.png">
            <script src="file:///{plugin_dir}/ui/app.js"></script>
            <script>
                // ❌ 这会失败！CORS 错误
                fetch('file:///{plugin_dir}/data/config.json')
                    .then(r => r.json())
                    .catch(e => console.error('CORS error:', e));
            </script>
        </body>
    </html>
    """
)
```

**问题**:
- ❌ CORS 阻止 fetch 加载本地文件
- ❌ Windows 路径 `C:\` 需要转换
- ❌ 路径暴露文件系统结构

---

#### ⚠️ 使用 HTTP 服务器（可行但复杂）

```python
import http.server
import threading
import os
from auroraview import WebView, find_free_port

# 启动本地服务器
plugin_dir = os.path.dirname(__file__)
port = find_free_port()

def start_server():
    os.chdir(plugin_dir)
    server = http.server.HTTPServer(
        ('localhost', port),
        http.server.SimpleHTTPRequestHandler
    )
    server.serve_forever()

server_thread = threading.Thread(target=start_server, daemon=True)
server_thread.start()

webview = WebView(
    html=f"""
    <html>
        <head>
            <link rel="stylesheet" href="http://localhost:{port}/ui/style.css">
        </head>
        <body>
            <img src="http://localhost:{port}/icons/logo.png">
            <script src="http://localhost:{port}/ui/app.js"></script>
            <script>
                // ✅ 这可以工作
                fetch('http://localhost:{port}/data/config.json')
                    .then(r => r.json())
                    .then(data => console.log(data));
            </script>
        </body>
    </html>
    """
)
```

**问题**:
- ⚠️ 需要管理服务器线程
- ⚠️ 需要动态分配端口
- ⚠️ 需要处理服务器启动失败
- ⚠️ 其他进程可能访问（安全风险）

---

#### ✅ 使用 Custom Protocol（最佳方案）

```python
from auroraview import WebView
import os

plugin_dir = os.path.dirname(__file__)

webview = WebView(
    html="""
    <html>
        <head>
            <link rel="stylesheet" href="maya://ui/style.css">
        </head>
        <body>
            <img src="maya://icons/logo.png">
            <script src="maya://ui/app.js"></script>
            <script>
                // ✅ 完美工作！无 CORS 限制
                fetch('maya://data/config.json')
                    .then(r => r.json())
                    .then(data => console.log(data));

                // ✅ 可以加载场景资源
                fetch('maya://scenes/current/metadata.json')
                    .then(r => r.json())
                    .then(meta => updateUI(meta));
            </script>
        </body>
    </html>
    """,
    # 配置资源根目录
    asset_root=plugin_dir
)
```

**优点**:
- ✅ 无 CORS 限制
- ✅ 简洁的 URL
- ✅ 安全（只能访问插件目录）
- ✅ 跨平台一致
- ✅ 无需额外进程

---

## 🎯 最终建议

**强烈建议保留并实现 `protocol.rs`**！

这是现代 WebView 应用的标准做法，也是 AuroraView 作为 DCC 集成工具的核心功能之一。

没有它，用户将被迫使用：
- file:// + CORS 限制 😞
- HTTP 服务器 + 复杂管理 😫
- Data URL + 体积膨胀 😢

有了它，用户可以：
- 简洁的 URL ✨
- 无 CORS 限制 🚀
- 安全可控 🔒
- 开发体验好 💯

