# Custom Protocol Handler 设计文档

## 📋 概述

`protocol.rs` 提供了自定义协议处理器，允许 WebView 加载自定义 URI scheme 的资源（如 `dcc://`, `asset://`, `maya://`）。

---

## 🎯 设计目标

### 1. **DCC 资源加载**
在 DCC 应用（Maya、Houdini、Nuke）中，资源通常存储在特定位置：
- Maya 场景文件：`maya://scenes/character.ma`
- 纹理资源：`asset://textures/diffuse.png`
- 插件资源：`dcc://plugins/my_tool/ui.html`

### 2. **虚拟文件系统**
允许从内存、数据库或网络加载资源，而不是磁盘文件：
- 从 Python 字典加载：`memory://config.json`
- 从数据库加载：`db://assets/model_123`
- 从缓存加载：`cache://thumbnails/shot_001.jpg`

### 3. **安全隔离**
自定义协议可以实现权限控制：
- 只允许访问特定目录
- 验证文件类型和大小
- 记录访问日志

---

## 🏗️ 架构设计

### 当前实现（未集成）

```rust
// src/webview/protocol.rs
pub struct ProtocolHandler {
    handlers: Arc<Mutex<HashMap<String, ProtocolCallback>>>,
}

impl ProtocolHandler {
    pub fn register<F>(&self, scheme: &str, handler: F)
    where F: Fn(&str) -> Option<ProtocolResponse> + Send + Sync + 'static
    {
        // 注册自定义协议处理器
    }
    
    pub fn handle(&self, uri: &str) -> Option<ProtocolResponse> {
        // 处理协议请求
    }
}
```

### 需要集成到 Wry

Wry 提供了 `with_custom_protocol` 方法：

```rust
use wry::WebViewBuilder;

WebViewBuilder::new()
    .with_custom_protocol("dcc".into(), |_webview_id, request| {
        // 处理 dcc:// 协议请求
        let path = request.uri().path();
        
        // 读取文件或生成内容
        let content = load_dcc_resource(path);
        
        http::Response::builder()
            .header("Content-Type", "text/html")
            .body(content.into())
            .unwrap()
    })
    .build(&window)
    .unwrap();
```

---

## 💡 使用场景

### 场景 1: Maya 场景资源加载

```python
from auroraview import WebView

webview = WebView(
    title="Maya Asset Browser",
    width=800,
    height=600,
    html="""
    <html>
        <body>
            <h1>Maya Assets</h1>
            <img src="maya://thumbnails/character_rig.jpg">
            <script src="maya://scripts/asset_loader.js"></script>
        </body>
    </html>
    """
)

# 在 Rust 端注册 maya:// 协议
# 当 WebView 请求 maya://thumbnails/character_rig.jpg 时
# 从 Maya 项目目录加载文件
```

**Rust 实现**:
```rust
webview_builder.with_custom_protocol("maya".into(), |_id, request| {
    let path = request.uri().path();
    
    // 从 Maya 项目目录加载
    let maya_project = std::env::var("MAYA_PROJECT").unwrap();
    let full_path = format!("{}/{}", maya_project, path);
    
    match std::fs::read(&full_path) {
        Ok(data) => {
            let mime = mime_guess::from_path(&full_path)
                .first_or_octet_stream()
                .to_string();
            
            http::Response::builder()
                .header("Content-Type", mime)
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

---

### 场景 2: 虚拟文件系统

```python
# Python 端提供资源
webview = WebView(...)

# 注册虚拟资源
webview.register_virtual_file("config://app.json", {
    "theme": "dark",
    "language": "en"
})

# HTML 中使用
# <script>
#   fetch('config://app.json')
#     .then(r => r.json())
#     .then(config => console.log(config))
# </script>
```

**Rust 实现**:
```rust
// 使用 Arc<Mutex<HashMap>> 存储虚拟文件
let virtual_fs = Arc::new(Mutex::new(HashMap::new()));
let virtual_fs_clone = virtual_fs.clone();

