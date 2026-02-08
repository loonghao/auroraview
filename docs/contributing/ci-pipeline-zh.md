# CI 流水线架构

本文档描述了 AuroraView 的 CI/CD 流水线架构，针对包隔离和高效构建进行了优化。

## 概述

AuroraView 使用**包隔离 CI 策略**，每个包（Rust crates、SDK、MCP、Gallery、Docs）都有自己的 CI 工作流。这种方法：

- **减少 CI 时间**：只构建和测试受影响的包
- **改善反馈**：针对性更改可获得更快的反馈
- **尊重依赖关系**：依赖链自动触发下游测试

## 包结构

```
AuroraView
├── Rust Crates
│   ├── aurora-signals (独立)
│   ├── aurora-protect (独立)
│   ├── auroraview-plugin-core (独立)
│   ├── auroraview-plugin-fs → plugin-core
│   ├── auroraview-extensions (独立)
│   ├── auroraview-plugins → plugin-core, plugin-fs, extensions
│   ├── auroraview-core → signals, plugins
│   ├── auroraview-pack → protect (可选)
│   ├── auroraview-cli → core, pack
│   └── auroraview (根) → core, signals
├── 前端包
│   ├── @auroraview/sdk (TypeScript)
│   └── auroraview-gallery → SDK
├── Python 包
│   ├── auroraview (Python 绑定)
│   └── auroraview-mcp (MCP 服务器)
└── 文档
    └── docs (VitePress)
```

## 工作流文件

| 工作流 | 用途 | 触发条件 |
|--------|------|----------|
| `pr-checks.yml` | PR 验证 | Pull requests |
| `rust-crates-ci.yml` | Rust crate 测试 | Crate 变更 |
| `python-ci.yml` | Python 测试 | Python 变更 |
| `sdk-ci.yml` | SDK 构建和测试 | SDK 变更 |
| `mcp-ci.yml` | MCP 服务器 CI | MCP 变更 |
| `docs.yml` | 文档 | 文档变更 |
| `build-gallery.yml` | Gallery 打包 | 发布 |

## 依赖链检测

当文件发生变化时，CI 会根据依赖图自动检测哪些包需要测试。

### 示例：`aurora-signals` 变更

```
aurora-signals 变更
    └── 触发: auroraview-core (依赖 signals)
        └── 触发: auroraview-cli (依赖 core)
            └── 触发: auroraview (根, 依赖 core)
```

### 示例：`auroraview-plugin-core` 变更

```
auroraview-plugin-core 变更
    ├── 触发: auroraview-plugin-fs (依赖 plugin-core)
    └── 触发: auroraview-plugins (依赖 plugin-core)
        └── 触发: auroraview-core (依赖 plugins)
            └── 触发: auroraview-cli, auroraview (根)
```

## 本地开发命令

使用 `just` 命令进行包级别测试：

```bash
# 测试单个 crate
just test-signals          # aurora-signals
just test-protect          # aurora-protect
just test-plugin-core      # auroraview-plugin-core
just test-plugin-fs        # auroraview-plugin-fs
just test-extensions       # auroraview-extensions
just test-plugins          # auroraview-plugins
just test-core             # auroraview-core
just test-pack             # auroraview-pack
just test-cli              # auroraview-cli

# 测试组
just test-standalone       # 所有独立 crate
just test-python           # 仅 Python 测试
just test-python-unit      # Python 单元测试
just test-python-integration  # Python 集成测试

# SDK 和 Gallery
just sdk-test              # SDK 单元测试
just sdk-ci                # 完整 SDK CI
just gallery-test          # Gallery E2E 测试

# MCP
just mcp-test              # MCP 测试
just mcp-ci                # 完整 MCP CI
```

## 路径过滤器

CI 使用路径过滤器来确定运行哪些工作流：

| 类别 | 路径 | 触发 |
|------|------|------|
| `rust` | `src/**`, `crates/**`, `Cargo.*` | Rust 构建, wheel 构建 |
| `python` | `python/**`, `tests/python/**` | Python 测试 |
| `sdk` | `packages/auroraview-sdk/**` | SDK 构建 |
| `mcp` | `packages/auroraview-mcp/**` | MCP 构建 |
| `gallery` | `gallery/**` | Gallery E2E |
| `docs` | `docs/**`, `*.md` | 文档构建 |
| `ci` | `.github/**`, `justfile` | 所有检查 |

## 制品复用

为避免重复构建，制品在作业之间共享：

1. **SDK 资源**：构建一次，用于 wheel 构建和 Gallery
2. **Wheels**：每个平台构建一次，用于 Python 测试和 Gallery 打包
3. **CLI**：每个平台构建一次，用于 Gallery 打包

## 最佳实践

### 对于贡献者

1. **聚焦变更**：保持 PR 专注于特定包
2. **运行本地测试**：推送前使用 `just test-<package>`
3. **检查 CI 摘要**：查看 PR 检查中的"检测到的变更"摘要

### 对于维护者

1. **监控 CI 时间**：跟踪每个包的构建时间
2. **更新依赖**：保持依赖图与 `Cargo.toml` 同步
3. **缓存优化**：确保缓存键是包特定的

## 发布工作流

发布流程由 `.github/workflows/release.yml` 处理，管理以下内容：

