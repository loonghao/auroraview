---
type: "agent_requested"
description: "Example description"
---

AuroraView provides a modern web-based UI solution for professional DCC applications like Maya, 3ds Max, Houdini, Blender, Photoshop, and Unreal Engine. On Windows we now prioritize a WebView2 backend implemented in Rust (windows-rs + webview2-com) with PyO3 bindings and maturin packaging, optimized for embedding into Qt-based DCC hosts without extra Python dependencies.

## 常用对比框架

### WebView 框架
- pywebview: https://github.com/r0x0r/pywebview
- rust tauri: https://github.com/tauri-apps/tauri
- qtwebview: https://github.com/qt/qtwebview

### 打包参考
- PyOxidizer: https://github.com/loonghao/PyOxidizer
- Pake: https://github.com/tw93/Pake

## 核心技术栈（2025）
- Windows: WebView2（Chromium Edge）+ Rust（windows-rs, webview2-com），以“父 HWND 子窗口”方式内嵌至 Qt 容器；UI 操作在 STA 线程执行
- 跨 DCC：Maya/3ds Max/Houdini/Nuke 使用 Qt 宿主；Blender 采用浮动工具窗
- Python 包：仅一个 .pyd（pyo3 + maturin），不引入第三方 Python 依赖
- 并发模型：Rust 侧消息桥与调度，Qt 负责事件循环；Rust 不接管消息泵
- 跨平台路线：macOS（WKWebView）、Linux（WebKitGTK 或 CEF/OSR）按需启用
- 最低需要支持python-3.7

### HTML / 静态资源加载策略
- 内联 HTML：`WebView.load_html(html: str)`，适用于简单原型或不依赖本地静态资源的页面
- 本地静态站点：优先使用 `WebView.load_url("file:///.../index.html")` 加载磁盘上的 HTML 文件；相对路径的图片/CSS/JS 由浏览器根据该文件路径解析
- DCC + Qt 场景：`QtWebView` 只是一个 Qt 宿主 QWidget，所有导航行为都委托给核心 `WebView`：
  - `QtWebView.load_html(html: str)` → `WebView.load_html(html)`（不再暴露 `base_url`）
  - `QtWebView.load_url(url: str)` → `WebView.load_url(url)`，其中 `url` 可以是 `http://` 或 `file:///` URL
- 需要自定义相对路径解析时，推荐在 HTML 中使用 `<base href="file:///.../">` 显式声明，而不是在 Python API 上引入 `base_url` 参数。


## DCC 适配矩阵（Windows 优先）
- Maya 2025+（Qt6/PySide6）：QWidget.winId() → HWND；Qt 容器负责 resize/focus/DPI 事件
- 3ds Max（Qt）：QWidget.winId() → HWND；同 Maya 流程
- Houdini（PySide2/Qt5）：hou.qt.mainWindow().winId() → HWND；子 widget 作为容器
- Nuke（PySide2/Qt）：自定义 Panel 返回 QWidget；winId() → HWND
- Blender（非 Qt）：采用浮动/owned 顶层工具窗（Win32 + WebView2），提供置顶与尺寸钩子

## Python 包与 API（零依赖）
- 单一 .pyd：pyo3 + maturin，ABI3（按 DCC Python 版本矩阵构建 wheel）
- API（示意）：
  - create(parent_hwnd, x, y, w, h, url, opts) -> handle
  - set_bounds(handle, x, y, w, h)
  - navigate(handle, url)
  - eval(handle, js)
  - post_message(handle, json)
  - on_message(handle, callback)
  - dispose(handle)

## Standalone 模式
- 保留现有 wry/tao 模式用于独立运行和开发调试
- 后续可迁移为 Win32 + WebView2 顶层窗口（不依赖 wry/winit），与 DCC 内嵌实现一致

## Cargo Features（规划）
- 默认：win-webview2 + python-bindings
- 可选：wry-backend（standalone 兼容）
- 预留：mac-wkwebview、linux-webkitgtk、linux-cef（OSR）

## 事件循环与线程模型
- DCC：Qt 负责事件循环；WebView2 控制器/视图创建与操作在 STA/UI 线程执行
- Rust 通过任务队列将 UI 操作 marshal 到创建线程；DPI/焦点/IME 由 Qt 事件桥接
- Standalone：自有事件循环（当前 wry/tao），或改用 Win32 消息循环

## 测试与 CI

### 测试框架与组织策略

**Rust 测试框架**：
- **单元测试**：保留在源文件中的 `#[cfg(test)]` 模块
  - 测试私有函数和内部实现细节
  - 位置：与源代码在同一文件中
  - 运行：`cargo test --lib`

