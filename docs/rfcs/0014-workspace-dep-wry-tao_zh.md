# RFC 0014: 集中管理 `wry` / `tao` 到 `[workspace.dependencies]`

- 编号: 0014
- 标题: `wry` / `tao` 通过 `[workspace.dependencies]` 集中版本
- 状态: Draft
- 创建日期: 2026-05-21
- 作者: AuroraView Core Team
- 拆分自: RFC 0013 §4.1.5 D9 修订
- 关联文件:
  - `Cargo.toml`（workspace root，新增 `[workspace.dependencies]`）
  - `crates/auroraview-core/Cargo.toml`
  - `crates/auroraview-desktop/Cargo.toml`
  - `crates/auroraview-browser/Cargo.toml`
  - `crates/auroraview-cli/Cargo.toml`

---

## 1. 摘要

把 `wry` / `tao` 的版本号从各 crate 的 `[dependencies]` 里抽到 workspace 根 `Cargo.toml` 的 `[workspace.dependencies]`，所有相关 crate 改用 `{ workspace = true }` 引用，消除版本飘移风险。

**不**做 `pub use wry`，避免把上游 crate 类型钉进 `auroraview-core` 的公共 API 表面。

---

## 2. 动机

当前仓库根 `Cargo.toml` 直接写：

```toml
[dependencies]
wry = "0.54.4"
tao = "0.34.6"
```

各 crate（`auroraview-core` / `auroraview-desktop` / `auroraview-browser` / `auroraview-cli` / 根 crate）各自独立写版本号，存在以下风险：

1. **版本飘移**：升级 wry 时漏改某个 crate，编译期会同时拉两个版本，符号链接器层面冲突或运行期出现未定义行为。
2. **跨 crate 类型不兼容**：`auroraview-core` 暴露的 `wry::WebViewBuilder` 类型与 `auroraview-cli` 引用的 `wry::WebViewBuilder` 来自不同版本，调用 `auroraview_core::attach_drag_drop_handler` 时会编译期类型不匹配。
3. **CI 噪声**：每次 wry 升级 PR 要改 5 个 `Cargo.toml`，PR 描述冗长。

后续 RFC 0015 引入的 `attach_drag_drop_handler` helper 在签名中暴露 `wry::WebViewBuilder<'a>`，要求所有调用方使用同一 wry 版本——本 RFC 是 0015 的硬前置依赖。

---

## 3. 设计

### 3.1 workspace 根 `Cargo.toml`

新增 `[workspace.dependencies]` 段：

```toml
[workspace.dependencies]
wry = "0.54.4"
tao = "0.34.6"
# 后续视需要把其它 webview 相关公共依赖也并入此处
```

根 `Cargo.toml` 自身的 `[dependencies]` 段把：

```toml
wry = "0.54.4"
tao = "0.34.6"
```

改为：

```toml
wry = { workspace = true }
tao = { workspace = true }
```

### 3.2 各 crate `Cargo.toml`

`auroraview-core` / `auroraview-desktop` / `auroraview-browser` / `auroraview-cli` 四个 crate 各自的 `Cargo.toml`，把对 `wry` / `tao` 的版本号引用全部改为 `{ workspace = true }`。每个 crate 改 1~2 行，无源码变更。

### 3.3 不做 `pub use wry`

`auroraview-core` 的对外公共 API（如未来 RFC 0015 的 `DragDropIpcSink` trait、`attach_drag_drop_handler` 函数）签名中**会**出现 `wry::WebViewBuilder` 类型——这是无法回避的，因为 helper 就是要操作 wry builder。

但与"`pub use wry` 把整个 wry 模块 re-export"是两件事：

- **当前方案**：`auroraview-core` 在签名中**引用** wry 类型（无法回避），但**不主动 re-export**；下游各 crate 通过 workspace dep 直接 `use wry`。wry 升级 minor/patch 时，只要 workspace dep 同步升级，整个 workspace 重新编译即通过。
- **如果 wry 出 breaking 版本**（如 `0.54 → 0.55` 改 `WebViewBuilder` 签名），`auroraview-core` 的 helper 签名也会受影响，但通过 workspace dep 升级一次即可同步全 workspace —— 这与 `pub use wry` 在效果上等价，但避免了"`auroraview-core` 主动暴露上游 crate"的设计承诺，未来想换 `wry` → 其它 webview crate 时改名空间更大。

---

## 4. 实施步骤

1. **Step 1**：根 `Cargo.toml` 新增 `[workspace.dependencies]` 段，写入 `wry` / `tao` 版本。
2. **Step 2**：根 `Cargo.toml` 自身 `[dependencies]` + 4 个 crate 的 `Cargo.toml` 全部改为 `{ workspace = true }`。
3. **Step 3**：`vx just build` 验证编译通过；`vx just test` 验证无回归。

整体改动量 ~10 行，可作为单独 PR 提交。

---

## 5. 兼容性

- **完全无破坏性**：仅改 `Cargo.toml`，对外 API 与运行期行为零变化。
- **向后兼容**：升级后任何现有调用方代码不需调整。

---

## 6. 风险

| 风险 | 评估 | 对策 |
|---|---|---|
| 升级 wry 时仍需修改根 `[workspace.dependencies]` | 低 | 改一处比改 5 处更不易出错；CI 编译失败会立即兜底 |
| 未来 wry 引入新泛型参数（如 `WebViewBuilder<'a, T>`） | 低 | workspace dep 不能消除 helper 签名的同步成本，但能保证一处升级即全 workspace 同步生效 |

---

## 7. 后续依赖

- **RFC 0015** 在 `auroraview-core` 暴露 `attach_drag_drop_handler` helper（签名包含 `wry::WebViewBuilder<'a>`），需先合入本 RFC 才能保证跨 crate 编译期类型一致。
