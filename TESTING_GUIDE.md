# AuroraView 测试指南

## 快速开始

### 安装依赖
```bash
just install
```

### 构建项目
```bash
just build
```

### 运行所有测试
```bash
just test
```

## 本地测试命令

### 基础测试

#### 运行所有测试
```bash
just test
```
运行 Rust 和 Python 的所有测试。

#### 运行快速测试
```bash
just test-fast
```
跳过标记为 `slow` 的测试。

#### 运行单元测试
```bash
just test-unit
```
仅运行单元测试。

#### 运行集成测试
```bash
just test-integration
```
仅运行集成测试。

#### 运行特定测试文件
```bash
just test-file tests/test_basic.py
```

#### 运行特定标记的测试
```bash
just test-marker unit
```

### 代码质量检查

#### 格式化代码
```bash
just format
```
自动格式化 Rust 和 Python 代码。

#### 运行 Linting
```bash
just lint
```
检查代码质量（Clippy、Ruff）。

#### 自动修复问题
```bash
just fix
```
自动修复 Linting 问题。

#### 运行所有检查
```bash
just check
```
运行格式化、Linting 和测试。

### 覆盖率报告

#### Python 覆盖率
```bash
just coverage-python
```
生成 Python 代码覆盖率报告（HTML 和 XML）。

#### Rust 覆盖率
```bash
just coverage-rust
```
生成 Rust 代码覆盖率报告（需要 cargo-tarpaulin）。

#### 所有覆盖率
```bash
just coverage-all
```
生成 Rust 和 Python 的覆盖率报告。

## 多 Python 版本测试

### 测试单个 Python 版本

#### Python 3.7
```bash
just test-py37
```

#### Python 3.8
```bash
just test-py38
```

#### Python 3.9
```bash
just test-py39
```

#### Python 3.10
```bash
just test-py310
```

#### Python 3.11
```bash
just test-py311
```

#### Python 3.12
```bash
just test-py312
```

### 测试所有支持的 Python 版本
```bash
just test-all-python
```

这将依次运行 Python 3.7 到 3.12 的所有测试。

**预期结果**: 所有版本都应该通过 39 个测试。

## CI/CD 测试

### 快速 CI 测试
在 PR 上自动运行：
- Python 3.8, 3.10, 3.12 的快速测试
- 代码质量检查（Linting、格式化）

### 多版本 CI 测试
在每个 PR 上运行：
- Python 3.7 - 3.12 的完整测试
- 验证 ABI3 兼容性

### 完整 CI 测试
在 main 分支上运行：
- 所有操作系统（Ubuntu、Windows、macOS）
- 所有 Python 版本（3.7 - 3.12）
- 覆盖率报告

## 开发工作流

### 1. 开始开发
```bash
just dev
```
设置开发环境（安装依赖并构建）。

### 2. 进行更改
编辑代码...

### 3. 运行检查
```bash
just check
```
运行格式化、Linting 和测试。

### 4. 提交更改
```bash
git add .
git commit -m "feat: your feature"
git push
```

### 5. 监控 CI
在 GitHub 上查看 CI 结果。

## 故障排除

### 构建失败
```bash
just clean
just build
```

### 测试失败
```bash
just test-fast
```
运行快速测试以获得更快的反馈。

### 特定 Python 版本问题
```bash
just test-py37  # 测试 Python 3.7
```

### 清理所有构建
```bash
just clean
```

## 项目信息
```bash
just info
```
显示项目信息（Rust、Python、UV 版本）。

## 安全审计
```bash
just audit
```
运行 Cargo 安全审计。

## 文档
```bash
just docs
```
生成并打开 Rust 文档。

## 发布

### 构建发布版本
```bash
just build-release
```

### 构建发布轮子
```bash
just release
```
在 `target/wheels/` 中生成发布轮子。

## 常见任务

| 任务 | 命令 |
|------|------|
| 快速测试 | `just test-fast` |
| 完整测试 | `just test` |
| 所有 Python 版本 | `just test-all-python` |
| 代码格式化 | `just format` |
| 代码检查 | `just lint` |
| 自动修复 | `just fix` |
| 覆盖率报告 | `just coverage-all` |
| 清理构建 | `just clean` |
| 发布构建 | `just release` |

## 更多信息

查看 `justfile` 了解所有可用命令。

```bash
just --list
```