- **集成测试**：独立管理在 `tests/` 目录
  - 测试公共 API 和完整工作流
  - 使用 `rstest` 框架提供 fixture 和参数化测试能力
  - 位置：`tests/*.rs`
  - 运行：`cargo test --test '*' --features "test-helpers"`

- **测试框架依赖**（dev-dependencies）：
  - `rstest` - Fixture-based 测试框架，提供类似 pytest 的 fixture 功能
  - `test-case` - 表驱动测试
  - `proptest` - 属性测试
  - `mockall` - Mock 框架
  - `serial_test` - 串行测试执行
  - `criterion` - 性能基准测试

**Python 测试框架**：
- 使用 `pytest` 作为主测试框架
- 位置：`tests/*.py`
- 运行：`pytest tests/`

### 测试覆盖范围
- 冒烟：创建/显示/导航/resize/焦点 与 JS 消息收发
- 稳定性：睡眠/显示器切换/远程桌面 恢复；多实例并存
- 性能：首次可见 < 300ms、FPS≥60、空闲 CPU≤5%、内存增量≤200MB

### CI/CD 配置
- Windows 构建矩阵（cp39–cp312 x64），签名与 WebView2 Runtime 检测提示
- Rust 集成测试在 CI 中使用 `test-helpers` feature 运行
- 使用 `just` 命令统一本地开发和 CI 环境


- 多搜索互联网寻找解决方案
- 我们的代码需要兼容python-3.7和所有DCC环境
- 不需要做过多的总结，少用emoji表情在代码中。
- 每次提交代码到远端需要解决Lint和clippy的风格问题
- 避免optimized，fixed之类的词在我们的代码还有函数中
- 我们的函数，还有函数名字的设计要简短精炼，多使用行业标准。
- 不要自己造轮子，多用业内的成熟现有的crate
- 如果CI有问题你可以自己使用github API根据对于的PR去看我们相关的错误信息
- 为了保证我们开发的一致性，我们CI/CD和我们本地开发都多用just的命令去统一环境和管理
- 需要保证我们PR阶段的测试与我们发布的构建应该一致


## 前端 & Python 框架设计（AuroraView）

### 前端命名空间（window.auroraview）
- 全局唯一入口：`window.auroraview`
- 核心能力：
  - `auroraview.call(method: string, params?: any): Promise<any>`
    - `method` 为字符串命名空间，如：`"api.export_scene"`、`"tool.apply"`
    - `params` 支持：对象（当作 kwargs）、数组（当作位置参数）、单值（单一位置参数）
    - 正常返回：Promise resolve 为 Python 返回的 JSON 兼容值
    - 异常：Promise reject，错误对象包含 `name` / `message` / 可选 `code` / `data`
  - `auroraview.on(event: string, handler: (payload: any) => void): () => void`
    - 订阅后端推送事件，返回取消订阅函数
  - `auroraview.off(event: string, handler: (payload: any) => void): void`
- pywebview 风格语法糖：`auroraview.api`
  - 通过 JS Proxy 映射：`auroraview.api.foo(...args)` → `auroraview.call("api.foo", args)`
  - 主要服务于 PyWebView 用户迁移场景
- 预留字段：
  - `auroraview.platform?: string` （如 `"standalone"`、`"maya2025-pyside6"`）
  - `auroraview.token?: string` （CSRF/session token，对齐 pywebview）
- Ready 事件：
  - 注入完成后触发 `window` 级事件 `"auroraviewready"`
  - 前端可在该事件中安全使用 `auroraview.call/on/api`

### Python 抽象：AuroraView 基类

- 导入约定：`from auroraview import AuroraView`
- 构造参数（简化版）：
  - `parent: Any | None = None`
    - `None` → Standalone 模式（Rust Shell / WebView 壳）
    - Qt 对象 → DCC 模式（PySide 宿主，作为 parent dock/panel）
  - `url: str | None = None` / `html: str | None = None`
  - `title: str = "AuroraView"`
  - `width: int = 800, height: int = 600`
  - `fullscreen: bool = False`
  - `debug: bool = False`
  - `api: Any | None = None`  → 暴露到 `auroraview.api.*`
- 生命周期钩子（可覆写）：
  - `def on_show(self) -> None: ...`
  - `def on_hide(self) -> None: ...`
  - `def on_close(self) -> None: ...`
  - `def on_ready(self) -> None: ...`  （WebView & JS bridge 就绪）
- 事件推送：
  - `self.emit(event_name: str, payload: Any) -> None`
    - Python → JS：触发前端 `auroraview.on(event_name, handler)`

### JS → Python 映射机制

