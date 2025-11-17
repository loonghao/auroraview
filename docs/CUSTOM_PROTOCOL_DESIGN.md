# 自定义协议系统设计

## 🎯 设计目标

1. **内置 `auroraview://` 协议** - 解决通用资源跨域问题
2. **支持自定义协议注册** - 允许 DCC 应用注册自己的协议处理器
3. **Python API** - 简洁易用的接口

---

## 📐 架构设计

### 1. 内置协议：`auroraview://`

**用途**: 加载本地静态资源（HTML、CSS、JS、图片等）

**URL 格式**:
```
auroraview://css/style.css
auroraview://js/app.js
auroraview://icons/logo.png
```

**路径映射**:
```
auroraview://css/style.css → {asset_root}/css/style.css
```

**Python API**:
```python
from auroraview import WebView

webview = WebView.create(
    "My App",
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

---

### 2. 自定义协议注册

**用途**: DCC 应用注册自己的协议处理器

**使用场景**:
- Maya: `maya://scenes/character.ma`
- Houdini: `houdini://hip/project.hip`
- Nuke: `nuke://scripts/comp.nk`
- 自定义: `fbx://models/character.fbx`

**Python API**:
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
            "status": int         # HTTP 状态码（200, 404, 等）
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
webview = WebView.create("FBX Viewer", asset_root="C:/assets")

# 注册自定义协议
webview.register_protocol("fbx", handle_fbx_protocol)

# 在 HTML 中使用
webview.load_html("""
<html>
    <body>
        <h1>FBX Viewer</h1>
        <script>
            // 通过 fetch 加载 FBX 文件
            fetch('fbx://models/character.fbx')
                .then(r => r.arrayBuffer())
                .then(data => {
                    console.log('FBX loaded:', data.byteLength, 'bytes');
                    // 解析 FBX...
                });
        </script>
    </body>
</html>
""")

webview.show()
```

---

## 🔧 实现细节

### Rust 端实现

#### 1. 扩展 `WebViewConfig`

```rust
pub struct WebViewConfig {
    // ... 现有字段
    
    /// 资源根目录（用于 auroraview:// 协议）
    pub asset_root: Option<PathBuf>,
    
    /// 自定义协议处理器（scheme -> handler）
    pub custom_protocols: HashMap<String, ProtocolCallback>,
}
```

#### 2. 集成到 `NativeBackend::create_webview`

```rust
fn create_webview(
    window: &tao::window::Window,
    config: &WebViewConfig,
    ipc_handler: Arc<IpcHandler>,
) -> Result<WryWebView, Box<dyn std::error::Error>> {
    let mut builder = WryWebViewBuilder::new();
    
    // 1. 注册内置 auroraview:// 协议
    if let Some(asset_root) = &config.asset_root {
        let asset_root = asset_root.clone();
        builder = builder.with_custom_protocol("auroraview".into(), move |_id, request| {
            handle_auroraview_protocol(&asset_root, request)
        });
    }
    
    // 2. 注册自定义协议
    for (scheme, handler) in &config.custom_protocols {
        let handler = handler.clone();
        let scheme = scheme.clone();
        builder = builder.with_custom_protocol(scheme, move |_id, request| {
            handle_custom_protocol(&handler, request)
        });
    }
    
    // ... 其他配置
}
```

---

## 📝 完整示例

### Maya 插件示例

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
    "Maya Asset Browser",
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
        <h1>Asset Browser</h1>
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

---

## ✅ 优势

1. **无 CORS 限制** - 自定义协议不受浏览器 CORS 限制
2. **简洁 API** - Python 函数即可注册协议
3. **灵活** - 可以从文件、内存、数据库等任何来源加载资源
4. **安全** - 每个协议独立控制访问权限
5. **高性能** - 直接文件读取，无 HTTP 服务器开销


