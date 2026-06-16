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

# Python 代码保护（可选）
[python.protection]
enabled = true
method = "bytecode"              # "bytecode"（快速）或 "py2pyd"（慢）
optimization = 2

# 加密设置（用于 bytecode 方法）
[python.protection.encryption]
enabled = true
algorithm = "x25519"             # "x25519"（快速）或 "p256"（FIPS 兼容）

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

## Vx 依赖引导

AuroraView Pack 可以在打包阶段下载/内嵌 vx runtime 以及其他资源，提供统一的工具链并支持离线安装。

```toml
[vx]
enabled = true
runtime_url = "https://github.com/loonghao/vx/releases/download/vx-v0.6.10/vx-0.6.10-x86_64-pc-windows-msvc.zip"
runtime_checksum = "<sha256>"
cache_dir = "./.pack-cache/vx"
ensure = ["uv", "node@20", "go@1.22", "rust@stable"]
allow_insecure = false
allowed_domains = ["github.com", "objects.githubusercontent.com"]
block_unknown_domains = false
require_checksum = false

[[downloads]]
name = "vx-runtime"
url = "https://github.com/loonghao/vx/releases/download/vx-v0.6.10/vx-0.6.10-x86_64-pc-windows-msvc.zip"
checksum = "<sha256>"
extract = true
strip_components = 1
stage = "before_collect"
dest = "python/bin/vx"
executable = ["vx.exe"]

[hooks]
use_vx = true

[hooks.vx]
before_collect = ["vx --version"]
after_pack = ["vx uv pip list"]
```

- `downloads.stage` 支持 `before_collect`、`before_pack`、`after_pack`。
- `hooks.use_vx` 会用 `vx` 包装原有 hooks，`hooks.vx.*` 始终通过 vx 执行。
- 设定 `AURORAVIEW_OFFLINE=1` 时仅使用缓存制品。
- 运行时安装器在检测到 `AURORAVIEW_VX_PATH` 或 PATH 中存在 `vx` 时优先使用 `vx uv pip`。

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

## 代码保护

AuroraView 提供两种方法来保护你的 Python 源代码，防止逆向工程：

### 保护方法

| 方法 | 速度 | 依赖 | 保护级别 | 描述 |
|------|------|------|----------|------|
| `bytecode` | **快** | 仅 Python | 高 | 使用 ECC + AES-256-GCM 加密 Python 字节码 |
| `py2pyd` | 慢 | C/C++ 编译器 | 最高 | 通过 Cython 编译为原生 `.pyd`/`.so` |

### 字节码加密（推荐）

`bytecode` 方法是默认且推荐的方式：

1. **编译** `.py` 文件为 `.pyc` 字节码
2. **加密** 字节码使用 AES-256-GCM（对称加密）
3. **保护** AES 密钥使用 ECC（X25519 或 P-256）
4. **解密** 运行时通过引导加载器解密

```
┌─────────────────────────────────────────────────────────────────┐
│                        打包时（构建）                            │
├─────────────────────────────────────────────────────────────────┤
│  .py ──► py_compile ──► .pyc 字节码                             │
│                              │                                  │
│                              ▼                                  │
│                    AES-256-GCM 加密                             │
│                              │                                  │
│                              ▼                                  │
│                      加密的 .pyc.enc                            │
│                                                                 │
│  同时: AES 密钥 ──► ECC 公钥加密 ──► 加密的密钥                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                       运行时（用户机器）                         │
├─────────────────────────────────────────────────────────────────┤
│  1. 加密密钥 ──► 嵌入的私钥解密 ──► AES 密钥                    │
│                                                                 │
│  2. .pyc.enc ──► AES 解密 ──► .pyc 字节码（~GB/s）              │
│                                                                 │
│  3. marshal.loads() + exec() 执行                               │
└─────────────────────────────────────────────────────────────────┘
```

**配置：**

```toml
[python.protection]
enabled = true
method = "bytecode"              # 快速，不需要 C 编译器
optimization = 2                 # Python 字节码优化级别 (0-2)

[python.protection.encryption]
enabled = true
algorithm = "x25519"             # "x25519"（快速）或 "p256"（FIPS 兼容）
```

**加密算法：**

| 算法 | 速度 | 安全性 | 使用场景 |
|------|------|--------|----------|
| `x25519` | **快** | 现代，128位安全 | 默认，推荐 |
| `p256` | 中等 | NIST/FIPS 兼容 | 政府/企业 |

### py2pyd 编译（最高保护）

`py2pyd` 方法将 Python 编译为原生机器码：

1. **转换** `.py` 为 C 代码（通过 Cython）
2. **编译** C 代码为原生 `.pyd`（Windows）或 `.so`（Linux/macOS）
3. **替换** 原始 `.py` 文件为编译后的扩展

**配置：**

```toml
[python.protection]
enabled = true
method = "py2pyd"                # 慢，需要 C 编译器
optimization = 3                 # C 编译器优化级别 (0-3)
keep_temp = false                # 保留临时文件用于调试
```

**依赖：**
- C/C++ 编译器（Windows 上是 MSVC，Linux/macOS 上是 GCC/Clang）
- Cython（通过 uv 自动安装）

**注意：** 此方法明显较慢，因为它为每个被编译的文件创建一个新的虚拟环境。

### 排除文件

你可以排除特定文件或模式：

```toml
[python.protection]
enabled = true
method = "bytecode"
exclude = [
    "config.py",           # 保持配置可读
    "**/tests/**",         # 跳过测试文件
    "setup.py",            # 跳过安装文件
]
```

