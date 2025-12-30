# Photoshop 集成

AuroraView 通过 UXP 插件系统和 WebSocket 通信与 Adobe Photoshop 集成。

## 架构

```
┌─────────────────────────────────────────────┐
│              Photoshop                       │
├─────────────────────────────────────────────┤
│  ┌─────────────┐      ┌──────────────────┐ │
│  │  UXP 面板   │ ◄──► │  AuroraView      │ │
│  │  (WebView)  │      │  Python 后端     │ │
│  └─────────────┘      └──────────────────┘ │
│         │                      │            │
│         │ WebSocket            │            │
│         ▼                      ▼            │
│  ┌─────────────────────────────────────┐   │
│  │      Photoshop API (batchPlay)      │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

## 要求

| 组件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Adobe Photoshop | 24.0 (2023) | 26.0+ (2025) |
| Rust | 1.70 | 1.75+ |
| Node.js（可选） | 16.x | 20.x+ |
| 操作系统 | Windows 10, macOS 11 | Windows 11, macOS 14+ |

## 设置指南

### 步骤 1：安装 UXP Developer Tool

1. 打开 **Creative Cloud Desktop**
2. 转到 **所有应用**
3. 搜索 "UXP Developer Tool"
4. 点击 **安装**

### 步骤 2：构建 WebSocket 服务器

```bash
cd examples/photoshop_examples
cargo build --release
```

### 步骤 3：启动服务器

```bash
RUST_LOG=info cargo run --bin websocket_server
```

### 步骤 4：加载 UXP 插件

1. 启动 **UXP Developer Tool**
2. 确保 Photoshop 正在运行
3. 点击 **Add Plugin**
4. 选择 `examples/photoshop_examples/uxp_plugin/manifest.json`
5. 点击 **Load**

## 计划中的 API

```python
from auroraview import PhotoshopWebView

class PhotoshopAPI:
    def get_active_document(self) -> dict:
        """获取活动文档信息"""
        pass

    def get_layers(self) -> dict:
        """获取文档图层"""
        pass

    def select_layer(self, name: str) -> dict:
        """按名称选择图层"""
        pass

webview = PhotoshopWebView(
    url="http://localhost:3000",
    api=PhotoshopAPI()
)
webview.show()
```

## 开发状态

| 功能 | 状态 |
|------|------|
| 基础集成 | 🚧 开发中 |
| 图层管理 | 📋 计划中 |
| 滤镜应用 | 📋 计划中 |
| 选区同步 | 📋 计划中 |

## 资源

- [Adobe UXP 文档](https://developer.adobe.com/photoshop/uxp/)
- [Photoshop API 参考](https://developer.adobe.com/photoshop/uxp/2022/ps_reference/)
