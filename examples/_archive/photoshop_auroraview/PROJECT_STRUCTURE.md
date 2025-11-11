# 项目结构说明

## 📁 完整目录树

```
photoshop_auroraview/
│
├── 📄 README.md                      # 英文文档 (完整功能说明)
├── 📄 README_zh.md                   # 中文文档 (完整功能说明)
├── 📄 QUICK_START.md                 # 5分钟快速开始指南
├── 📄 IMPLEMENTATION_SUMMARY.md      # 实现总结
├── 📄 PROJECT_STRUCTURE.md           # 本文档
│
├── 🚀 start.ps1                      # Windows 一键启动脚本
├── 🚀 start.sh                       # macOS/Linux 一键启动脚本
│
├── 🐍 python/                        # Python 后端 (核心逻辑层)
│   ├── photoshop_bridge.py          # WebSocket 服务器
│   ├── image_processor.py           # 图像处理模块 (Pillow, OpenCV)
│   ├── photoshop_tool.py            # 主入口 (AuroraView WebView)
│   └── requirements.txt             # Python 依赖
│
├── 🌐 ui/                            # WebView UI (React + Vite)
│   ├── src/
│   │   ├── App.tsx                  # 主 React 组件
│   │   ├── App.css                  # 样式 (深色主题)
│   │   └── main.tsx                 # React 入口
│   ├── index.html                   # HTML 模板
│   ├── package.json                 # Node.js 依赖
│   ├── tsconfig.json                # TypeScript 配置
│   ├── tsconfig.node.json           # Node TypeScript 配置
│   └── vite.config.ts               # Vite 配置
│
└── 🔌 uxp_plugin/                    # Photoshop UXP 插件 (最小桥接)
    ├── manifest.json                # UXP Manifest v5
    ├── index.html                   # 插件 UI (最小化)
    └── index.js                     # WebSocket 客户端
```

## 📦 核心模块说明

### 1. Python 后端 (`python/`)

#### `photoshop_bridge.py`
**职责**: WebSocket 服务器,处理 Photoshop 通信

**关键类**:
- `PhotoshopBridge`: 主服务器类
  - `start()`: 启动 WebSocket 服务器
  - `register_handler()`: 注册消息处理器
  - `set_webview_callback()`: 设置 WebView 回调
  - `execute_photoshop_command()`: 发送命令到 Photoshop

**依赖**: `websockets`, `asyncio`, `json`

#### `image_processor.py`
**职责**: 图像处理逻辑

**关键类**:
- `ImageProcessor`: 图像处理器
  - `base64_to_image()`: Base64 → PIL Image
  - `image_to_base64()`: PIL Image → Base64
  - `apply_gaussian_blur()`: 高斯模糊
  - `enhance_contrast()`: 对比度增强
  - `sharpen()`: 锐化
  - `edge_detection()`: 边缘检测 (OpenCV)

**依赖**: `Pillow`, `opencv-python`, `numpy`

#### `photoshop_tool.py`
**职责**: 主入口,集成所有组件

**关键类**:
- `PhotoshopTool`: 主工具类
  - `create_webview()`: 创建 AuroraView WebView
  - `start_bridge()`: 启动 WebSocket 服务器
  - `run()`: 运行工具

**依赖**: `auroraview`, `photoshop_bridge`, `image_processor`

### 2. WebView UI (`ui/`)

#### `src/App.tsx`
**职责**: 主 React 组件

**功能**:
- 连接状态显示
- 图像获取按钮
- 滤镜控制面板 (滑块)
- 实时图像预览

**事件监听**:
- `photoshop-connected`: Photoshop 连接
- `layer-created`: 图层创建
- `image-received`: 图像接收

**Python API 调用**:
- `window.auroraview.call('apply_filter', ...)`
- `window.auroraview.call('send_to_photoshop', ...)`
- `window.auroraview.call('get_status')`

#### `src/App.css`
**职责**: 样式定义

**特性**:
- 深色主题 (#1e1e1e 背景)
- 现代化 UI 组件
- 响应式布局
- 自定义滑块样式

### 3. UXP 插件 (`uxp_plugin/`)

#### `manifest.json`
**职责**: UXP 插件配置

**关键配置**:
- `manifestVersion: 5` (最新版本)
- `host: PS` (Photoshop)
- `requiredPermissions.network`: WebSocket 权限

#### `index.js`
**职责**: WebSocket 客户端,Photoshop API 包装

**关键函数**:
- `connect()`: 连接到 Python WebSocket 服务器
- `sendMessage()`: 发送消息到 Python
- `handleMessage()`: 处理来自 Python 的消息
- `executeCommand()`: 执行 Photoshop 命令
- `getActiveLayerImage()`: 获取活动图层图像
- `createLayer()`: 创建新图层

## 🔄 数据流

### 1. 启动流程
```
1. 用户运行 start.ps1/start.sh
   ↓
2. 启动 Vite dev server (http://localhost:5173)
   ↓
3. 启动 Python backend (photoshop_tool.py)
   ↓
4. 创建 AuroraView WebView (加载 Vite UI)
   ↓
5. 启动 WebSocket server (ws://localhost:9001)
   ↓
6. 用户加载 UXP 插件到 Photoshop
   ↓
7. UXP 插件连接到 WebSocket server
   ↓
8. 系统就绪! 🎉
```

### 2. 图像处理流程
```
1. 用户在 WebView UI 点击 "Get Image from Photoshop"
   ↓
2. WebView → Python: call('send_to_photoshop', {command: 'get_active_layer_image'})
   ↓
3. Python → Photoshop: WebSocket message
   ↓
4. Photoshop UXP: 获取图层信息,发送回 Python
   ↓
5. Python → WebView: 触发 'image-received' 事件
   ↓
6. WebView: 显示图像预览
   ↓
7. 用户调整滑块,点击 "Apply Blur"
   ↓
8. WebView → Python: call('apply_filter', {type: 'gaussian_blur', ...})
   ↓
9. Python: 使用 Pillow 处理图像
   ↓
10. Python → WebView: 返回处理后的 Base64 图像
    ↓
11. WebView: 更新预览 🎨
```

## 🛠️ 技术栈

### Python
- **websockets**: 异步 WebSocket 服务器
- **Pillow**: 图像处理
- **OpenCV**: 计算机视觉
- **NumPy**: 数值计算
- **AuroraView**: WebView 框架

### Frontend
- **React 18**: UI 框架
- **TypeScript**: 类型安全
- **Vite 5**: 开发服务器 (HMR)
- **CSS3**: 现代样式

### Photoshop
- **UXP**: 扩展平台
- **Manifest v5**: 最新配置格式
- **WebSocket API**: 网络通信

## 📊 代码统计

| 模块 | 文件数 | 代码行数 | 主要语言 |
|------|--------|---------|---------|
| Python Backend | 3 | ~400 | Python |
| WebView UI | 3 | ~300 | TypeScript/CSS |
| UXP Plugin | 3 | ~200 | JavaScript |
| 文档 | 5 | ~800 | Markdown |
| **总计** | **14** | **~1700** | - |

## 🎯 设计原则

1. **分层架构**: UI、逻辑、数据分离
2. **最小化 UXP**: 仅作为桥接,不包含复杂逻辑
3. **Python 优先**: 核心逻辑在 Python 层
4. **现代化 UI**: 使用最新前端技术
5. **可扩展性**: 易于添加新功能

## 📚 相关文档

- [README.md](./README.md) - 完整英文文档
- [README_zh.md](./README_zh.md) - 完整中文文档
- [QUICK_START.md](./QUICK_START.md) - 快速开始
- [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md) - 实现总结