1. `auroraview.api.*` 风格
   - Python 在构造 `AuroraView` 时传入 `api` 对象，例如 `api=self`
   - 规则：`auroraview.api.export_scene(*args)` → Python `api.export_scene(*args)`
   - 实现：
     - JS：`auroraview.api.foo(...args)` → `auroraview.call("api.foo", args)`
     - Python：解析 method 前缀 `"api."`，以 `getattr(api, name)` 查找方法
     - `params` 为 list → 调用 `func(*params)`；为 dict → 调用 `func(**params)`

2. 通用 `auroraview.call(method, params)`
   - AuroraView 提供注册接口：`bind_call(method: str, func: Callable)`
   - 使用示例：
     - JS：`auroraview.call("tool.apply", {"strength": 0.8})`
     - Python：
       - `self.bind_call("tool.apply", self.apply_tool)`
       - `def apply_tool(self, strength: float, mode: str = "preview"): ...`
   - 调用规则：
     - `params` 是 dict → `func(**params)`
     - `params` 是 list → `func(*params)`
     - 其它 → 单一位置参数

### auroraview.call 请求/响应协议（目标结构）

- JS → 后端（Rust/Python）消息：
  - `type: "call"`
  - `id: string` 唯一ID（例如自增序号 + 时间戳），用于在回包中对应 Promise
  - `method: string` 方法名（如 `"api.echo"`、`"tool.apply"`）
  - `params: any` 参数（对象/数组/单值，遵循上文的调用规则）

- 后端 → JS 回包消息：
  - `type: "call_result"`
  - `id: string` 必须与请求中 `id` 完全一致
  - `ok: bool` 调用是否成功
  - 成功时：`result: any` 为返回值（JSON 兼容类型）
  - 失败时：`error: { name: string; message: string; code?: string | int; data?: any }`

- JS 侧 Promise 语义：
  - 发起调用时，JS 为 `id` 建立一个 pending Promise
  - 收到 `call_result`：
    - `ok == true` → `resolve(result)`
    - `ok == false` → `reject(error)`
  - 如果超时或底层报错，可在 JS 层构造 `error` 并 `reject`

### 事件分发机制（Python → JavaScript）

**关键设计决策（2025-01）**：统一使用 `window.auroraview.trigger()` 进行事件分发

- **Rust 层实现**：
  - 所有 `WebViewMessage::EmitEvent` 处理统一使用 `window.auroraview.trigger(event, data)`
  - 替代原有的 `window.dispatchEvent(new CustomEvent(...))`
  - 涉及文件：
    - `src/webview/backend/native.rs`
    - `src/webview/event_loop.rs`（两处）
    - `src/webview/webview_inner.rs`
    - `src/webview/backend/mod.rs`
    - `src/webview/backend/custom.rs`

- **标准模式**：
  ```javascript
  (function() {
      if (window.auroraview && window.auroraview.trigger) {
          window.auroraview.trigger('event_name', data);
      } else {
          console.error('[AuroraView] Event bridge not ready, cannot emit event: event_name');
      }
  })();
  ```

- **前端接收**：
  - 使用 `window.auroraview.on(event, handler)` 订阅事件
  - 与 Rust 层的 `trigger()` 完全匹配
  - 形成完整的事件流：`Python emit()` → `Rust trigger()` → `JS on()`

- **架构优势**：
  - API 一致性：统一使用 AuroraView 事件系统，不混用浏览器原生 API
  - 错误处理：事件桥接未就绪时有明确错误提示
  - 可维护性：所有事件分发逻辑集中在 `window.auroraview` 命名空间



### 最小端到端示例（当前实现）

> 注意：当前实现中，`auroraview.call()` 在桥接层还是 *fire-and-forget*，Promise 会立即 resolve 为 `undefined`；
> 最推荐的模式是：前端通过 `auroraview.api.*` 触发 Python 逻辑，Python 再通过 `emit` 向前端推事件。

- 前端（JS）：

  ```js
  // 等待 bridge 就绪
  window.addEventListener('auroraviewready', () => {
    // 订阅来自 Python 的 echo 结果
    auroraview.on('echo_result', (payload) => {
      console.log('[AuroraView] echo_result from Python:', payload);
    });

    // 通过 api 调用 Python 方法（fire-and-forget）
    auroraview.api.echo({ message: 'hello from JS' });
  });
  ```

- Python（概念示例，基于 AuroraView 基类）：

  ```python
  from auroraview import AuroraView

  class MyTool(AuroraView):
      def __init__(self, **kwargs):
          super().__init__(api=self, **kwargs)

      # JS 侧调用：auroraview.api.echo({ message: "..." })
      def echo(self, message: str) -> None:
          # 当前实现：通过事件把结果推回前端
          self.emit("echo_result", {"message": message})
  ```

