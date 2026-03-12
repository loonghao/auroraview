# AuroraView Core 代码说明与「去掉白色边框」修改指南

本文档面向：第一次接触 Rust、之前主要用 Python 的开发者。会用「Python 里类似什么」的方式解释 `crates/auroraview-core` 在做什么，并说明**若要去掉 QtWebView 的白色边框，应从何处入手**。

---

## 一、auroraview-core 是做什么的？（整体类比）

可以把它想成：

- **Python 里的一个「工具库 + 配置/协议定义」包**：不负责启动整个 GUI 程序，而是提供：
  - 静态资源（HTML/JS）
  - 配置结构体（类似 dataclass）
  - 协议/URL 处理
  - **窗口样式工具函数**（尤其是 Windows 下改窗口边框、无边框、子窗口等）
  - IPC 消息类型、后端抽象接口等

- **WebView 窗口本身不是在 core 里创建的**。真正创建窗口和 WebView 的代码在**工作区根目录**的 `src/webview/`（例如 `desktop.rs`、`backend/native.rs`）。  
  core 只提供：**配置类型、样式工具函数、资源**，给上层（如 `auroraview` 主库、CLI）用。

所以：  
**「修改 QtWebView 的创建样式、去掉白色边框」** 主要涉及两处：

1. **样式逻辑定义**：在 `crates/auroraview-core/src/builder/window_style.rs`
2. **样式在何时、对哪个窗口调用**：在 `src/webview/`（如 `backend/native.rs`、`desktop.rs`）

下面先简述 core 各模块，再专门说「白色边框」的修改入口。

---

## 二、core 各模块简要说明（用 Python 能懂的话说）

### 2.1 `lib.rs`：入口与「导出」

- 类似 Python 里一个包的 `__init__.py`：声明子模块（`pub mod xxx`）和对外暴露的接口（`pub use ...`）。
- 从这里可以看到 core 提供了：assets、backend、bom、builder、cleanup、cli、config、dom、events、icon、ipc、menu、metrics、port、protocol、service_discovery、signals、templates、thread_safety、utils、window 等。

### 2.2 `config.rs`：配置结构体

- 类似 Python 的 `dataclass` 或 `pydantic.BaseModel`。
- 定义窗口/WebView 的通用配置：宽高、是否透明、是否无边框、父窗口句柄、嵌入模式（EmbedMode：None / Child / Owner）等。
- **和「白色边框」相关**：`decorations`（是否有标题栏/边框）、`embed_mode`（Qt 嵌入时用 Child 或 Owner）、`transparent` 等都会影响后面用哪一套窗口样式。

### 2.3 `builder/`：WebView 构建时的「共享逻辑」

- 不创建窗口，只提供「创建 WebView 时大家都会用到的逻辑」：
  - **`window_style.rs`**：**和边框、白边最相关**。提供一堆 Windows 下的窗口样式函数，例如：
    - `apply_child_window_style`：设为子窗口并**去掉容易导致白边的扩展样式**（如 WS_EX_STATICEDGE、WS_EX_CLIENTEDGE、WS_EX_WINDOWEDGE、WS_EX_DLGMODALFRAME）。
    - `apply_frameless_window_style` / `apply_frameless_popup_window_style`：无标题栏、无边框。
    - `disable_window_shadow`：关阴影、关圆角、边框厚度等（DWM 相关）。
    - `remove_clip_children_style`：透明窗口时必须去掉 WS_CLIPCHILDREN，否则子窗口（WebView2）会透出白底。
  - `common_config.rs`：例如深色背景色常量，用来减少「白闪」。
  - `com_init.rs`：Windows COM 初始化（WebView2 需要）。
  - `web_context.rs`：WebView 数据目录等。
  - 其他：拖拽、IPC 处理、协议注册等（在启用 `wry-builder` 时）。

### 2.4 `backend/`：WebView 后端的「抽象层」

- 定义「WebView 后端」要实现的接口（trait），类似 Python 的抽象基类（ABC）。
- 具体实现（例如用 wry/WebView2）在**主仓库**的 `src/webview/backend/native.rs`，不在 core 里。
- core 只提供：错误类型、工厂、生命周期、消息处理等抽象，方便多后端或多模式（独立窗口 / 嵌入 Qt）共用一套接口。

### 2.5 `ipc/`：进程间/前后端消息