## 命令行模式（Headless CLI）

打包后的可执行文件默认是 GUI 应用——双击（或无参数启动）照常打开窗口。它**同时**支持在终端里直接运行已注册的 Python 命令，无需打开窗口。

```bash
my-app.exe                          # 打开 GUI 窗口（行为不变）
my-app.exe -h                       # 列出已开启 CLI 的命令
my-app.exe list                     # 列出命令（加 --json 输出机器可读格式）
my-app.exe run export --path ./out  # 执行单个命令，打印结果后退出
my-app.exe -V                       # 打印版本
```

只有当**首个参数**是保留动词（`run`、`list`）或保留标志（`-h`/`--help`、`-V`/`--version`）时才进入 CLI 路径。裸文件路径（文件关联、拖拽）一律打开 GUI，因此不会破坏既有行为。

### 把命令暴露给 CLI

CLI 暴露是**显式开启**的——命令默认仅 GUI 可用。通过 `@webview.command` 的 `cli` 参数开启：

```python
@webview.command(name="export", help="导出工程", cli=True)
def export(path: str, dpi: int = 300) -> dict:
    return {"written": path, "dpi": dpi}

# 指定短别名：
@webview.command(name="export-document-image", cli="exi")
def export_document_image(path: str) -> dict: ...

# 多个别名：
@webview.command(name="validate", cli=["val", "v"])
def validate() -> dict: ...
```

`cli` 取值同时承担开关与别名两个职责：

| `cli` 取值           | 含义                       |
|----------------------|----------------------------|
| `False`（默认）      | 仅 GUI，不暴露到命令行     |
| `True`               | 暴露，无别名               |
| `"exi"`              | 暴露，别名 `exi`           |
| `["exi", "edi"]`     | 暴露，多个别名             |

用 `help=` 和 `args_help={...}` 丰富 `-h` 输出；缺省时回退到 docstring / 签名。批量开启已有命令可调用 `webview.commands.enable_cli("export", "validate")`，或传入 `{名称: 别名}` 映射。

### 传参

支持关键字与位置两种形态，可混用（位置在前，关键字在后）：

```bash
my-app.exe run export --path ./out --dpi 600   # 关键字
my-app.exe run export ./out 600                 # 位置（按签名顺序）
my-app.exe run export ./out --dpi 600           # 混用
```

参数按类型注解（`int`/`float`/`bool`/`str`）转换，其余按 JSON 解析（`--config '{"a":1}'`）。布尔支持 `--flag` / `--no-flag`。退出码：`0` 成功，`1` 命令抛异常，`2` 命令未找到或参数错误。

### 实现原理

`-h`/`list` 显示的命令清单在**打包期**采集并嵌入 overlay，因此这两个命令毫秒级返回、完全不启动 Python。采集时以 `AURORAVIEW_CLI_DUMP=1` 跑一次打包的入口。由于需要在构建主机上运行打包的解释器，跨平台打包或非 `standalone` 策略时会跳过（并打印警告）——此时 `-h`/`list` 不列出任何命令，需在匹配的主机上重新打包。`run` 首次执行时解压 Python 运行时（之后缓存复用），在一次性进程中调用命令——不复用 GUI 的常驻后端。

Windows 上打包 exe 启动时会附着到父控制台，使 CLI 输出能回到终端（双击仍无黑窗）。macOS/Linux 无 GUI 子系统的 stdio 隔离问题，输出直接生效。

### Windows `.cmd` 包装脚本

打包 exe 是 GUI 子系统的，双击不会闪出黑窗。代价是 `cmd.exe` 和 PowerShell **不会等待** GUI 子系统进程——`app.exe run export ...` 会在命令输出落地之前就返回提示符，退出码也会丢失。

为此，当打包产物暴露了 CLI 命令、且**不是**以 console 子系统构建（默认情况）时，打包器会在 exe 旁生成一个 `<name>.cmd` 包装脚本：

```bat
@echo off
start "" /wait /b "%~dp0app.exe" %*
exit /b %ERRORLEVEL%
```

`start /wait` 同步运行 exe，使终端阻塞到其结束；`%*` 透传所有参数；`exit /b %ERRORLEVEL%` 传递退出码。`%~dp0` 以脚本自身所在目录解析 exe，因此与调用方当前目录无关。

在终端里请调用 `.cmd`，输出与退出码才会正确：

```bash
app.cmd -h                       # 阻塞，打印帮助后返回
app.cmd run export --path ./out  # 阻塞直到命令结束
app.exe                          # 双击 / GUI 启动仍使用 exe
```

该脚本是 best-effort 的：若无法写入（例如输出目录只读），构建仍会成功，只打印一条警告。console 子系统构建（`[bundle.platform.windows] console = true`）不会生成它——此时 exe 本身就会阻塞终端。

## 最佳实践

1. **使用 `site-packages` 存放依赖**: 所有第三方包放到 `python/site-packages/`

2. **使用 `bin/` 存放可执行文件**: 外部二进制文件放到 `python/bin/`

3. **分离 API 和 UI 逻辑**: 打包模式下，Python 只提供 API，Rust 处理 UI

4. **处理两种模式**: 设计代码同时支持开发和打包模式

5. **使用环境变量**: 检查 `AURORAVIEW_PACKED` 来适配行为

6. **日志输出到 stderr**: 打包模式下，stdout 用于 JSON-RPC，使用 stderr 记录日志

7. **生产环境使用字节码保护**: 启用 `[python.protection]` 并设置 `method = "bytecode"` 以获得快速、安全的代码保护