后续在 `call` 的 request/response 协议补全后，可以把 `echo` 设计为真正返回值的函数，
前端直接使用 `await auroraview.api.echo("hello")` 获取结果。

### Standalone / DCC 双模式

- Standalone：
  - 不依赖 Qt/PySide，仅使用 Rust 壳 + WebView（现有 wry/tao，可逐步演进到 Win32 + WebView2）
  - `parent=None`，AuroraView 负责创建顶层窗口并托管事件循环
- DCC：
  - 仅在 Python 层使用 Qt/PySide 作为宿主壳（通过 `parent` 注入）
  - Rust/Python core 不直接依赖 Qt；Qt 逻辑集中在 `auroraview.dcc.qt_shell` 类模块
  - 通过 parent 将 WebView 子窗口嵌入 DCC 主窗口（Dock/Pane），保持原生体验

### Rust / Python 分层结构（目标）

- Rust：
  - `core/`：协议与 RPC（call/on）、事件总线、会话管理、http_discovery
  - `backend/`：平台 & WebView 实现（如 win_webview2、wry、mac_wkwebview、linux_webkitgtk）
  - `host/`：宿主集成（如 standalone 进程、DCC 宿主 Maya/Nuke/Houdini 等）
  - `bridge/js_injection.rs`：集中管理注入的 JS 片段（构造 `window.auroraview`）
  - `python_binding/`：PyO3 暴露给 Python 的 API，包装 core + backend/host

- Python：
  - `AuroraView` 基类：统一 JS 桥、事件、call 绑定与生命周期钩子
  - `core.py`：不依赖 Qt 的 Window/App 抽象，对接 Rust binding
  - `backend/`：平台相关包装（如 standalone.py、qt.py），负责创建/管理 WebView 窗口
  - `host/`：DCC 集成层（如 maya.py、nuke.py），负责获取 parent、dock/panel 注册等
  - 通过 `api`/`bind_call`/`emit` 将 Rust/backend/host 差异封装在内部，对前端保持统一协议

## JavaScript 资源统一管理

### 设计决策（2025-01）

**所有 JavaScript 代码统一通过 `js_assets.rs` 模块管理**

- **问题背景**：之前 Rust 代码中存在大量硬编码的 JavaScript 字符串，分散在多个文件中（`event_loop.rs`、`backend/native.rs`、`backend/mod.rs`、`webview_inner.rs`、`standalone.rs` 等），难以维护和更新。

- **解决方案**：
  1. **集中管理**：所有 JavaScript 资源文件统一存放在 `src/assets/js/` 目录下
  2. **模板化**：运行时动态生成的 JavaScript 代码使用模板文件（`src/assets/js/runtime/`）
  3. **统一接口**：通过 `src/webview/js_assets.rs` 模块提供统一的访问接口

### 目录结构

```
src/assets/js/
├── core/              # 核心功能脚本
│   └── event_bridge.js
├── features/          # 功能特性脚本
│   └── context_menu.js
└── runtime/           # 运行时模板脚本
    ├── emit_event.js  # 事件触发模板
    └── load_url.js    # URL 加载模板
```

### 使用方式

#### 1. 静态资源

```rust
use crate::webview::js_assets;

// 获取事件桥接脚本
let script = js_assets::EVENT_BRIDGE;

// 获取上下文菜单禁用脚本
let script = js_assets::CONTEXT_MENU_DISABLE;
```

#### 2. 运行时模板

```rust
use crate::webview::js_assets;

// 生成事件触发脚本
let json_str = data.to_string();
let escaped_json = json_str.replace('\\', "\\\\").replace('\'', "\\'");
let script = js_assets::build_emit_event_script("my_event", &escaped_json);

// 生成 URL 加载脚本
let script = js_assets::build_load_url_script("https://example.com");
```

### 架构优势

✅ **集中管理**：所有 JavaScript 代码集中在一个目录，易于查找和维护
✅ **类型安全**：通过 Rust 函数接口访问，编译时检查
✅ **易于测试**：JavaScript 代码可以独立测试
✅ **版本控制**：JavaScript 代码变更可以清晰追踪
✅ **代码复用**：避免重复的硬编码字符串
✅ **一致性**：所有事件触发和 URL 加载使用统一模板，确保行为一致

---

## 架构最佳实践参考（Qt WebView & Flet SDK 研究）

> 本节内容基于对 Qt WebView 和 Flet Python SDK 的深入研究，提取可应用于 AuroraView 的设计模式和最佳实践。

### 1. 跨平台抽象层设计（参考 Qt WebView）

#### 1.1 抽象基类模式

Qt WebView 使用纯虚基类定义平台无关接口：

