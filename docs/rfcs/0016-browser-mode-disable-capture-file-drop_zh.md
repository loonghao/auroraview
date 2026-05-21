# RFC 0016: Browser 模式禁用 `capture_file_drop`

- 编号: 0016
- 标题: 多 Tab `Browser` 模式硬禁用 `capture_file_drop`（永不挂载 `with_drag_drop_handler`）
- 状态: Draft
- 创建日期: 2026-05-21
- 作者: AuroraView Core Team
- 拆分自: RFC 0013 §4.3.4（D5 / D17 / D18 修订）
- 前置依赖: **RFC 0015**（提供 `attach_drag_drop_handler` helper）
- 关联文件:
  - `src/webview/tab_manager.rs::TabManager`（路径 3 / 4 业务 tab + controller）
  - `crates/auroraview-browser/src/tab/manager.rs`（路径 9 业务 tab）
  - `crates/auroraview-browser/src/browser.rs`（路径 10 controller）
  - `crates/auroraview-browser/src/config.rs::BrowserConfig`（**不**修改）
  - `crates/auroraview-cli/src/packed/webview/mod.rs`（packed 进入 Browser 模式时的入口）

---

## 1. 摘要

在多 Tab `Browser` 模式（新路径 `auroraview-browser::Browser` 与旧路径 `webview/tab_manager::TabManager`）下，**所有 webview 永不调用 `with_drag_drop_handler`**——业务 tab 与 controller 一视同仁。

实现方式：`BrowserConfig` / `TabManagerConfig` **不**新增 `capture_file_drop` 字段，调用 `attach_drag_drop_handler` 时**直接传 `capture=false` 常量**。不需要"运行期 warn + 清零"逻辑，避免 RFC 0013 v14 中 D5（不加字段）与 D17（读字段并清零）相互否定的设计漏洞。

---

## 2. 动机

### 2.1 为什么不能"业务 tab 挂、controller 不挂"

早期方案设想"controller 永不挂、业务 tab 共享 `BrowserConfig.capture_file_drop`"，看似自洽，但**与 wry/WebView2 的 `IDropTarget` 行为模型直接冲突**：

1. **业务 tab 与 controller 是两个独立 WebView2 实例**，在窗口层面叠在一起（controller 渲染 tab bar / 地址栏 / 菜单条等 UI 外壳，业务 tab 在外壳内的"内容区"占位）。
2. WebView2 的 `IDropTarget` 接管是**逐 WebView 决定**的——controller 不挂 → controller 区域内 OS 拖放回退浏览器原生 HTML5；业务 tab 挂 → tab 区域内 OS 拖放被吃掉变 IPC。**两套语义在像素级共存**。
3. 用户从屏幕拖文件**穿越 controller / tab 边界**时（最常见场景：从桌面拖文件、鼠标先掠过 tab bar 再落到 tab 内容区），鼠标 hit-test 落在哪一个 WebView 上完全取决于像素级布局：
   - 鼠标在 tab bar 上方时：触发 controller 的 HTML5 `dragover`（用于 tab 重排手势检测）；
   - 鼠标移入 tab 内容区时：**突然不触发 HTML5 事件，转而触发业务 tab 的 IPC `file_drop_hover`**；
   - 用户松手位置又落在边缘：可能两边都不触发 / 先触发 controller HTML5 `drop` 后业务 tab IPC `file_drop`——**状态机无法收敛**。
4. 前端**无法**写出连贯的 hover 反馈，因为事件流是两个独立 webview 在冒泡两套不兼容的协议。
5. 即便业务页面只关心"放下后拿到路径"，OS 层在 hover 阶段（鼠标尚在 controller）已经把 drag image 显示在 controller 的 HTML5 `dragover` 反馈里，用户体验与单 webview 截然不同。

> **不是 RFC 实现层就能解决的边界 bug**：这是 wry/WebView2 在"多 webview 叠加"模型上的根本约束；要让"业务 tab 拿 IPC + controller 保 HTML5"真正可行，需要 wry 暴露一个"父子 webview 拖放协议合并"的 API，目前不存在。

### 2.2 用户应走的替代路径

需要"拖文件拿绝对路径"的页面有两条路：

- **方案 A（HTML5）**：在 web 内容侧使用浏览器原生 HTML5 拖放（`dragover` + `drop` + `DataTransfer`）。注意 `DataTransfer.files` 在 web 平台不暴露绝对路径，只暴露文件名 + 字节流——这是 web 安全约束。
- **方案 B（推荐）**：把需要"绝对路径 IPC"的页面改为顶层 `AuroraView` 实例（独立窗口、单 webview），在该实例上设 `capture_file_drop=True`。

---

## 3. 设计

### 3.1 `BrowserConfig` / `TabManagerConfig` 不新增字段

**`crates/auroraview-browser/src/config.rs::BrowserConfig`**：不新增 `capture_file_drop` 字段。

**`crates/auroraview-browser/src/config.rs::BrowserConfigBuilder`**：不新增 `capture_file_drop` 链式方法。