- 定义「发往 WebView 的消息类型」和窗口事件类型（如 WebView2 创建完成）。
- 类似定义了一堆「协议结构体」：加载 URL、执行 JS、关闭、窗口事件等。不涉及窗口长什么样。

### 2.6 `assets`、`templates`、`bom`、`dom`、`menu`、`cleanup` 等

- **assets**：嵌入的 HTML/JS（加载页、错误页、前端脚本等）。
- **templates**：用 Askama 渲染的 JS 模板（类似 Jinja2）。
- **bom**：Browser Object Model，暴露给前端的 API（导航、缩放等）。
- **dom**：DOM 操作封装（发 JS 到 WebView 执行）。
- **menu**：菜单相关。
- **cleanup**：清理 WebView 用户数据目录等。

这些和「窗口白色边框」无直接关系，知道是「资源与能力」即可。

---

## 三、「QtWebView」和实际实现的关系

- 项目里**没有**直接使用 Qt 的 QWebView 控件。
- 实际是：用 **wry + WebView2（Windows）** 创建一个**无边框/子窗口**，再把这个窗口**嵌入到 Qt 窗口**里（通过父窗口句柄 parent_hwnd / SetParent）。
- 所以你说的「QtWebView 的创建样式」在这套架构里 = **wry 创建的那个窗口的样式**（以及 WebView2 内部子窗口的样式）。  
  这些样式的**定义**在 core 的 `builder/window_style.rs`，**调用时机**在 `src/webview/`。

---

## 四、去掉白色边框：应该从哪里入手？

白色边框通常来自几类东西：

1. **窗口的扩展样式**：如 `WS_EX_CLIENTEDGE`、`WS_EX_STATICEDGE`、`WS_EX_WINDOWEDGE` 等，会画出系统边框（常显为白/灰边）。
2. **WebView2 内部子窗口**：WebView2 会创建多个子窗口，若它们带着边框样式，也会出现白边。
3. **DWM 边框/阴影**：Win11 的圆角、边框颜色、边框厚度等。

对应到代码位置和修改方式如下。

### 4.1 核心文件：`crates/auroraview-core/src/builder/window_style.rs`

这里已经实现了「去掉会引发白边的样式」：

- **`apply_child_window_style`**（约 97–165 行）  
  - 在设为子窗口时，会**去掉**：`WS_EX_STATICEDGE`、`WS_EX_CLIENTEDGE`、`WS_EX_WINDOWEDGE`、`WS_EX_DLGMODALFRAME`。  
  - 注释里明确写了："Remove extended styles that can cause white borders"。
- **`compute_frameless_window_styles` / `apply_frameless_window_style`**  
  - 无边框窗口时也会去掉上述类似样式以及 WS_CAPTION、WS_THICKFRAME、WS_BORDER 等。

**若你仍看到白边，可以：**

1. **在 core 里加强「去边框」**  
   - 在 `apply_child_window_style` 或 `compute_frameless_window_styles` 里，确认没有遗漏任何会画边的扩展样式（可查 MSDN 的 Extended Window Styles），再按需去掉。
2. **确保嵌入 Qt 时真的调用了这些函数**（见下一小节）。

### 4.2 调用处 1：嵌入模式（Qt 里用 parent + Child/Owner）— `src/webview/backend/native.rs`

- 嵌入到 Qt 时，走的是 **NativeBackend::create_embedded**。
- 在 **WebView2 创建之后**（约 939–1012 行），会根据 `config.decorations` 和 `config.embed_mode` 应用样式：
  - **Child 模式**：`apply_frameless_window_style(hwnd)`（约 956–958 行）。
  - **Owner / None**：`apply_frameless_popup_window_style(hwnd)`。
  - 然后还有 `disable_window_shadow`、`remove_clip_children_style`（透明时）、`extend_frame_into_client_area` 等。

**重要点：**  
在 **Child 模式**下，当前代码**没有**在 `native.rs` 里调用 **`apply_child_window_style`**，只调用了 `apply_frameless_window_style`。  
而 `apply_child_window_style` 会额外做两件事：  
① 再次确保去掉那几种「易产生白边」的扩展样式；  
② 强制设为 WS_CHILD 并 SetParent。

**建议修改（在 `src/webview/backend/native.rs`）：**

- 在 **EmbedMode::Child** 分支里，在调用 `apply_frameless_window_style` **之后**（或替代/补充），增加对 **`apply_child_window_style`** 的调用，并传入当前窗口的 `hwnd` 和 Qt 的 `parent_hwnd`，例如：