```cpp
// Qt WebView 抽象基类设计（参考）
class QWebViewPrivate : public QObject {
public:
    // Core navigation
    virtual void setUrl(const QUrl &url) = 0;
    virtual QUrl url() const = 0;
    virtual bool canGoBack() const = 0;
    virtual bool canGoForward() const = 0;
    virtual void goBack() = 0;
    virtual void goForward() = 0;

    // Content loading
    virtual void loadHtml(const QString &html, const QUrl &baseUrl) = 0;
    virtual int loadProgress() const = 0;
    virtual bool isLoading() const = 0;

    // JavaScript execution with callback
    virtual void runJavaScript(
        const QString &script,
        const std::function<void(const QVariant &)> &resultCallback
    ) = 0;

    // Cookie management
    virtual void setCookie(const QString &domain, const QString &name, const QString &value) = 0;
    virtual void deleteCookie(const QString &domain, const QString &name) = 0;
    virtual void deleteAllCookies() = 0;

    // Settings abstraction
    virtual QWebViewSettingsPrivate *settings() const = 0;
    virtual QString httpUserAgent() const = 0;
};
```

**AuroraView 应用建议**：

```rust
// Rust trait 定义（建议实现）
pub trait WebViewBackend: Send + Sync {
    // Core navigation
    fn set_url(&self, url: &str) -> Result<(), WebViewError>;
    fn url(&self) -> Option<String>;
    fn can_go_back(&self) -> bool;
    fn can_go_forward(&self) -> bool;
    fn go_back(&self) -> Result<(), WebViewError>;
    fn go_forward(&self) -> Result<(), WebViewError>;

    // Content loading
    fn load_html(&self, html: &str, base_url: Option<&str>) -> Result<(), WebViewError>;
    fn load_progress(&self) -> u8;
    fn is_loading(&self) -> bool;

    // JavaScript execution with async callback
    fn run_javascript<F>(&self, script: &str, callback: F) -> Result<(), WebViewError>
    where
        F: FnOnce(Result<serde_json::Value, WebViewError>) + Send + 'static;

    // Cookie management
    fn set_cookie(&self, domain: &str, name: &str, value: &str) -> Result<(), WebViewError>;
    fn delete_cookie(&self, domain: &str, name: &str) -> Result<(), WebViewError>;
    fn delete_all_cookies(&self) -> Result<(), WebViewError>;

    // Settings
    fn settings(&self) -> &dyn WebViewSettings;
    fn http_user_agent(&self) -> String;
}
```

#### 1.2 工厂模式与插件系统

Qt WebView 使用工厂模式创建平台特定实现：

```cpp
// Qt 工厂模式（参考）
QWebViewPrivate *QWebViewFactory::createWebView(QWebView *view) {
    QWebViewPlugin *plugin = getPlugin();
    if (plugin)
        return plugin->create(QStringLiteral("webview"), view);
    return nullptr;
}

// 环境变量覆盖后端选择
static QString getPluginName() {
    static const QString name = !qEnvironmentVariableIsEmpty("QT_WEBVIEW_PLUGIN")
        ? QString::fromLatin1(qgetenv("QT_WEBVIEW_PLUGIN"))
        : QStringLiteral("native");
    return name;
}
```

**AuroraView 应用建议**：

```rust
// Backend factory pattern
pub struct WebViewFactory;

impl WebViewFactory {
    pub fn create(config: &WebViewConfig) -> Result<Box<dyn WebViewBackend>, WebViewError> {
        // Environment variable override
        let backend_name = std::env::var("AURORAVIEW_BACKEND")
            .unwrap_or_else(|_| Self::default_backend());

        match backend_name.as_str() {
            "webview2" => Ok(Box::new(WebView2Backend::new(config)?)),
            "wry" => Ok(Box::new(WryBackend::new(config)?)),
            #[cfg(target_os = "macos")]
            "wkwebview" => Ok(Box::new(WKWebViewBackend::new(config)?)),
            _ => Err(WebViewError::UnsupportedBackend(backend_name)),
        }
    }

    fn default_backend() -> String {
        #[cfg(target_os = "windows")]
        return "webview2".to_string();
        #[cfg(target_os = "macos")]
        return "wkwebview".to_string();
        #[cfg(target_os = "linux")]
        return "wry".to_string();
    }
}
```

#### 1.3 设置抽象层

Qt WebView 为每个平台提供独立的设置实现：