1. **版本管理**: 使用 `release-please` 自动化版本升级和变更日志生成
2. **Wheel 构建**: 为所有支持的平台构建平台特定的 wheel
3. **包发布**: 发布到 PyPI（Python）和 npm（TypeScript SDK）
4. **GitHub 发布**: 创建包含 CLI 二进制文件和 Gallery 可执行文件的发布资源

### 支持的平台

| 平台 | 架构 | Python Wheel | PyPI 上传 | GitHub 发布 |
|------|------|-------------|-----------|-------------|
| Windows | x64 (amd64) | ✅ 是 | ✅ 是 | ✅ 是 |
| macOS | universal2 (x64+ARM64) | ✅ 是 | ✅ 是 | ✅ 是 |
| Linux | x86_64 | ✅ 是 | ❌ 否 | ✅ 是 |
| Windows | ARM64 | ❌ 否 | ❌ 否 | 仅 CLI/Gallery |
| Linux | ARM64 | ❌ 否 | ❌ 否 | 仅 CLI/Gallery |

**为什么没有 ARM64 Python wheel？**

- **Linux ARM64**：`wry` 依赖 `webkit2gtk`，需要原生 ARM64 系统库（`libwebkit2gtk-4.1-dev`）。从 x86_64 交叉编译需要完整的 ARM64 sysroot 及这些库，极其复杂且不可靠。`pkg-config` 在没有正确配置交叉编译 sysroot 的情况下无法解析 ARM64 库路径。
- **Windows ARM64**：`maturin` 需要与目标架构匹配的 Python 解释器来确定正确的 wheel 文件名和 ABI 标签。GitHub Actions 运行器仅提供 x86_64 Python 解释器。
- **解决方法**：ARM64 用户可以从源码构建（`pip install .`，需要 Rust 工具链），或使用 CLI/Gallery ARM64 二进制文件（纯 Rust 构建，不依赖 Python）。

注意：Linux x86_64 wheel 不会上传到 PyPI，因为它们需要系统库（webkit2gtk）并使用非标准平台标签。Linux 用户应从 GitHub 发布页面安装或从源代码构建。

### NPM 发布

SDK 作为 `@auroraview/sdk` 发布到 npm。如果发布失败：

1. **Token 过期**: 在 https://www.npmjs.com/settings/loonghao/tokens 生成新令牌
2. **创建自动化令牌**: 选择具有发布权限的 "Automation" 类型
3. **更新 GitHub Secret**: 在仓库设置中设置 `NPM_TOKEN`
4. **验证包访问权限**: 确保包存在并且您具有发布权限

### PyPI 发布

Python 包作为 `auroraview` 发布到 PyPI。关键注意事项：

1. **文件大小限制**: PyPI 对每个文件有 100MB 限制。源代码分发包（sdist）通常由于捆绑的资源而超过此限制，因此它们仅单独构建用于 GitHub 发布。
2. **平台标签**: 只有 Windows 和 macOS 的 wheel 会上传到 PyPI。Linux wheel 使用非标准标签并被排除。
3. **ABI3 支持**: Python 3.8+ 使用 abi3（稳定 ABI），每个平台只需一个 wheel。Python 3.7 需要单独的非 abi3 构建。

## 故障排除

### CI 意外运行所有检查

- 检查是否修改了 `.github/**` 或 `justfile`（触发所有检查）
- 验证路径过滤器配置正确

### 依赖未被检测到

- 确保依赖在工作流的依赖链计算中列出
- 检查 `rust-crates-ci.yml` 中的依赖图逻辑

### 缓存未命中

- 缓存键基于 `Cargo.lock` 哈希
- 不同的包可能有不同的缓存键

### NPM 发布失败，返回 404

错误：`404 Not Found - PUT https://registry.npmjs.org/@auroraview%2fsdk`

**解决方案**：
1. 验证 GitHub 仓库 secrets 中已设置 `NPM_TOKEN`
2. 在 https://www.npmjs.com/settings/loonghao/tokens 生成新令牌
3. 使用 "Automation" 令牌类型（不是 "Publish"）
4. 确保令牌未过期

### PyPI 发布失败，提示"文件太大"

错误：`400 File too large. Limit for project 'auroraview' is 100 MB`

**解决方案**：
- 对于包含 Rust 代码和资源的源代码分发包（sdist），这是预期的
- CI 工作流会自动将 sdist 从 PyPI 上传中排除
- sdist 单独构建并仅上传到 GitHub 发布
- 需要源代码的用户可以从 GitHub 发布页面下载

### ARM64 构建失败

ARM64 Python wheel 构建已从 CI 中移除，原因是根本性的交叉编译限制：

- **Linux ARM64**：`wry`/`webkit2gtk` 在没有完整 ARM64 sysroot 的情况下无法交叉编译。`pkg-config` 报错 "not been configured to support cross-compilation"，因为 x86_64 运行器上没有 ARM64 开发库。
- **Windows ARM64**：`maturin` 找不到匹配的 Python 解释器。错误："Need a Python interpreter to compile for Windows without PyO3's generate-import-lib feature"。即使启用 `generate-import-lib`，maturin 仍需要解释器来生成 wheel 打包元数据。

CLI 和 Gallery 的 ARM64 二进制文件（纯 Rust，无 Python 依赖）仍通过交叉编译构建，可在 GitHub Releases 中获取。
