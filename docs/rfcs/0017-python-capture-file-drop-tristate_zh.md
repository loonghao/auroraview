# RFC 0017: Python `capture_file_drop` 三态契约（`Optional[bool]`）

- 编号: 0017
- 标题: Python 透传链 `capture_file_drop` 保留 `Optional[bool]` 三态 + CI grep 防回归
- 状态: Draft
- 创建日期: 2026-05-21
- 作者: AuroraView Core Team
- 拆分自: RFC 0013 §4.2.5 / §4.5.1 关键点 1（D3 / D12 修订）
- 前置依赖: **RFC 0015**（提供 Rust `WebViewConfig.capture_file_drop: bool` 字段）
- 关联文件:
  - `python/auroraview/core/config.py::ContentConfig`
  - `python/auroraview/core/config.py::WebViewConfig.from_kwargs`
  - `python/auroraview/core/config.py::WebViewConfig.to_kwargs`
  - `python/auroraview/core/factory.py`
  - `python/auroraview/core/mixins/factory.py`
  - `python/auroraview/core/mixins/content.py`
  - `python/auroraview/integration/qt/_core.py`
  - `python/auroraview/__main__.py`（CLI 入口透传）
  - `src/bindings/desktop_runner.rs`（PyO3 binding 层 `unwrap_or(false)` 兜底）
  - `scripts/ci/check_capture_file_drop_defaults.py`（新增 CI grep）
  - `tests/python/unit/test_file_drop_events.py`
  - `tests/python/integration/test_capture_file_drop_passthrough.py`（新增）
  - `tests/python/unit/test_child_window_isolation.py`（新增）

---

## 1. 摘要

Python 侧 `capture_file_drop` 使用 `Optional[bool]` 三态：

| 取值 | 含义 |
|---|---|
| `None`（默认）| 未指定，让下层（Rust `WebViewConfig`）默认值兜底 |
| `True` | 显式启用 IPC 代理 |
| `False` | 显式禁用 IPC 代理 |

透传链上**任何**中间层都**不得**用 `setdefault("capture_file_drop", False)` / `kwargs.get("capture_file_drop", False)` / `value or False` 等方式把 `None` 蹋平为 `False`。三态必须**原样穿透**到 Rust PyO3 binding 层，由 Rust 侧 `unwrap_or(false)` 兜底。

---

## 2. 动机

### 2.1 三态契约的真实工程价值

1. **防中间层 silent override**：透传链上任何一层蹋平都会让真实意图丢失。如果未来 PyO3 入口接入新的下层来源（如 `[user_settings]` toml、远程配置），`None` 可以自然回退到下层；如果中间层早已蹋平，新引入的下层来源永远无法生效。
2. **跨入口语义对齐**：`Option<bool>` 在 manifest（RFC 0015 §4.1）和 Pack flag（RFC 0015 §4.2）已是统一语言；Python kwarg 沿用同一三态。
3. **测试可观测**：三态让"用户没传"和"用户传了 False"在集成测试中可独立断言。

### 2.2 当前架构下 Rust `unwrap_or(false)` 不会屏蔽下层

PyO3 入口与 packed app 运行时**不在同一进程**：
- `auroraview pack` 产物 standalone exe 走 Rust 入口，**不经过** PyO3 binding；
- `PackMode::FullStack` 下 Python 是子进程，通过 IPC 与 packed runtime 通信。

因此 PyO3 binding 的 `unwrap_or(false)` 兜底**不会**屏蔽 packed overlay 的值；三态保留的价值在防 silent override + 为未来扩展留余地，**不**在"覆盖 manifest"这种当前不存在的场景。

---

## 3. 设计

### 3.1 字段归属：`ContentConfig`

`capture_file_drop` 放入 `ContentConfig`，与既有 `allow_file_protocol` 同组。

**理由**：

1. **语义同源**：`allow_file_protocol` 与 `capture_file_drop` 都属于"宿主主动放权给 web 内容访问本地文件路径"的能力门。
2. **`WindowConfig` 边界明确**：现有字段都是窗口的视觉/几何/呈现属性，没有"web 内容能力"或"事件路由"类字段。
3. **顶层不放置**：顶层目前都是"跨多个子组共享"的全局属性；`capture_file_drop` 有清晰归属。

### 3.2 `ContentConfig` 字段定义