**`src/webview/tab_manager.rs::TabManagerConfig`**：不新增 `capture_file_drop` 字段。

**理由**：空字段只会让用户误以为"配置生效了"，反而比"字段不存在 → 编译期报错"更糟糕。当前 0.x 阶段直接靠 Rust 类型系统拒绝这种误用；未来 wry 上游修复后再独立 RFC 引入字段。

### 3.2 4 处 builder 调用点直接传 `capture=false` 常量

| # | 文件 | 改动 |
|---|---|---|
| 3 | `src/webview/tab_manager.rs::create_tab_webview`（业务 tab） | `builder = attach_drag_drop_handler(builder, false, &ipc_handler);` 直接常量 false |
| 4 | `src/webview/tab_manager.rs:984`（controller） | 同上，常量 false |
| 9 | `crates/auroraview-browser/src/tab/manager.rs:122`（业务 tab） | 同上 |
| 10 | `crates/auroraview-browser/src/browser.rs:545`（controller） | 同上 |

每处 builder 调用现场加一句注释：

```rust
// Browser 模式（含 controller 与所有业务 tab）永不挂 with_drag_drop_handler。
// 多 webview 叠加场景下挂载会导致 OS 拖放跨边界状态机无法收敛（详见 RFC 0016 §2.1）。
// 需要"拖文件拿绝对路径"的页面应改为顶层 AuroraView 实例 + capture_file_drop=True。
builder = attach_drag_drop_handler(builder, false, &ipc_handler);
```

### 3.3 `Browser::new` / `TabManager::new` 入口不读 cfg、无 warn 逻辑

由于 `BrowserConfig` 不含 `capture_file_drop` 字段，`Browser::new` / `TabManager::new` 入口**不需要**任何检查、清零、warn。

> **与 RFC 0013 v14 D17 的差异**：v14 D17 修订设想"`Browser::new(cfg) → mut effective_cfg = cfg; if effective_cfg.tab_webview_config.capture_file_drop { warn!; effective_cfg.tab_webview_config.capture_file_drop = false; }`"，但 v14 D5 同时声称 `BrowserConfig` 不加该字段——D17 的代码示例引用了一个 D5 不允许存在的字段，自相矛盾。本 RFC 取消 D17 的运行期检查路径，回到"D5 唯一真相"。

### 3.4 packed runtime 不需要 mode 分流

由于 §3.3 已删除"运行期 warn"，原 v14 §4.2.4.3 D18 修订引入的 `PackedRuntimeMode::TopLevelAuroraView` / `Browser` 枚举与 `resolve_packed_capture_file_drop_with_mode` 函数也**一并删除**。

`AURORAVIEW_CAPTURE_FILE_DROP` env var 在 packed runtime 解析层**不再 mode 分流**。Browser 模式下 packed runtime 本就**不读** `capture_file_drop` 字段（4 处 builder 调用点都是常量 false），env var 设了等于"对一个不被读取的字段做了运行时覆盖"——不会产生任何矛盾日志，因为根本没有第二条 warn。

> 终端用户如果在 Browser 模式下设了 `AURORAVIEW_CAPTURE_FILE_DROP=1` 期望开启 IPC 代理，**确实不会生效**且**确实没有提示**。这是 trade-off：在 Browser 模式下提供 env var 提示需要回到 v14 D18 的 mode 枚举路径，工程量与 RFC 0015 §3.6 的 child window 文档警示等价；本 RFC 选择"文档说明 + 用户主动 grep"路径，与 RFC 0015 §5.1 故障排查段统一。

### 3.5 与 RFC 0015 IPC 作用域限定段的关系

RFC 0015 §5 末尾"作用域限定"段已声明：

> Browser 内部业务 tab + controller 都不挂 handler（详见 RFC 0016）；
> 同一进程内多个独立 `AuroraView` 实例之间，`file_drop*` 按各自 IPC 通路独立分发。

本 RFC 与 RFC 0015 §5 互相呼应，前端订阅时假设"事件来自当前 webview 自身"即可，不要假设跨 webview 冒泡。

---

## 4. 测试方案

### 4.1 Rust 测试

`crates/auroraview-browser/tests/browser_drag_drop_isolation_tests.rs` 新增：

- **`browser_never_attaches_drag_drop_handler`**：通过 §5 CI grep 守住"`browser.rs` / `tab/manager.rs` 中所有 `attach_drag_drop_handler` 调用第 2 个参数都是字面量 `false`"。
- **`browser_config_does_not_expose_capture_file_drop`**：编译期断言 `BrowserConfig` 不含 `capture_file_drop` 字段：

  ```rust
  // 通过尝试构造一个含 capture_file_drop 字段的字面量来反向证明字段不存在
  // （编译失败说明字段不存在，符合 RFC 0016 §3.1）
  // 这条测试用 #[cfg(test_compile_fail)] 或 trybuild crate 守住
  ```

  > 简化版：直接由 §5 CI grep 守住"`config.rs` 中 `BrowserConfig` 结构体定义不含 `capture_file_drop`"。