```cpp
// Qt Settings 抽象（参考）
class QWebViewSettingsPrivate {
public:
    virtual bool localStorageEnabled() const = 0;
    virtual void setLocalStorageEnabled(bool enabled) = 0;
    virtual bool javaScriptEnabled() const = 0;
    virtual void setJavaScriptEnabled(bool enabled) = 0;
    virtual bool localContentCanAccessFileUrls() const = 0;
    virtual void setLocalContentCanAccessFileUrls(bool enabled) = 0;
    virtual bool allowFileAccessFromFileUrls() const = 0;
    virtual void setAllowFileAccessFromFileUrls(bool enabled) = 0;
};
```

**AuroraView 应用建议**：

```rust
// Settings trait
pub trait WebViewSettings: Send + Sync {
    fn local_storage_enabled(&self) -> bool;
    fn set_local_storage_enabled(&mut self, enabled: bool);
    fn javascript_enabled(&self) -> bool;
    fn set_javascript_enabled(&mut self, enabled: bool);
    fn allow_file_access(&self) -> bool;
    fn set_allow_file_access(&mut self, enabled: bool);
    fn dev_tools_enabled(&self) -> bool;
    fn set_dev_tools_enabled(&mut self, enabled: bool);
}
```

### 2. Python API 设计模式（参考 Flet SDK）

#### 2.1 Dataclass + Decorator 控件系统

Flet 使用 dataclass 和装饰器实现类型安全的控件定义：

```python
# Flet 控件装饰器模式（参考）
from dataclasses import dataclass, field
from typing import Optional, Any

@dataclass_transform()
def control(
    dart_widget_name: Optional[str] = None,
    *,
    isolated: Optional[bool] = None,
    **dataclass_kwargs: Any,
):
    """Decorator to define a control with optional widget name and isolation."""
    def wrapper(cls):
        cls = dataclass(**dataclass_kwargs)(cls)
        # Set widget type and isolation
        return cls
    return wrapper

@control("Button")
class Button(BaseControl):
    text: str = ""
    on_click: Optional[EventHandler] = None
```

**AuroraView 应用建议**：

```python
# AuroraView 控件装饰器（建议实现）
from dataclasses import dataclass, field
from typing import Optional, Callable, Any

def component(name: Optional[str] = None, *, isolated: bool = False):
    """Decorator to define an AuroraView component."""
    def wrapper(cls):
        cls = dataclass(kw_only=True)(cls)
        cls._component_name = name or cls.__name__
        cls._isolated = isolated
        return cls
    return wrapper

@component("CustomPanel")
class CustomPanel(BaseComponent):
    title: str = ""
    width: int = 400
    height: int = 300
    on_ready: Optional[Callable[[], None]] = None
```

#### 2.2 事件系统设计

Flet 的事件系统支持同步和异步处理：

```python
# Flet 事件处理模式（参考）
@dataclass
class Event(Generic[EventControlType]):
    name: str
    data: Optional[Any] = field(default=None, kw_only=True)
    control: EventControlType = field(repr=False)

    @property
    def page(self) -> "Page":
        return self.control.page

# 事件触发支持多种处理器类型
async def _trigger_event(self, event_name: str, event_data: Any):
    handler = getattr(self, f"on_{event_name}")

    if asyncio.iscoroutinefunction(handler):
        await handler(event)
    elif inspect.isgeneratorfunction(handler):
        for _ in handler(event):
            await session.after_event(self)
    elif callable(handler):
        handler(event)
```

**AuroraView 应用建议**：

```python
# AuroraView 事件系统（建议实现）
from dataclasses import dataclass, field
from typing import Generic, TypeVar, Optional, Any, Callable, Union
import asyncio
import inspect

T = TypeVar("T", bound="BaseComponent")

@dataclass
class Event(Generic[T]):
    """Base event class for AuroraView components."""
    name: str
    control: T = field(repr=False)
    data: Optional[Any] = None

    @property
    def webview(self) -> "WebView":
        return self.control.webview

# Event handler type aliases
EventHandler = Union[
    Callable[[], Any],
    Callable[[Event], Any],
    Callable[[], "Awaitable[Any]"],
    Callable[[Event], "Awaitable[Any]"],
]

class EventMixin:
    """Mixin for event handling support."""

    async def trigger_event(self, event_name: str, data: Any = None):
        handler = getattr(self, f"on_{event_name}", None)
        if handler is None:
            return

        event = Event(name=event_name, control=self, data=data)

        # Support both sync and async handlers
        if asyncio.iscoroutinefunction(handler):
            await handler(event)
        elif callable(handler):
            result = handler(event)
            if asyncio.iscoroutine(result):
                await result
```

#### 2.3 Context Variables 模式

Flet 使用 contextvars 实现线程安全的页面上下文：

```python
# Flet context 模式（参考）
from contextvars import ContextVar

_context_page: ContextVar["Page"] = ContextVar("context_page")

class Context:
    def reset_auto_update(self):
        self._auto_update = True

    def enable_components_mode(self):
        self._components_mode = True

context = Context()
```

