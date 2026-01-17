# RFC 0006: AuroraView Browser Crate

## Summary

将浏览器功能从 `auroraview` 主库独立出来，创建 `auroraview-browser` crate，提供完整的多标签页浏览器功能，包括 Tab 管理、导航控制、书签、历史记录、扩展系统等。

## Motivation

### 当前问题

1. **代码耦合**：浏览器相关代码（`tab_manager.rs`、`browser_controller.html`）分散在 `src/webview/` 和 `crates/auroraview-core/src/assets/` 中
2. **职责不清**：`TabManager` 既负责窗口管理又负责 UI 渲染，违反单一职责原则
3. **难以扩展**：书签、历史记录、扩展等功能难以添加到现有架构
4. **UI 定制困难**：Controller HTML 硬编码在 Rust 代码中，无法方便地定制样式

### 目标

- 创建独立的 `auroraview-browser` crate，专注于浏览器功能
- 提供 Edge/Chrome 风格的现代化 UI
- 支持书签栏、历史记录、扩展系统
- 易于定制和扩展
- 可独立使用，也可集成到 DCC 应用中

## Design

### 1. Crate 结构

```
crates/auroraview-browser/
├── Cargo.toml
├── src/
│   ├── lib.rs                 # 公共 API
│   ├── browser.rs             # Browser 主结构体
│   ├── tab/
│   │   ├── mod.rs
│   │   ├── tab.rs             # Tab 数据结构
│   │   ├── manager.rs         # Tab 管理器
│   │   └── events.rs          # Tab 事件
│   ├── navigation/
│   │   ├── mod.rs
│   │   ├── history.rs         # 历史记录
│   │   └── bookmarks.rs       # 书签管理
│   ├── extensions/
│   │   ├── mod.rs
│   │   ├── extension.rs       # 扩展接口
│   │   └── registry.rs        # 扩展注册表
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── theme.rs           # 主题系统
│   │   ├── controller.rs      # Controller WebView
│   │   └── components/        # UI 组件
│   └── config.rs              # 配置
├── assets/
│   ├── html/
│   │   ├── controller.html    # 主控制器 UI
│   │   ├── new_tab.html       # 新标签页
│   │   └── settings.html      # 设置页面
│   ├── css/
│   │   ├── themes/
│   │   │   ├── light.css      # 浅色主题（Edge 风格）
│   │   │   └── dark.css       # 深色主题
│   │   └── components/
│   ├── js/
│   │   ├── controller.js      # Controller 逻辑
│   │   ├── tab-bar.js         # 标签栏组件
│   │   └── toolbar.js         # 工具栏组件
│   └── icons/                 # SVG 图标
└── tests/
```

### 2. 核心 API

```rust
// crates/auroraview-browser/src/lib.rs

/// Browser configuration
pub struct BrowserConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub home_url: String,
    pub theme: Theme,
    pub features: BrowserFeatures,
    pub debug: bool,
}

/// Browser features toggle
pub struct BrowserFeatures {
    pub bookmarks_bar: bool,
    pub history: bool,
    pub extensions: bool,
    pub downloads: bool,
    pub dev_tools: bool,
}

/// Theme selection
pub enum Theme {
    Light,
    Dark,
    System,
    Custom(CustomTheme),
}

/// Main browser struct
pub struct Browser {
    config: BrowserConfig,
    tabs: TabManager,
    bookmarks: BookmarkManager,
    history: HistoryManager,
    extensions: ExtensionRegistry,
}

impl Browser {
    /// Create a new browser instance
    pub fn new(config: BrowserConfig) -> Self;
    
    /// Run the browser (blocking)
    pub fn run(&mut self);
    
    /// Tab operations
    pub fn new_tab(&mut self, url: &str) -> TabId;
    pub fn close_tab(&mut self, id: TabId);
    pub fn activate_tab(&mut self, id: TabId);
    pub fn get_active_tab(&self) -> Option<&Tab>;
    
    /// Navigation
    pub fn navigate(&mut self, url: &str);
    pub fn go_back(&mut self);
    pub fn go_forward(&mut self);
    pub fn reload(&mut self);
    
    /// Bookmarks
    pub fn add_bookmark(&mut self, url: &str, title: &str);
    pub fn remove_bookmark(&mut self, id: BookmarkId);
    pub fn get_bookmarks(&self) -> &[Bookmark];
    
    /// History
    pub fn get_history(&self, limit: usize) -> Vec<HistoryEntry>;
    pub fn clear_history(&mut self);
    
    /// Extensions
    pub fn register_extension(&mut self, ext: Box<dyn Extension>);
}
```