`src/webview/tab_manager.rs` 的对应测试镜像。

### 4.2 手工冒烟矩阵

| 模式 | `capture=false`（默认）| `capture=true`（任何方式尝试设置）|
|---|---|---|
| Multi-tab 业务 tab | HTML5 `drop` 可用 / IPC 不触发 | **HTML5 `drop` 仍可用 / IPC 仍不触发**（`BrowserConfig` 编译期就不接受字段）|
| Multi-tab controller | HTML5 `drop` 可用（永远）| 同左 |
| Packed Browser 模式 + `AURORAVIEW_CAPTURE_FILE_DROP=1` | IPC 不触发（env var 对 Browser 模式无效）| 同左 |

### 4.3 文档验证

`docs/zh/guide/file-drop.md` 与 `docs/zh/guide/multi-tab.md`（如存在）需在 multi-tab 章节明确：

> Browser 模式下 `capture_file_drop` 不可用。需要通过 IPC 拿到拖入文件的绝对路径，请使用顶层 `AuroraView` 实例（独立窗口、单 webview）。

---

## 5. CI 防回归 grep

`scripts/ci/check_browser_no_drag_drop_capture.py`（或 `just check-browser-no-capture`）：

```bash
# 禁止 BrowserConfig / TabManagerConfig 中出现 capture_file_drop 字段
if rg "capture_file_drop" crates/auroraview-browser/src/config.rs ; then
    echo "ERROR: BrowserConfig must not expose capture_file_drop (RFC 0016 §3.1)"
    exit 1
fi
if rg "capture_file_drop" src/webview/tab_manager.rs ; then
    echo "ERROR: TabManagerConfig must not expose capture_file_drop (RFC 0016 §3.1)"
    exit 1
fi

# 禁止 Browser / TabManager 路径下 attach_drag_drop_handler 第 2 个参数为非 false 字面量
# （如果调用了 helper，必须是常量 false；动态值意味着引入了 RFC 0016 不允许的配置入口）
if rg "attach_drag_drop_handler\([^,]+,\s*[^f]" crates/auroraview-browser/src/ src/webview/tab_manager.rs ; then
    echo "ERROR: Browser/TabManager paths must always pass capture=false to attach_drag_drop_handler (RFC 0016 §3.2)"
    exit 1
fi

echo "OK: Browser mode never attaches drag-drop handler."
```

---

## 6. 实施步骤

1. **Step 1 — Browser controller**：`crates/auroraview-browser/src/browser.rs:545` controller builder 调用点加 `attach_drag_drop_handler(builder, false, &ipc_handler)` + 注释。
2. **Step 2 — Browser 业务 tab**：`crates/auroraview-browser/src/tab/manager.rs:122` 业务 tab builder 调用点同上。
3. **Step 3 — 旧路径 tab_manager**：`src/webview/tab_manager.rs` 两处（业务 tab :469 + controller :984）镜像处理。
4. **Step 4 — CI grep**：新增 `scripts/ci/check_browser_no_drag_drop_capture.py`，接入 `vx just test` 流程。
5. **Step 5 — 文档**：`docs/zh/guide/file-drop.md` / multi-tab 章节明确"Browser 模式不支持 `capture_file_drop`，请使用顶层 AuroraView 实例"的迁移路径；CHANGELOG 在 multi-tab 段标注。

每步通过 `vx just test` 验证。

---

## 7. 兼容性

- **完全无 API 破坏**：`BrowserConfig` / `TabManagerConfig` / `BrowserConfigBuilder` 公共表面没有任何变化。
- **运行期行为**：
  - 之前 Browser 模式下所有 webview **均未挂** `with_drag_drop_handler`（RFC 0013 v14 §2.1 路径表已确认）；本 RFC 实施后行为不变（仍不挂）。
  - 唯一变化是 4 处 builder 现在统一通过 `attach_drag_drop_handler(builder, false, ...)` 调用，代码风格与其它 5 处 builder 对齐。
- **零迁移成本**。

---

## 8. 风险

| 风险 | 评估 | 对策 |
|---|---|---|
| 用户在 Browser 模式下想用 IPC 拖放 | 低 | 文档明确指引到"顶层 `AuroraView` 实例"；CI grep + 编译期字段不存在双重防御 |
| 未来 wry 修复多 webview 拖放协议 | 低 | 届时独立 RFC 重新引入 `BrowserConfig.capture_file_drop`，本 RFC 的"4 处常量 false"代码反向变成"读取并应用配置" |
| Packed Browser 模式下 env var 无提示 | 低 | RFC 0015 §5.1 故障排查文档统一覆盖 |

---

## 9. 后续 RFC

- 上游 wry 暴露"父子 webview 拖放协议合并"或"全窗口统一 `IDropTarget`" API 后，可独立 RFC 重新引入 `BrowserConfig.capture_file_drop` + per-tab 覆盖。