**AuroraView 应用建议**：

```python
# AuroraView context 管理（建议实现）
from contextvars import ContextVar
from typing import Optional, TYPE_CHECKING

if TYPE_CHECKING:
    from .webview import WebView

_current_webview: ContextVar[Optional["WebView"]] = ContextVar(
    "current_webview", default=None
)

def get_current_webview() -> Optional["WebView"]:
    """Get the current WebView from context."""
    return _current_webview.get()

def set_current_webview(webview: "WebView") -> None:
    """Set the current WebView in context."""
    _current_webview.set(webview)

# Usage in event handlers
class WebView:
    async def _trigger_event(self, event_name: str, data: Any):
        set_current_webview(self)
        try:
            await self.trigger_event(event_name, data)
        finally:
            set_current_webview(None)
```

#### 2.4 Async-First 设计

Flet 采用 async-first 设计，同时支持同步调用：

```python
# Flet async-first 模式（参考）
def run(
    main: Union[Callable[["Page"], Any], Callable[["Page"], Awaitable[Any]]],
    **kwargs
):
    """Run the Flet app, supporting both sync and async main functions."""
    if is_pyodide():
        __run_pyodide(main=main)
        return
    return asyncio.run(run_async(main=main, **kwargs))

async def run_async(
    main: Union[Callable[["Page"], Any], Callable[["Page"], Awaitable[Any]]],
    **kwargs
):
    """Async entry point for Flet apps."""
    # Setup session, page, and run main
    pass
```

**AuroraView 应用建议**：

```python
# AuroraView async-first 设计（建议实现）
from typing import Union, Callable, Awaitable, Any
import asyncio

def run(
    main: Union[Callable[["WebView"], Any], Callable[["WebView"], Awaitable[Any]]],
    *,
    url: Optional[str] = None,
    html: Optional[str] = None,
    title: str = "AuroraView",
    width: int = 800,
    height: int = 600,
    debug: bool = False,
):
    """Run AuroraView app, supporting both sync and async main functions."""
    return asyncio.run(run_async(
        main=main,
        url=url,
        html=html,
        title=title,
        width=width,
        height=height,
        debug=debug,
    ))

async def run_async(
    main: Union[Callable[["WebView"], Any], Callable[["WebView"], Awaitable[Any]]],
    **kwargs
):
    """Async entry point for AuroraView apps."""
    webview = WebView(**kwargs)

    # Support both sync and async main
    if asyncio.iscoroutinefunction(main):
        await main(webview)
    else:
        main(webview)

    await webview.wait_closed()
```

### 3. 功能对比与改进路线图

#### 3.1 Qt WebView vs AuroraView 对比

| 功能领域 | Qt WebView | AuroraView 当前 | 改进建议 |
|---------|-----------|----------------|---------|
| 抽象层 | `QWebViewPrivate` 纯虚基类 | 无统一抽象 | 实现 `WebViewBackend` trait |
| 工厂模式 | `QWebViewFactory` + 插件系统 | 硬编码后端选择 | 实现工厂模式 + 环境变量覆盖 |
| 设置管理 | `QWebViewSettingsPrivate` 抽象 | 分散的配置参数 | 统一 `WebViewSettings` trait |
| Cookie 管理 | 完整 CRUD API | 未实现 | 添加 Cookie API |
| JS 执行 | 异步回调支持 | 同步执行 | 添加异步回调支持 |
| 导航事件 | 完整事件链 | 基础事件 | 扩展导航事件 |
| 错误处理 | 详细错误状态映射 | 基础错误类型 | 增强错误类型系统 |

#### 3.2 Flet SDK vs AuroraView 对比

| 功能领域 | Flet SDK | AuroraView 当前 | 改进建议 |
|---------|---------|----------------|---------|
| 控件系统 | Dataclass + 装饰器 | 类继承 | 考虑装饰器模式 |
| 事件系统 | 泛型 Event + 多处理器类型 | 基础回调 | 增强事件类型系统 |
| 异步支持 | Async-first + 同步兼容 | 同步为主 | 添加 async API |
| 上下文管理 | ContextVar | 无 | 添加上下文变量 |
| 生命周期 | `did_mount`/`will_unmount` | `on_ready`/`on_close` | 扩展生命周期钩子 |
| 服务注册 | `ServiceRegistry` | 无 | 考虑服务注册模式 |
| React Hooks | `use_state`/`use_effect` 等 | 无 | 可选：添加 hooks 支持 |

#### 3.3 改进优先级

**P0 - 核心架构改进**：
1. 实现 `WebViewBackend` trait 统一后端抽象
2. 实现工厂模式支持多后端选择
3. 统一设置管理接口