### 3. UI 架构（Edge 风格）

参考 Microsoft Edge 浏览器布局：

```
┌─────────────────────────────────────────────────────────────────────┐
│ [Tabs Row]                                                    [- □ x]│
│ ┌─────────┐ ┌─────────┐ ┌─────────┐ [+]                             │
│ │ Tab 1   │ │ Tab 2   │ │ Tab 3   │                                 │
│ └─────────┘ └─────────┘ └─────────┘                                 │
├─────────────────────────────────────────────────────────────────────┤
│ [Toolbar Row]                                                        │
│ [←] [→] [↻] [🏠] │ 🔍 Search or enter URL...              │ [⭐] [⋯] │
├─────────────────────────────────────────────────────────────────────┤
│ [Bookmarks Bar] (optional)                                           │
│ [📁 Folder] [🔖 Site1] [🔖 Site2] [🔖 Site3] ...                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│                      [Content Area]                                  │
│                                                                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

#### UI 组件层次

```
Controller (总高度: ~88-120px，取决于书签栏是否显示)
├── TabBar (38px)
│   ├── Tab[] (可拖拽、可右键菜单)
│   │   ├── Favicon (16x16)
│   │   ├── Title (ellipsis)
│   │   ├── LoadingIndicator
│   │   └── CloseButton
│   └── NewTabButton
├── Toolbar (50px)
│   ├── NavigationButtons (Back, Forward, Reload, Home)
│   ├── AddressBar (Omnibox)
│   │   ├── SecurityIcon (🔒/⚠️)
│   │   ├── URLInput
│   │   └── StarButton (收藏)
│   └── ExtensionButtons (Downloads, Extensions, Menu)
└── BookmarksBar (32px, optional)
    ├── BookmarkItem[]
    └── MoreButton (▼)
```

### 4. 事件系统

```rust
/// Browser events
pub enum BrowserEvent {
    // Tab events
    TabCreated { id: TabId, url: String },
    TabClosed { id: TabId },
    TabActivated { id: TabId },
    TabUpdated { id: TabId, title: String, url: String },
    TabLoading { id: TabId, is_loading: bool },
    
    // Navigation events
    NavigationStarted { tab_id: TabId, url: String },
    NavigationCompleted { tab_id: TabId, url: String },
    NavigationFailed { tab_id: TabId, error: String },
    
    // Bookmark events
    BookmarkAdded { id: BookmarkId, url: String, title: String },
    BookmarkRemoved { id: BookmarkId },
    
    // Download events
    DownloadStarted { id: DownloadId, url: String, filename: String },
    DownloadProgress { id: DownloadId, received: u64, total: u64 },
    DownloadCompleted { id: DownloadId, path: PathBuf },
}
```

### 5. 扩展系统

```rust
/// Extension trait for browser plugins
pub trait Extension: Send + Sync {
    /// Extension unique identifier
    fn id(&self) -> &str;
    
    /// Display name
    fn name(&self) -> &str;
    
    /// Extension icon (SVG or data URL)
    fn icon(&self) -> Option<&str>;
    
    /// Called when extension is loaded
    fn on_load(&mut self, browser: &Browser);
    
    /// Called when extension is unloaded
    fn on_unload(&mut self);
    
    /// Handle browser events
    fn on_event(&mut self, event: &BrowserEvent);
    
    /// Toolbar button click handler
    fn on_toolbar_click(&mut self, browser: &mut Browser);
    
    /// Get popup HTML (if any)
    fn popup_html(&self) -> Option<&str>;
}
```

### 6. Python API

```python
from auroraview import run_browser
from auroraview.browser import Browser, BrowserConfig, Theme

# Simple usage
run_browser(
    title="My Browser",
    width=1280,
    height=900,
    home_url="https://google.com",
    theme="light",  # or "dark", "system"
    bookmarks_bar=True,
    debug=True,
)