```python
@dataclass
class ContentConfig:
    """Initial content configuration and content-side capability gates.

    Attributes:
        url: URL to load (optional)
        html: HTML content to load (optional)
        asset_root: Root directory for auroraview:// protocol (optional)
        allow_file_protocol: Enable file:// protocol (default: False, security risk)
        capture_file_drop: Forward OS file drops as IPC ``file_drop`` events.
            Tri-state with ``Optional[bool]``:

            - ``None`` (default) — inherit lower-layer default (currently ``False``).
            - ``True`` — force enable; HTML5 ``dragover``/``drop`` inside the
              WebView become inert (upstream wry/WebView2 limitation, see RFC 0015 §2).
            - ``False`` — force disable.

            Note: ``capture_file_drop`` is ignored in multi-tab Browser mode
            (RFC 0016). For absolute file paths via IPC, use a top-level
            ``AuroraView`` instance.
    """

    url: Optional[str] = None
    html: Optional[str] = None
    asset_root: Optional[str] = None
    allow_file_protocol: bool = False
    capture_file_drop: Optional[bool] = None
```

### 3.3 `from_kwargs` / `to_kwargs` 透传规则

`from_kwargs` 中 `ContentConfig` 构造改用**无默认值**的 `kwargs.get(key)`：

```python
content = ContentConfig(
    url=kwargs.get("url"),
    html=kwargs.get("html"),
    asset_root=kwargs.get("asset_root"),
    allow_file_protocol=kwargs.get("allow_file_protocol", False),
    # 关键：无默认值。`None` / `True` / `False` 三态必须保留到 Rust 侧
    capture_file_drop=kwargs.get("capture_file_drop"),
)
```

`to_kwargs` 同步增加透传，**绝对不要**写 `or False` / `bool(...)` 把 `None` 蹋平。**注释必须显式标注语义锚点**：

```python
# RFC 0017 三态契约：capture_file_drop 必须保持 Optional[bool] 一路穿透到
# Rust PyO3 binding（src/bindings/desktop_runner.rs），由 Rust 侧 unwrap_or(false)
# 兜底。在此处或下游胶水代码中加默认值（or False / setdefault / bool(...)）
# 都会让"未传 vs 显式 False"两态合并，破坏三态语义。
"capture_file_drop": self.content.capture_file_drop,   # Optional[bool]，原样传给 Rust
```

### 3.4 各文件改动清单

| 文件 | 改动 |
|---|---|
| `python/auroraview/core/config.py` | `ContentConfig` 新增字段 + `from_kwargs` 改用 `kwargs.get("capture_file_drop")`（无默认值）+ `to_kwargs` 在 `# Content` 段追加透传 + 注释锚点 |
| `python/auroraview/core/factory.py` | 透传到 Rust binding 层时**不**蹋平 `None` |
| `python/auroraview/core/mixins/factory.py` | 不引入 `setdefault`；`**kwargs` 中 `capture_file_drop` 原样透传 |
| `python/auroraview/core/mixins/content.py` | 若涉及 `ContentConfig` 构造则同步透传（与 `allow_file_protocol` 一致） |
| `python/auroraview/__main__.py` | CLI 入口 `--capture-file-drop` 透传到底层（仅 `True` / 未传 两种状态） |
| `python/auroraview/integration/qt/_core.py` | DCC（Qt）路径**不**特殊处理，沿用 `ContentConfig.capture_file_drop = None` 默认 |
| `src/bindings/desktop_runner.rs` | PyO3 binding 字段映射：从 Python 接收 `Option<bool>`，到 Rust `WebViewConfig.capture_file_drop` 时 `unwrap_or(false)`；Rust 侧 `WebViewConfig` 自身保持 `bool`（最底层 source of truth） |

> **不在透传链上的 binding**：`src/bindings/webview2.rs` 暴露的是基于 HWND 的最小 handle API，**完全不接受** `WebViewConfig`，**无需**透传。

### 3.5 DCC 调用链字段流向

| # | 层 | 类型 / 函数 | 字段 | 类型 |
|---|---|---|---|---|
| 1 | Python 用户代码 | `AuroraView(parent=..., capture_file_drop=True)` | kwarg | `Optional[bool]` |
| 2 | Python 基类 | `core/config.py::ContentConfig` | `capture_file_drop: Optional[bool]` | 三态 |
| 3 | Python 工厂 | `core/factory.py::create_webview` | 透传 dict | 三态 |
| 4 | Python mixin | `core/mixins/factory.py` / `mixins/content.py` | 透传，无 `setdefault` | 三态 |
| 5 | DCC Qt 层 | `integration/qt/_core.py` | **直接透传，不特殊处理** | 三态 |
| 6 | PyO3 binding | `src/bindings/desktop_runner.rs` | 接收 `Option<bool>`，落地时 `unwrap_or(false)` | 三态 → `bool` |
| 7 | Rust core | `src/webview/config.rs::WebViewConfig` | `capture_file_drop: bool` | `bool`（最底层）|
| 8 | Rust backend | `src/webview/backend/native.rs::NativeBackend::create_webview` | 读取并传给 `attach_drag_drop_handler` | `bool` |