**P1 - API 增强**：
1. 添加 Cookie 管理 API
2. 添加 JavaScript 异步执行回调
3. 扩展导航事件系统
4. 增强错误类型系统

**P2 - Python API 改进**：
1. 添加 async/await 支持
2. 实现上下文变量管理
3. 扩展生命周期钩子
4. 考虑装饰器模式简化控件定义

### 4. 代码示例：统一后端抽象实现

```rust
// crates/auroraview-core/src/backend/mod.rs（建议结构）

mod webview2;
mod wry;
#[cfg(target_os = "macos")]
mod wkwebview;

use crate::error::WebViewError;
use serde_json::Value;

/// Unified WebView backend trait
pub trait WebViewBackend: Send + Sync {
    /// Initialize the backend with configuration
    fn initialize(&mut self, config: &BackendConfig) -> Result<(), WebViewError>;

    /// Navigate to URL
    fn navigate(&self, url: &str) -> Result<(), WebViewError>;

    /// Load HTML content
    fn load_html(&self, html: &str) -> Result<(), WebViewError>;

    /// Execute JavaScript with optional callback
    fn eval_js(&self, script: &str) -> Result<(), WebViewError>;

    /// Execute JavaScript with result callback
    fn eval_js_with_callback<F>(&self, script: &str, callback: F) -> Result<(), WebViewError>
    where
        F: FnOnce(Result<Value, WebViewError>) + Send + 'static;

    /// Get current URL
    fn current_url(&self) -> Option<String>;

    /// Navigation controls
    fn can_go_back(&self) -> bool;
    fn can_go_forward(&self) -> bool;
    fn go_back(&self) -> Result<(), WebViewError>;
    fn go_forward(&self) -> Result<(), WebViewError>;
    fn reload(&self) -> Result<(), WebViewError>;
    fn stop(&self) -> Result<(), WebViewError>;

    /// Cookie management
    fn set_cookie(&self, domain: &str, name: &str, value: &str) -> Result<(), WebViewError>;
    fn get_cookie(&self, domain: &str, name: &str) -> Result<Option<String>, WebViewError>;
    fn delete_cookie(&self, domain: &str, name: &str) -> Result<(), WebViewError>;
    fn clear_cookies(&self) -> Result<(), WebViewError>;

    /// Settings
    fn settings(&self) -> &dyn WebViewSettings;
    fn settings_mut(&mut self) -> &mut dyn WebViewSettings;

    /// Lifecycle
    fn close(&self) -> Result<(), WebViewError>;
    fn is_closed(&self) -> bool;
}

/// Backend factory
pub struct BackendFactory;

impl BackendFactory {
    pub fn create(config: &BackendConfig) -> Result<Box<dyn WebViewBackend>, WebViewError> {
        let backend_type = std::env::var("AURORAVIEW_BACKEND")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(Self::default_backend);

        match backend_type {
            BackendType::WebView2 => {
                #[cfg(target_os = "windows")]
                return Ok(Box::new(webview2::WebView2Backend::new(config)?));
                #[cfg(not(target_os = "windows"))]
                return Err(WebViewError::UnsupportedPlatform("WebView2 requires Windows"));
            }
            BackendType::Wry => Ok(Box::new(wry::WryBackend::new(config)?)),
            BackendType::WKWebView => {
                #[cfg(target_os = "macos")]
                return Ok(Box::new(wkwebview::WKWebViewBackend::new(config)?));
                #[cfg(not(target_os = "macos"))]
                return Err(WebViewError::UnsupportedPlatform("WKWebView requires macOS"));
            }
        }
    }

    fn default_backend() -> BackendType {
        #[cfg(target_os = "windows")]
        return BackendType::WebView2;
        #[cfg(target_os = "macos")]
        return BackendType::WKWebView;
        #[cfg(target_os = "linux")]
        return BackendType::Wry;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    WebView2,
    Wry,
    WKWebView,
}
```

### 5. 关键设计原则总结

1. **抽象优先**：使用 trait/interface 定义平台无关 API，具体实现隐藏在后端模块中
2. **工厂模式**：通过工厂创建后端实例，支持运行时选择和环境变量覆盖
3. **Async-First**：API 设计优先考虑异步，同时提供同步包装
4. **类型安全**：使用泛型和类型别名增强类型安全性
5. **生命周期明确**：提供清晰的生命周期钩子（init/mount/unmount/close）
6. **错误处理**：使用 Result 类型和详细的错误枚举
7. **上下文管理**：使用 ContextVar 管理线程/协程本地状态
8. **事件驱动**：统一的事件系统支持多种处理器类型