# Advanced usage with config
config = BrowserConfig(
    title="Custom Browser",
    width=1400,
    height=1000,
    home_url="https://github.com",
    theme=Theme.DARK,
    features={
        "bookmarks_bar": True,
        "history": True,
        "extensions": True,
        "downloads": True,
    },
)

browser = Browser(config)

# Add initial bookmarks
browser.add_bookmark("https://github.com", "GitHub")
browser.add_bookmark("https://google.com", "Google")

# Run (blocking)
browser.run()
```

## Implementation Plan

### Phase 1: 基础架构 (Week 1-2) ✅

1. [x] 创建 `crates/auroraview-browser/` 目录结构
2. [x] 实现 `BrowserConfig` 和 `Browser` 基础结构
3. [x] 迁移 `TabManager` 到新 crate
4. [x] 实现基础 Tab 管理功能

### Phase 2: UI 重构 (Week 2-3) ✅

1. [x] 创建 Edge 风格 Controller HTML/CSS/JS
2. [x] 实现主题系统（Light/Dark）
3. [x] 实现 Tab 拖拽排序
4. [x] 实现右键菜单

### Phase 3: 功能扩展 (Week 3-4) ✅

1. [x] 实现书签管理器
2. [x] 实现书签栏 UI
3. [x] 实现历史记录
4. [ ] 实现下载管理（待定）

### Phase 4: 扩展系统 (Week 4-5) ✅

1. [x] 定义 Extension trait
2. [x] 实现扩展注册和生命周期
3. [x] 创建 Chrome Extension 兼容层（通过 `plugins` feature 集成 auroraview-plugins）
4. [x] 添加 DevTools 和 CDP（Chrome DevTools Protocol）支持

### Phase 5: Python 绑定 (Week 5-6)

1. [ ] 导出 Python API
2. [ ] 更新 `run_browser` 函数
3. [ ] 添加示例和文档

## Migration Path

### 向后兼容

保留现有的 `run_tab_browser` 作为别名，标记为 deprecated：

```rust
#[deprecated(since = "0.5.0", note = "Use run_browser instead")]
pub fn run_tab_browser(...) -> PyResult<()> {
    run_browser(...)
}
```

### 代码迁移

1. `src/webview/tab_manager.rs` → `crates/auroraview-browser/src/tab/manager.rs`
2. `src/bindings/tab_browser.rs` → 调用 `auroraview-browser` crate
3. `crates/auroraview-core/src/assets/html/browser_controller.html` → `crates/auroraview-browser/assets/html/`

## Alternatives Considered

### 1. 保持在主库中

**优点**：无需迁移  
**缺点**：代码膨胀，职责不清

### 2. 仅分离 UI 资源

**优点**：改动小  
**缺点**：架构问题仍未解决

### 3. 使用 Feature Flag

**优点**：灵活  
**缺点**：编译复杂度增加

## Dependencies

- `auroraview-core`: 核心 WebView 功能
- `auroraview-plugins` (optional): Chrome Extension API 兼容层
- `wry`: WebView 后端
- `tao`: 窗口管理
- `serde`: 序列化（书签、历史等持久化）
- `rust-embed`: 静态资源嵌入
- `parking_lot`: 线程安全锁
- `chrono`: 时间处理
- `uuid`: 唯一标识符生成

## Open Questions

1. **持久化存储**：书签和历史记录应该存储在哪里？
   - 建议：`~/.auroraview/browser/` 或 `%APPDATA%/auroraview/browser/`

2. **扩展沙箱**：扩展是否需要权限系统？
   - 建议：初期简化，后续迭代

3. **同步功能**：是否支持跨设备同步？
   - 建议：v1.0 不支持，作为 Future Work

## References

- [Microsoft Edge WebView2Browser Sample](https://github.com/MicrosoftEdge/WebView2Browser)
- [Chromium Browser Architecture](https://www.chromium.org/developers/design-documents/)
- [Firefox Browser Architecture](https://firefox-source-docs.mozilla.org/browser/)
- [Tauri WebView Bindings](https://github.com/nicksrandall/tauri-webview-bindings)
