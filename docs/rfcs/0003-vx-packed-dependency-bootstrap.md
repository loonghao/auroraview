# RFC 0003: Packed Dependency Bootstrap via vx

> **Status**: Draft
> **Author**: AuroraView Team
> **Created**: 2026-01-08
> **Updated**: 2026-01-08
> **Target Version**: v0.6.0

## 摘要

为 AuroraView 打包流程引入基于 vx 的统一依赖引导能力：在打包阶段可直接通过 URL 下载并内嵌第三方工具/依赖（如 vx 发行包），提供可配置的下载/解压与校验管线，并允许在各打包阶段通过 vx 执行 hook 命令（包含 curl 等下载指令）。打包产物在运行时由 `gallery/backend/dependency_installer.py` 复用 vx（含 uv 虚拟环境）完成依赖安装和启动，使示例与扩展可以灵活依赖 Python/Node/Go/Rust 生态而不污染宿主环境。

## 动机

- **第三方依赖灵活性**：示例和插件需要额外依赖（Python 包、Node 工具、Go/Rust CLI）；当前 pack 流程缺少可复用的下载/注入机制。
- **一致的跨语言运行时**：vx 同时覆盖 uv、Node、Go、Rust，可作为默认工具链入口，降低环境碎片化。
- **可配置的离线/缓存策略**：需要在 CI、DCC 离线环境下可靠打包，避免重复下载。
- **安全性**：URL 下载必须具备校验与白名单能力，避免供应链风险。

## 设计方案

### 1. 配置扩展（auroraview.pack.toml）

新增可选节，默认保持现状（向后兼容）：

```toml
[vx]
enabled = true                   # 开启 vx 作为统一工具入口（默认 true，可关闭）
runtime_url = "https://github.com/loonghao/vx/releases/download/vx-v0.6.10/vx-0.6.10-x86_64-pc-windows-msvc.zip"
runtime_checksum = "<sha256>"     # 可选，推荐必填
cache_dir = "./.pack-cache/vx"    # 本地缓存目录，可复用 CI 缓存
ensure = ["uv", "node@20", "go@1.22", "rust@stable"]

[[downloads]]
name = "vx-runtime"
url = "https://github.com/loonghao/vx/releases/download/vx-v0.6.10/vx-0.6.10-x86_64-pc-windows-msvc.zip"
checksum = "<sha256>"             # 可选：sha256/sha512，缺失则仅警告
strip_components = 1               # 解压时去除的路径层级，可选
extract = true                     # 是否解压；false 则按原样放入
stage = "before_collect"          # 下载阶段：before_collect | before_pack | after_pack
dest = "python/bin/vx"            # 相对 overlay 的放置路径
executable = ["vx.exe"]            # 标记为可执行（Windows 设置 +x 无效，仅记录）

# 可重复 [[downloads]]
[[downloads]]
name = "extra-assets"
url = "https://example.com/assets.zip"
checksum = "<sha256>"
extract = true
stage = "before_pack"
dest = "resources/assets"

[hooks]
use_vx = true                      # 现有 hooks.* 命令优先以 vx 运行

# 兼容旧 hooks，新增 vx 专属命令数组
[hooks.vx]
before_collect = [
  "vx --version",                 # 验证下载结果
  "vx uv pip install -r requirements.txt"
]
after_pack = [
  "vx curl -L https://example.com/posthook.txt -o $OUT_DIR/posthook.txt"
]
```

要点：
- `vx.enabled=false` 时回退到现有行为，忽略 vx 针对性的下载/执行。
- `downloads.stage` 允许将资源放入 overlay，不改变现有 `hooks.collect` 语义。
- `hooks.use_vx=true` 时，原有 `hooks.before_collect / after_pack` 里的命令将自动包装为 `vx <cmd>`，确保一致的运行时（如 `vx uv`, `vx npm`, `vx go`）。

### 2. 打包流程增强

1) **解析配置**：解析 `[vx]`、`[[downloads]]`、`hooks.vx` 与 `hooks.use_vx`。
2) **下载与缓存**：
   - 先查 `cache_dir`，命中则跳过下载。
   - 缺失时执行下载，校验 `checksum`（失败即报错，缺失则仅警告）。
   - 支持 `AURORAVIEW_OFFLINE=1`：仅使用缓存，缓存缺失则失败。
