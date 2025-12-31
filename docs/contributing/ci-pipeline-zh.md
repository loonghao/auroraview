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
