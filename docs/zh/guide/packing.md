# 应用打包

本指南介绍 AuroraView 打包系统，它可以将前端资源、Python 代码和依赖打包成单个可执行文件。

## 概述

AuroraView 支持多种打包策略以适应不同的部署场景：

| 策略 | 描述 | 输出大小 | 使用场景 |
|------|------|----------|----------|
| `standalone` | 嵌入 Python 运行时 (python-build-standalone) | ~50-100MB | 完全离线分发 |
| `pyoxidizer` | 基于 PyOxidizer 的嵌入 | ~30-50MB | 单文件优化版 |
| `embedded` | Overlay 模式，需要系统 Python | ~15MB | 系统有 Python 时 |
| `portable` | 目录形式，包含运行时 | 不定 | 开发/测试 |
| `system` | 使用系统 Python | ~5MB | 最小分发 |

## 架构

### 开发模式 vs 打包模式

**开发模式**（未打包）:
```
Python main.py
    ├── 创建 WebView（通过 Rust 绑定）
    ├── 加载前端 (dist/index.html)
    ├── 注册 API 回调 (@view.bind_call)
    ├── 创建 PluginManager
    └── view.show() - 启动事件循环

[Python 是主进程，直接控制 WebView]
```

**打包模式**:
```
app.exe (Rust)
    ├── 解压资源和 Python 运行时
    ├── 创建 WebView
    ├── 加载前端（从 overlay）
    ├── 启动 Python 后端进程 (main.py)
    │       └── 作为 API 服务器运行 (JSON-RPC over stdin/stdout)
    └── 事件循环 (Rust 主线程)

[Rust 是主进程，Python 是提供 API 的子进程]
```

这是一个**前后端分离架构**：
- Rust 控制 WebView 生命周期（更稳定）
- Python 崩溃不影响 UI
- 可以重启 Python 后端而不重启 UI
- 更好的进程隔离和错误处理

## 配置

### 打包配置文件 (auroraview.pack.toml)

```toml
# ============================================================================
# 包信息
# ============================================================================
[package]
name = "my-app"
version = "1.0.0"
description = "我的应用描述"
authors = ["Your Name <your@email.com>"]
license = "MIT"

# ============================================================================
# 应用配置
# ============================================================================
[app]
title = "我的应用"
frontend_path = "./dist"         # 构建后的前端目录
# url = "https://example.com"    # 替代方案：从 URL 加载

# ============================================================================
# 窗口配置
# ============================================================================
[window]
width = 1280
height = 720
min_width = 800
min_height = 600
resizable = true
frameless = false
start_position = "center"

# ============================================================================
# 打包配置
# ============================================================================
[bundle]
icon = "./assets/my-app-icon.png"
identifier = "com.mycompany.myapp"
copyright = "Copyright 2025 My Company"

# ============================================================================
# 平台特定打包配置
# ============================================================================
[bundle.platform.windows]
console = false                  # GUI 应用隐藏控制台
file_version = "1.0.0.0"
product_version = "1.0.0"

# ============================================================================
# Python 后端配置（全栈模式）
# ============================================================================
[python]
enabled = true
version = "3.11"
entry_point = "main:run"         # module:function 格式
packages = ["pyyaml", "requests"]
include_paths = [".", "src"]
exclude = ["__pycache__", "*.pyc", "tests"]
strategy = "standalone"          # standalone, pyoxidizer, embedded, portable, system

# ============================================================================
# 构建配置
# ============================================================================
[build]
before = ["npm run build"]       # 构建前命令
after = []                       # 构建后命令
out_dir = "./pack-output"
release = true

# ============================================================================
# 运行时环境配置
# ============================================================================
[runtime]
[runtime.env]
APP_ENV = "production"
LOG_LEVEL = "info"
```

## 构建

### 使用 Just 命令

```bash
# 构建并打包 Gallery
just gallery-pack

# 使用自定义配置打包
just pack --config path/to/config.toml
```

### 直接使用 CLI

```bash
# 打包应用
cargo run -p auroraview-cli --release -- pack --config auroraview.pack.toml

# 带构建步骤打包
cargo run -p auroraview-cli --release -- pack --config auroraview.pack.toml --build
```

## 运行时行为

### 环境变量

打包后的可执行文件为 Python 后端设置以下环境变量：

| 变量 | 描述 |
|------|------|
| `AURORAVIEW_PACKED` | 打包模式下设为 `"1"` |
| `AURORAVIEW_RESOURCES_DIR` | 解压资源路径 |
| `AURORAVIEW_EXAMPLES_DIR` | 示例路径（如果存在） |
| `AURORAVIEW_PYTHON_PATH` | 模块搜索路径 |

### 在 Python 中检测打包模式

```python
import os

PACKED_MODE = os.environ.get("AURORAVIEW_PACKED", "0") == "1"

if PACKED_MODE:
    # 运行在打包模式
    resources_dir = os.environ.get("AURORAVIEW_RESOURCES_DIR")
    # 作为 API 服务器运行
    run_api_server()
else:
    # 运行在开发模式
    # 直接创建 WebView
    view = WebView(...)
    view.show()
```

### JSON-RPC 协议

打包模式下，Python 通过 stdin/stdout 的 JSON-RPC 与 Rust 通信：

**请求** (Rust → Python):
```json
{
    "id": "unique-id",
    "method": "api.get_samples",
    "params": {}
}
```

**响应** (Python → Rust):
```json
{
    "id": "unique-id",
    "ok": true,
    "result": [...]
}
```

## 故障排除

### 常见问题

**模块未找到错误**:
- 检查 `module_search_paths` 配置
- 验证包是否在 `packages` 列表中
- 检查打包时包是否成功收集

**Python 后端未启动**:
- 检查 `entry_point` 格式（`module:function` 或 `file.py`）
- 验证 Python 版本兼容性
- 检查 stderr 输出的错误

### 调试模式

启用调试日志：

```bash
RUST_LOG=debug ./my-app.exe
```

或在配置中：
```toml
[debug]
enabled = true
verbose = true
```

## 最佳实践

1. **使用 `site-packages` 存放依赖**: 所有第三方包放到 `python/site-packages/`

2. **使用 `bin/` 存放可执行文件**: 外部二进制文件放到 `python/bin/`

3. **分离 API 和 UI 逻辑**: 打包模式下，Python 只提供 API，Rust 处理 UI

4. **处理两种模式**: 设计代码同时支持开发和打包模式

5. **使用环境变量**: 检查 `AURORAVIEW_PACKED` 来适配行为

6. **日志输出到 stderr**: 打包模式下，stdout 用于 JSON-RPC，使用 stderr 记录日志
