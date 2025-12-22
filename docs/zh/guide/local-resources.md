# 本地资源加载

本指南比较了在 WebView 应用程序中加载本地资源（图片、CSS、JS、字体）的不同方法。

## 概述

| 方法 | CORS | 复杂度 | 性能 | 安全性 | 推荐 |
|------|------|--------|------|--------|------|
| `file://` | 受限 | 简单 | 好 | 低 | 仅开发 |
| Data URL | 无 | 简单 | 中等 | 高 | 小资源 |
| HTTP 服务器 | 无 | 复杂 | 中等 | 低 | 开发 |
| 自定义协议 | 无 | 中等 | 好 | 高 | **生产** |

## 方法 1：File URL (`file://`)

```python
from auroraview import WebView

webview = WebView(
    title="本地资源",
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

**优点：**
- 简单，无需额外配置
- 所有平台支持
- 直接文件系统访问

**缺点：**
- **CORS 限制** - 无法使用 fetch/XHR 加载本地文件
- **安全限制** - 现代浏览器限制 `file://`
- **路径问题** - 绝对路径在不同环境下不一致
- **跨平台** - Windows (`C:\`) vs Unix (`/home/`)

**CORS 问题示例：**
```javascript
// 这会失败！
fetch('file:///C:/data/config.json')
    .then(r => r.json())
    .catch(e => console.error('CORS 错误:', e));
```

## 方法 2：Data URL（Base64）

```python
import base64

# 读取并编码图片
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

**优点：**
- 无 CORS 限制
- 单文件分发
- 跨平台一致

**缺点：**
- **体积增加 33%** - Base64 编码开销
- **HTML 文件巨大** - 不适合大量资源
- **无缓存** - 每次都重新加载
- **不适合大文件** - 视频、大图片

## 方法 3：本地 HTTP 服务器

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

**优点：**
- **无 CORS 限制** - 自由使用 fetch/XHR
- **完整 HTTP 功能** - 缓存、压缩、范围请求
- **开发体验好** - 类似 Web 开发

**缺点：**
- **需要额外进程** - 管理服务器生命周期
- **端口冲突** - 需要动态分配端口
- **安全风险** - 其他进程可能访问
- **复杂度高** - 处理启动、停止、错误

## 方法 4：自定义协议（推荐）

```python
from auroraview import WebView

webview = WebView(
    title="自定义协议",
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

**优点：**
- **无 CORS 限制** - 自定义协议被视为同源
- **安全** - 只能访问指定目录
- **简洁的 URL** - `asset://logo.png` vs `file:///C:/long/path/logo.png`
- **跨平台** - 路径处理在 Rust 端统一
- **灵活** - 可从内存、数据库、网络加载
- **性能好** - 直接文件读取，无 HTTP 开销

**缺点：**
- 需要在 Rust 后端实现
- 在 WebView 创建时一次性配置

## 实际建议

### 简单应用（少量资源）

**使用**：Data URL
```python
webview = WebView(html=f"""
    <style>{css_content}</style>
    <img src="data:image/png;base64,{logo_base64}">
""")
```

### 开发阶段

**使用**：本地 HTTP 服务器
```python
# Python 内置服务器
# 优点：热重载、调试方便
```

### 生产环境（DCC 集成）

**使用**：自定义协议
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

**原因**：
- 无 CORS 问题 - 自由使用 fetch
- 安全 - 只能访问插件目录
- 简洁 - URL 不暴露文件系统路径
- 灵活 - 可从 Maya 场景、数据库加载

## 实际案例：Maya 插件 UI

### 使用 file://（有问题）

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
            <script>
                // 这会失败！CORS 错误
                fetch('file:///{plugin_dir}/data/config.json')
                    .then(r => r.json())
                    .catch(e => console.error('CORS 错误:', e));
            </script>
        </body>
    </html>
    """
)
```

**问题**：
- CORS 阻止 fetch 加载本地文件
- Windows 路径需要转换
- 暴露文件系统结构

### 使用自定义协议（最佳）

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
            <script>
                // 完美工作！无 CORS 限制
                fetch('maya://data/config.json')
                    .then(r => r.json())
                    .then(data => console.log(data));

                // 可以加载场景资源
                fetch('maya://scenes/current/metadata.json')
                    .then(r => r.json())
                    .then(meta => updateUI(meta));
            </script>
        </body>
    </html>
    """,
    asset_root=plugin_dir
)
```

**优点**：
- 无 CORS 限制
- 简洁的 URL
- 安全（只能访问插件目录）
- 跨平台一致
- 无需额外进程

## 总结

自定义协议是生产环境 DCC 集成的**推荐方法**。它提供：

- 简洁的 URL
- 无 CORS 限制
- 安全、可控的访问
- 出色的开发体验

这是 Tauri、Electron 等现代 WebView 框架使用的相同方法。
