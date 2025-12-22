# 自定义协议

AuroraView 支持自定义协议，用于加载本地资源而无需担心 CORS 限制。

## 内置协议：`auroraview://`

内置的 `auroraview://` 协议允许从指定的资源根目录加载本地静态资源（HTML、CSS、JS、图片）。

### URL 格式

```
auroraview://css/style.css
auroraview://js/app.js
auroraview://icons/logo.png
```

### 路径映射

```
auroraview://css/style.css → {asset_root}/css/style.css
```

### 使用方法

```python
from auroraview import WebView

webview = WebView.create(
    "我的应用",
    html="""
    <html>
        <head>
            <link rel="stylesheet" href="auroraview://css/style.css">
        </head>
        <body>
            <img src="auroraview://icons/logo.png">
            <script src="auroraview://js/app.js"></script>
        </body>
    </html>
    """,
    asset_root="C:/projects/my_app/assets"  # 资源根目录
)
webview.show()
```

## 自定义协议注册

你可以注册自定义协议用于 DCC 特定的资源加载。

### 使用场景

- Maya: `maya://scenes/character.ma`
- Houdini: `houdini://hip/project.hip`
- Nuke: `nuke://scripts/comp.nk`
- 自定义: `fbx://models/character.fbx`

### Python API

```python
from auroraview import WebView

def handle_fbx_protocol(uri: str) -> dict:
    """
    处理 fbx:// 协议请求
    
    Args:
        uri: 完整 URI，例如 "fbx://models/character.fbx"
    
    Returns:
        {
            "data": bytes,        # 文件内容（bytes）
            "mime_type": str,     # MIME 类型
            "status": int         # HTTP 状态码（200, 404 等）
        }
    """
    # 解析路径
    path = uri.replace("fbx://", "")  # "models/character.fbx"
    
    # 读取 FBX 文件
    fbx_root = "C:/projects/models"
    full_path = f"{fbx_root}/{path}"
    
    try:
        with open(full_path, "rb") as f:
            data = f.read()
        
        return {
            "data": data,
            "mime_type": "application/octet-stream",
            "status": 200
        }
    except FileNotFoundError:
        return {
            "data": b"Not Found",
            "mime_type": "text/plain",
            "status": 404
        }

# 创建 WebView
webview = WebView.create("FBX 查看器", asset_root="C:/assets")

# 注册自定义协议
webview.register_protocol("fbx", handle_fbx_protocol)

# 在 HTML 中使用
webview.load_html("""
<html>
    <body>
        <h1>FBX 查看器</h1>
        <script>
            // 通过 fetch 加载 FBX 文件
            fetch('fbx://models/character.fbx')
                .then(r => r.arrayBuffer())
                .then(data => {
                    console.log('FBX 已加载:', data.byteLength, 'bytes');
                    // 解析 FBX...
                });
        </script>
    </body>
</html>
""")

webview.show()
```

## Maya 插件示例

```python
from auroraview import WebView
import maya.cmds as cmds
import os

def handle_maya_protocol(uri: str) -> dict:
    """处理 maya:// 协议 - 加载 Maya 场景文件缩略图"""
    path = uri.replace("maya://", "")
    
    # Maya 项目目录
    project_dir = cmds.workspace(q=True, rd=True)
    full_path = os.path.join(project_dir, path)
    
    if os.path.exists(full_path):
        with open(full_path, "rb") as f:
            return {
                "data": f.read(),
                "mime_type": "image/jpeg",
                "status": 200
            }
    else:
        return {
            "data": b"Not Found",
            "mime_type": "text/plain",
            "status": 404
        }

# 创建 WebView
webview = WebView.create(
    "Maya 资产浏览器",
    asset_root="C:/maya_plugin/ui",  # UI 资源目录
    parent=maya_hwnd,
    mode="owner"
)

# 注册 maya:// 协议
webview.register_protocol("maya", handle_maya_protocol)

# 加载 UI
webview.load_html("""
<html>
    <head>
        <link rel="stylesheet" href="auroraview://css/style.css">
    </head>
    <body>
        <h1>资产浏览器</h1>
        <div class="thumbnails">
            <img src="maya://thumbnails/character_rig.jpg">
            <img src="maya://thumbnails/environment.jpg">
        </div>
        <script src="auroraview://js/app.js"></script>
    </body>
</html>
""")

webview.show()
```

## 优势

1. **无 CORS 限制** - 自定义协议绕过浏览器 CORS 限制
2. **简洁 API** - Python 函数即可注册协议处理器
3. **灵活** - 可以从文件、内存、数据库等任何来源加载资源
4. **安全** - 每个协议独立控制访问权限
5. **高性能** - 直接文件读取，无 HTTP 服务器开销

## MIME 类型参考

常用资源的 MIME 类型：

| 扩展名 | MIME 类型 |
|--------|-----------|
| `.html` | `text/html` |
| `.css` | `text/css` |
| `.js` | `application/javascript` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.jpg`, `.jpeg` | `image/jpeg` |
| `.gif` | `image/gif` |
| `.svg` | `image/svg+xml` |
| `.woff` | `font/woff` |
| `.woff2` | `font/woff2` |
| `.fbx`, `.obj` | `application/octet-stream` |

## 安全注意事项

实现自定义协议处理器时：

1. **验证所有 URI** - 清理输入以防止注入攻击
2. **清理文件路径** - 防止目录遍历（`../`）
3. **限制访问** - 只允许访问预期的目录
4. **优雅处理错误** - 对无效请求返回正确的状态码