---

## 4. 测试方案

### 4.1 Python 单元测试

`tests/python/unit/test_file_drop_events.py` 新增：

- `test_capture_file_drop_default_none` — 构造 `AuroraView()` 不传 kwarg，断言 `cfg.content.capture_file_drop is None`（验证三态：未传 = `None`，**不是** `False`）。
- `test_capture_file_drop_explicit_true` — 显式 `True` → `cfg.content.capture_file_drop is True`。
- `test_capture_file_drop_explicit_false` — 显式 `False` → `cfg.content.capture_file_drop is False`（守住"显式 False 不被蹋平为 None"）。

### 4.2 Python 集成测试

`tests/python/integration/test_capture_file_drop_passthrough.py`（新增）覆盖调用链 1→8 的三态穿透：

- `test_passthrough_explicit_true`：构造 `AuroraView(capture_file_drop=True)`，通过 PyO3 测试钩子 `_dump_config()` 断言 Rust `WebViewConfig.capture_file_drop` 收到 `true`。
- `test_passthrough_explicit_false`：构造 `AuroraView(capture_file_drop=False)`，断言 Rust 侧收到 `false`（验证显式 `False` 没被中间层 `setdefault` 蹋平）。
- `test_passthrough_omitted_falls_to_default_false`（**核心防御**）：构造 `AuroraView()`（不传 kwarg），断言：
  - 中间层 dataclass：`cfg.content.capture_file_drop is None`（穿透到 Rust 之前都保持 `None`）；
  - 最终 Rust 侧：`_dump_config().capture_file_drop is False`（来自 Rust `unwrap_or(false)` 兜底）。

### 4.3 Child window 隔离测试

`tests/python/unit/test_child_window_isolation.py`（新增）守住 RFC 0015 §3.6 的 D11 修订成果——"用户构造 `AuroraView` 时**不应**因 `capture_file_drop=True` + `new_window_mode="child_webview"` 组合被错误拦截"：

- `test_main_window_capture_file_drop_with_child_webview_mode_is_legal`：构造 `AuroraView(capture_file_drop=True, new_window_mode="child_webview")`，断言**构造完成不抛任何错**（这是合法配置：主窗 IPC 正常工作，child window 自身不挂 handler 是预期行为）。
- `test_main_window_capture_file_drop_false_with_child_webview_mode_is_legal`：同上但 `capture_file_drop=False`。
- `test_main_window_capture_file_drop_omitted_with_child_webview_mode_is_legal`：同上但不传（覆盖 `None` 分支）。

> **作用**：防止任何人未来再次往 Python 层加"看到 `child_webview` 就拒绝 `capture_file_drop`"这种过度拦截。本文件**不验证** child window 内部代码事实（Rust 侧由 RFC 0015 §6.1 + §5 CI grep 守住）。

---

## 5. CI 防回归 grep

`scripts/ci/check_capture_file_drop_defaults.py`（接入 `vx just test`）：

```bash
#!/usr/bin/env bash
set -euo pipefail

# 禁止的模式 1：在 Python 透传链上对 capture_file_drop 使用 setdefault
if rg --type py "setdefault\(.{0,20}['\"]capture_file_drop['\"]" python/auroraview/ ; then
    echo "ERROR: capture_file_drop must be passed through as Optional[bool] (RFC 0017 §3.3)"
    exit 1
fi

# 禁止的模式 2：在 Python 透传链上把 None 蹋平成 False
if rg --type py "(get|pop)\(['\"]capture_file_drop['\"],\s*(True|False)" python/auroraview/ ; then
    echo "ERROR: do not provide a default for capture_file_drop in Python passthrough; \
           the field must remain Optional[bool] until it reaches Rust unwrap_or (RFC 0017 §3.3)"
    exit 1
fi

# 禁止的模式 3：or False / or True 蹋平
if rg --type py "capture_file_drop\s+or\s+(True|False)" python/auroraview/ ; then
    echo "ERROR: do not flatten Optional[bool] capture_file_drop with 'or' in Python (RFC 0017 §3.3)"
    exit 1
fi

echo "OK: capture_file_drop passthrough rules satisfied."
```