webview_builder.with_custom_protocol("config".into(), move |_id, request| {
    let path = request.uri().path();
    let fs = virtual_fs_clone.lock().unwrap();
    
    if let Some(content) = fs.get(path) {
        http::Response::builder()
            .header("Content-Type", "application/json")
            .body(content.clone().into())
            .unwrap()
    } else {
        http::Response::builder()
            .status(404)
            .body(b"Not Found".to_vec().into())
            .unwrap()
    }
})
```

---

### 场景 3: 嵌入式资源（编译时）

```rust
// 使用 include_bytes! 嵌入资源
const LOGO: &[u8] = include_bytes!("../assets/logo.png");
const STYLE: &str = include_str!("../assets/style.css");

webview_builder.with_custom_protocol("app".into(), |_id, request| {
    match request.uri().path() {
        "/logo.png" => {
            http::Response::builder()
                .header("Content-Type", "image/png")
                .body(LOGO.to_vec().into())
                .unwrap()
        }
        "/style.css" => {
            http::Response::builder()
                .header("Content-Type", "text/css")
                .body(STYLE.as_bytes().to_vec().into())
                .unwrap()
        }
        _ => {
            http::Response::builder()
                .status(404)
                .body(b"Not Found".to_vec().into())
                .unwrap()
        }
    }
})
```

---

## 🔧 集成方案

### 方案 A: 直接使用 Wry API（推荐）

**优点**:
- 简单直接，不需要额外抽象层
- 与 Wry 原生 API 一致
- 零运行时开销

**缺点**:
- 需要在创建 WebView 时注册
- 不能动态添加/删除协议

**实现**:
```rust
// 在 WebViewConfig 中添加
pub struct WebViewConfig {
    // ... 现有字段
    pub custom_protocols: Vec<(String, ProtocolCallback)>,
}

// 在 backend/native.rs 中使用
let mut builder = WebViewBuilder::new();
for (scheme, handler) in config.custom_protocols {
    builder = builder.with_custom_protocol(scheme, handler);
}
```

---

### 方案 B: 保留 ProtocolHandler 抽象层

**优点**:
- 提供更高级的 API
- 可以动态管理协议
- 统一的错误处理

**缺点**:
- 增加复杂度
- 需要维护额外代码

**实现**: 保留当前 `protocol.rs`，在 WebView 创建时桥接到 Wry

---

## 📊 当前状态

| 组件 | 状态 | 说明 |
|------|------|------|
| `protocol.rs` | ✅ 已实现 | 抽象层完整 |
| Wry 集成 | ❌ 未集成 | 需要在 backend/native.rs 中调用 |
| Python API | ❌ 未暴露 | 需要添加 PyO3 绑定 |
| 文档 | ⚠️ 部分 | 需要添加使用示例 |

---

## 🚀 建议

### 短期（保留）
1. **保留 protocol.rs** - 作为未来功能的设计参考
2. **添加 TODO 注释** - 说明何时启用和如何集成
3. **添加使用示例** - 在注释中展示预期用法

### 长期（实现）
1. **集成到 WebViewConfig** - 添加 `custom_protocols` 字段
2. **暴露 Python API** - 允许从 Python 注册协议
3. **添加内置协议** - 如 `asset://`, `dcc://`

---

## 📝 推荐的 TODO 注释

```rust
//! Custom protocol handler for loading resources
//!
//! **Status**: Not yet integrated with Wry backend
//!
//! **TODO**: Integrate with WebViewBuilder::with_custom_protocol
//! **TODO**: Add Python API for registering protocols
//! **TODO**: Add built-in protocols (asset://, dcc://)
//!
//! **Use cases**:
//! - Loading DCC resources (maya://scenes/file.ma)
//! - Virtual file system (memory://config.json)
//! - Embedded assets (app://logo.png)
```

---

您觉得这个设计如何？是否保留 `protocol.rs` 作为未来功能？