3) **解压与放置**：按 `stage` 在对应阶段解压/复制到 overlay（尊重 `strip_components`）。
4) **vx runtime 注入**：
   - 若 `vx.enabled`，确保 vx 可执行位于 overlay `python/bin/vx`（或平台路径），打包产物内可直接调用。
   - 将 `vx` 路径写入运行时环境（如 `AURORAVIEW_VX_PATH`）。
5) **Hook 执行**：
   - 新增 `hooks.vx.*` 在对应阶段执行，使用下载的 vx。
   - 旧 hooks 若 `use_vx=true`，命令前置 `vx` 调用；否则保持原样。
6) **日志与回执**：在打包日志中输出下载源、缓存命中、校验结果与 hook 执行结果，便于 CI 追踪。

### 3. 运行时与依赖安装（gallery/backend/dependency_installer.py）

- 运行时通过 `AURORAVIEW_VX_PATH` 找到打包内的 vx，可选回退到系统路径。
- 使用 `vx uv` 创建/管理隔离环境，安装 `auroraview.pack.toml` 中声明的 Python 依赖（沿用现有 `packages`/lockfile 逻辑）。
- 如配置了 Node/Go/Rust 工具，依赖安装脚本可调用 `vx npm`, `vx go install`, `vx cargo` 等完成预置。
- 支持离线优先：若 `AURORAVIEW_OFFLINE=1`，仅使用随包资源与缓存。

### 4. 安全性与合规

- 默认要求 HTTPS；若需 HTTP 必须在配置中显式 `allow_insecure = true`（可选字段）。
- 支持 `allowed_domains = ["github.com", "objects.githubusercontent.com", ...]` 白名单；不在白名单则警告或拒绝（按配置 `block_unknown_domains`）。
- 强烈推荐提供 `checksum`；缺失仅告警但不阻断（可通过 `require_checksum = true` 强化）。

### 5. Backward Compatibility

- 所有新增字段均为可选；未开启 vx 时行为与现有 pack 完全一致。
- 旧 hooks 无需修改；若未设置 `use_vx`，继续按原方式执行。
- 下载/解压管线仅在存在 `[[downloads]]` 时生效。

## 实现计划

### Phase 1：配置与下载管线（v0.6.0-alpha）
- [ ] TOML 解析：`[vx]`、`[[downloads]]`、`hooks.use_vx`、`hooks.vx.*`
- [ ] 下载与缓存组件（校验、离线模式、strip/extract、dest 放置）
- [ ] 打包阶段插入下载/解压流水线（before_collect / before_pack / after_pack）
- [ ] 打包日志输出与错误处理

### Phase 2：vx 集成与 hooks（v0.6.0-beta）
- [ ] 在打包产物中注入 vx runtime（按平台路径放置）并导出 `AURORAVIEW_VX_PATH`
- [ ] `hooks.use_vx` 自动包装旧 hooks；新增 `hooks.vx.*` 执行器
- [ ] 示例配置与文档更新（packing 指南追加示例）

### Phase 3：运行时依赖安装对接（v0.6.0）
- [ ] 更新 `gallery/backend/dependency_installer.py` 使用 `vx uv` / `vx npm` / `vx go`
- [ ] 离线/缓存策略在运行时生效（读取打包产物中的 vx 与下载物）
- [ ] 回归测试：Windows WebView2 + Qt 宿主下的依赖安装与示例启动

### Phase 4：增强与验收（v0.6.x）
- [ ] CI 覆盖：缓存命中/未命中、checksum 失败、离线模式
- [ ] 安全策略：白名单与 `require_checksum` 开关
- [ ] 文档与示例：提供最小配置与高级玩法（多语言工具链）

## 参考资料

- vx 项目发布页（用于 runtime_url 与校验）
- 现有打包文档：`docs/guide/packing.md`
- 运行时依赖安装脚本：`gallery/backend/dependency_installer.py`

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-01-08 | Draft | 初始草案 |