唯一允许的例外是 `src/bindings/desktop_runner.rs` 中 PyO3 binding 层的 `unwrap_or(false)` 兜底（位于 Rust，grep 不会命中）。

> **不用 lint 插件**：项目内 grep 已是足够的"绊索"，跨语言 lint 配置反而会引入新依赖；这条脚本和 §4.2 的 `test_passthrough_omitted_falls_to_default_false` 形成"静态 + 运行期"双重保护。

---

## 6. 实施步骤

1. **Step 1 — `ContentConfig` 字段**：`python/auroraview/core/config.py` 新增 `capture_file_drop: Optional[bool] = None` + docstring。
2. **Step 2 — `from_kwargs` / `to_kwargs`**：改用 `kwargs.get("capture_file_drop")` 无默认值 + `to_kwargs` 透传 + 注释锚点。
3. **Step 3 — 透传链各层**：`factory.py` / `mixins/factory.py` / `mixins/content.py` / `integration/qt/_core.py` / `__main__.py` 同步透传，禁止 `setdefault` / `or False`。
4. **Step 4 — PyO3 binding**：`src/bindings/desktop_runner.rs` 接收 `Option<bool>`、`unwrap_or(false)` 落地 Rust `WebViewConfig.capture_file_drop`。
5. **Step 5 — CI grep**：新增 `scripts/ci/check_capture_file_drop_defaults.py`，接入 `vx just test`。
6. **Step 6 — 测试**：§4 三类测试全部新增；通过 PyO3 expose `_dump_config` 测试钩子（仅 `cfg(test)` 启用）让集成测试可观察 Rust 侧值。
7. **Step 7 — DCC 迁移指南**：CHANGELOG / docs/zh/guide 显著标注：

   > 之前在 Maya / Houdini / Nuke 等 DCC 宿主中使用 AuroraView 时，文件拖放会自动转发为 `file_drop` IPC 事件。本版本起 DCC 路径默认行为改为「不接管」，与 standalone 等其他模式保持一致。
   >
   > 如果你的 DCC 工具依赖 `file_drop` 事件，请在构造 `AuroraView` 时显式传入 `capture_file_drop=True`：
   >
   > ```python
   > from auroraview import AuroraView
   >
   > class MyDccTool(AuroraView):
   >     def __init__(self, parent=None):
   >         super().__init__(parent=parent, capture_file_drop=True)
   > ```

每步通过 `vx just test` 验证。

---

## 7. 兼容性

- **Python API**：`AuroraView(...)` / `ContentConfig(...)` 新增 kwarg，默认 `None` → 现有调用方未传 kwarg 时行为与新版 Rust 默认 `false` 一致——**对 standalone / CLI / packed 用户无破坏**。
- **DCC 用户行为变更**：之前 DCC 路径默认 `True` 自动接管拖放，新版本默认 `None → false`。**这是 RFC 0015 §8.1 标注的 breaking 的直接表现**——DCC 用户需在构造时显式传 `capture_file_drop=True`（一行代码）。
- **Python kwarg `None` 与 `False`**：用户显式传 `False` 与不传 kwarg 在当前架构下 Rust 侧最终值都是 `false`，但中间层数据结构上有真实差异（用于未来扩展），由三态契约 + CI grep 双重守住。

---

## 8. 风险

| 风险 | 评估 | 对策 |
|---|---|---|
| 中间层默认值覆盖用户传入值（silent override） | 中 | 三重保护：§3.3 透传规则禁止 `setdefault` / `or False`、§4.2 集成测试 `test_passthrough_omitted_falls_to_default_false`、§5 CI grep |
| 用户绕过 `from_kwargs` 直接构造 `WebViewConfig` 后 `to_kwargs` 时蹋平 | 中 | §3.3 `to_kwargs` 内部就近注释锚点；用户胶水代码出错时有就近线索 |
| 用户对 `None` vs `False` 区分困惑 | 低 | docstring + DCC 迁移指南明确"未传 = 用代码默认"、"显式 `False` = 强制关"；当前架构下两者最终行为相同，差异只在未来扩展时显现 |

---

## 9. 后续 RFC

- 若未来 PyO3 入口接入新的下层来源（`[user_settings]` toml / 远程配置 / 热加载），`None` 自然回退到下层的语义就生效，三态契约保留的价值此时显现，**无需** RFC 修订。
- 若未来引入"进程内嵌入式 Python 解释器与 packed runtime 合并"的架构（目前没有规划），届时再单独走 RFC 处理 `unwrap_or(false)` 与 packed overlay 的优先级合并。