```rust
use auroraview_core::builder::{
    apply_child_window_style,
    ChildWindowStyleOptions,
    // ... 已有 imports
};

// 在 match config.embed_mode { EmbedMode::Child => { ... } } 里
EmbedMode::Child => {
    let _ = apply_frameless_window_style(hwnd_value);
    // 确保子窗口且去掉易产生白边的扩展样式
    let _ = apply_child_window_style(
        hwnd_value,
        parent_hwnd as isize,
        ChildWindowStyleOptions::for_dcc_embedding(), // 或 for_standalone() 看需求
    );
}
```

这样会**在嵌入 Qt 的 Child 窗口上**再执行一遍「去白边」的扩展样式移除。

### 4.3 调用处 2：WebView2 子窗口 — `fix_webview2_child_windows`（同一文件 `native.rs`）

- WebView2 会创建多个子窗口（如 Chrome_WidgetWin_0）。若它们带边框样式，也会出现白边。
- **`fix_webview2_child_windows(hwnd)`**（约 447–660 行）会递归遍历子窗口，去掉 `WS_BORDER`、`WS_CAPTION`、`WS_THICKFRAME`、`WS_DLGFRAME` 以及 `WS_EX_*EDGE*` 等，并做子类化防止拖动。

**你需要确认：**  
在嵌入 Qt 的创建路径里，**是否在合适的时机调用了 `fix_webview2_child_windows`**（例如在 WebView2 创建完成、并应用完主窗口样式之后）。若没有，可以在 `native.rs` 的 create_embedded 里，在应用完上述样式之后增加一次调用：

```rust
Self::fix_webview2_child_windows(hwnd_value);
```

这样 WebView2 内部的子窗口也不会带边框样式。

### 4.4 调用处 3：独立窗口 / desktop 路径 — `src/webview/desktop.rs`

- 独立窗口或带 parent 的桌面窗口会走 `desktop.rs`。
- 这里已经对「有 parent_hwnd + Child」调用了 **`apply_child_window_style`**（约 269–272 行），并在后面无边框、tool window 等逻辑里调用 `apply_frameless_*`、`disable_window_shadow` 等。

若你**不是**嵌入 Qt，而是独立窗口仍有白边，可以重点看 `desktop.rs` 里对 `apply_frameless_*` 和 `disable_window_shadow` 的调用是否都执行到了（例如是否被 `decorations == true` 等条件跳过）。

### 4.5 小结：修改清单（去掉白色边框）

| 目标 | 位置 | 建议 |
|-----|------|------|
| 样式定义（去哪些位） | `crates/auroraview-core/src/builder/window_style.rs` | 检查/加强 `apply_child_window_style`、`compute_frameless_window_styles` 里对 WS_EX_*EDGE* 等的清除。 |
| 嵌入 Qt（Child） | `src/webview/backend/native.rs`（create_embedded） | 在 Child 分支中增加 `apply_child_window_style(hwnd, parent_hwnd, ...)`，并确认调用 `fix_webview2_child_windows(hwnd)`。 |
| 独立窗口 | `src/webview/desktop.rs` | 确认无边框时 `apply_frameless_*`、`disable_window_shadow` 等都被执行。 |
| DWM 边框/阴影 | `window_style.rs` 的 `disable_window_shadow` | 已设边框厚度、圆角、颜色等；若仍有残留可在此函数里再收紧 DWM 属性。 |

---

## 五、Rust 和 Python 的简单对照（方便你读代码）

- **`pub mod xxx`**：类似 Python 的 `from . import xxx` 或包里的子模块。
- **`pub use ...`**：把别处的类型/函数重新导出，类似 `from .window_style import apply_child_window_style` 并在 `__init__.py` 里再 export。
- **`#[cfg(target_os = "windows")]`**：仅 Windows 编译，类似 `if sys.platform == "win32":`，但发生在编译期。
- **`unsafe { ... }`**：调用 FFI 或可能违反内存安全的代码，类似 Python 里调 C 扩展时要小心约定。
- **`Result<T, E>`**：类似返回 `(T, None)` 或 `(None, E)`，用 `?` 或 `match` 处理错误。
- **`Option<T>`**：类似 `T | None`。

---

如果你能提供：是「嵌入到 Qt 的 Child 窗口」还是「独立窗口」、以及系统（Win10/Win11），我可以按你的场景给出一份最小改动的 patch 式修改步骤（具体到行号与补丁）。
