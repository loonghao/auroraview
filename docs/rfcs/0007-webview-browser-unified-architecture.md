# RFC 0007: WebView 与 Browser 统一架构设计

## Summary

重新设计 AuroraView 的架构：
1. 将 Browser 中的通用能力拆分为**独立 crates**（核心 + 可选）
2. WebView 和 Browser 都可以按需组合这些 features
3. Browser 基于 WebView **组合模式**实现：Browser = Controller(WebView) + Tabs + 共享 Features

### Feature Crates 分类

**核心 Features（从 Browser 拆分）**：
- `auroraview-tabs` - 标签页管理
- `auroraview-extensions` - 扩展系统  
- `auroraview-bookmarks` - 书签管理
- `auroraview-history` - 历史记录
- `auroraview-devtools` - DevTools

**可选 Features（新增）**：
- `auroraview-downloads` - 下载管理器
- `auroraview-settings` - 设置管理
- `auroraview-notifications` - 通知管理

## Motivation

### 当前问题

```
┌─────────────────────────────────────────────────────────────────┐
│                      当前架构 (问题)                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   WebView (python/auroraview/core/webview.py)                   │
│   ├── 基础窗口管理                                                │
│   ├── 内容加载 (URL/HTML)                                        │
│   ├── JS 交互                                                    │
│   ├── 事件系统                                                    │
│   └── API 绑定                                                   │
│                                                                 │
│   Browser (crates/auroraview-browser/)    ← 平行关系，代码重复风险  │
│   ├── Tab 管理                                                   │
│   ├── 书签管理           ← 这些能力 WebView 也需要                  │
│   ├── 历史记录           ← 这些能力 WebView 也需要                  │
│   ├── 扩展系统           ← 这些能力 WebView 也需要                  │
│   ├── DevTools           ← 这些能力 WebView 也需要                  │
│   └── 浏览器 UI                                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

1. **架构不合理**：Browser 和 WebView 是平行关系，但 Browser 包含很多通用能力
2. **代码重复风险**：WebView 如果要用扩展/书签等功能，需要重复实现
3. **职责不清**：Browser 应该是"多 Tab WebView 容器"，而不是重新实现底层能力
4. **扩展功能受限**：普通桌面应用（如 Gallery）无法使用扩展系统

### 目标

- **Features 独立**：每个 feature 是独立 crate，可单独使用和复用
- **Browser = WebView 组合**：Browser 由多个 WebView 组合而成，复用 WebView 能力
- **组合优于继承**：通过组合模式添加功能
- **渐进式采用**：不破坏现有 API，新能力可选启用
- **分层架构**：核心 features 与可选 features 分离，按需引入

## Design

### 1. 新架构设计

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           新架构 (目标)                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                 Core Feature Crates (核心 - 从 Browser 拆分)     │   │
│   │                                                                 │   │
│   │  ┌─────────────────┐ ┌──────────────────┐  ┌───────────────┐   │   │
│   │  │auroraview-tabs  │ │auroraview-       │  │auroraview-    │   │   │
│   │  │                 │ │  extensions      │  │  bookmarks    │   │   │
│   │  │ - TabId         │ │ - Extension trait│  │ - Bookmark    │   │   │
│   │  │ - TabState      │ │ - Registry       │  │ - Folder      │   │   │
│   │  │ - TabManager    │ │ - Chrome bridge  │  │ - Manager     │   │   │
│   │  └─────────────────┘ └──────────────────┘  └───────────────┘   │   │
│   │                                                                 │   │
│   │  ┌──────────────────┐  ┌──────────────────┐                    │   │
│   │  │auroraview-       │  │auroraview-       │                    │   │
│   │  │  history         │  │  devtools        │                    │   │
│   │  │                  │  │                  │                    │   │
│   │  │ - HistoryEntry   │  │ - DevToolsManager│                    │   │
│   │  │ - Manager        │  │ - CDP support    │                    │   │
│   │  │ - Search         │  │                  │                    │   │
│   │  └──────────────────┘  └──────────────────┘                    │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │              Optional Feature Crates (可选 - 新增)               │   │
│   │                                                                 │   │
│   │  ┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐│   │
│   │  │auroraview-       │ │auroraview-       │ │auroraview-       ││   │
│   │  │  downloads       │ │  settings        │ │  notifications   ││   │
│   │  │                  │ │                  │ │                  ││   │
│   │  │ - DownloadItem   │ │ - SettingsStore  │ │ - Notification   ││   │
│   │  │ - DownloadQueue  │ │ - Preferences    │ │ - Permission     ││   │
│   │  │ - DownloadMgr    │ │ - ConfigSchema   │ │ - NotifyManager  ││   │
│   │  └──────────────────┘ └──────────────────┘ └──────────────────┘│   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                    已存在的 Crates                                │   │
│   │  ┌──────────────────┐  ┌──────────────────┐                     │   │
│   │  │auroraview-core   │  │auroraview-       │                     │   │
│   │  │                  │  │  plugins         │                     │   │
│   │  │ - Assets         │  │ - fs, clipboard  │                     │   │
│   │  │ - JS injection   │  │ - dialog, shell  │                     │   │
│   │  └──────────────────┘  └──────────────────┘                     │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                    ↑                                    │
│                                    │ 按需依赖                            │
│            ┌───────────────────────┴────────────────────┐               │
│            │                                            │               │
│            ↓                                            ↓               │
│   ┌─────────────────────────┐           ┌───────────────────────────┐   │
│   │        WebView          │           │         Browser           │   │
│   │    (单窗口 WebView)      │           │   (多 Tab 浏览器容器)      │   │
│   │                         │           │                           │   │
│   │  可选组合 features:      │           │   组合模式:                │   │
│   │  - tabs (单实例)        │           │   ┌─────────────────────┐ │   │
│   │  - extensions           │           │   │ Controller(WebView) │ │   │
│   │  - bookmarks            │           │   │ 浏览器 UI 控制器      │ │   │
│   │  - history              │           │   └─────────────────────┘ │   │
│   │  - devtools             │           │            +              │   │
│   │  - downloads            │           │   ┌─────────────────────┐ │   │
│   │  - settings             │           │   │ TabManager          │ │   │
│   │  - notifications        │           │   │ 管理多个 WebView     │ │   │
│   │  - plugins              │           │   └─────────────────────┘ │   │
│   │                         │           │            +              │   │
│   └─────────────────────────┘           │   ┌─────────────────────┐ │   │
│                                         │   │ Shared Features     │ │   │
│                                         │   │ 所有 Tab 共享        │ │   │
│                                         │   └─────────────────────┘ │   │
│                                         └───────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2. Crate 结构

#### 2.1 独立 Feature Crates

```
crates/
├── auroraview-core/          # 现有：核心资源、JS 注入
├── auroraview-plugins/       # 现有：fs, clipboard, dialog, shell...
│
├── # ===== Core Features (从 Browser 拆分) =====
│
├── auroraview-tabs/          # 新建：标签页管理
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── tab.rs            # Tab, TabId, TabState
│       ├── manager.rs        # TabManager
│       ├── group.rs          # TabGroup (标签组)
│       └── session.rs        # 会话恢复
│
├── auroraview-extensions/    # 新建：扩展系统
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── extension.rs      # Extension trait
│       ├── registry.rs       # ExtensionRegistry
│       ├── manifest.rs       # ExtensionManifest
│       └── chrome/           # Chrome Extension 兼容层
│           ├── mod.rs
│           └── bridge.rs
│
├── auroraview-bookmarks/     # 新建：书签管理
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── bookmark.rs       # Bookmark, BookmarkFolder
│       ├── manager.rs        # BookmarkManager
│       └── storage.rs        # 持久化
│
├── auroraview-history/       # 新建：历史记录
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── entry.rs          # HistoryEntry
│       ├── manager.rs        # HistoryManager
│       └── search.rs         # 搜索功能
│
├── auroraview-devtools/      # 新建：DevTools
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── manager.rs        # DevToolsManager
│       ├── config.rs         # DevToolsConfig
│       └── cdp/              # Chrome DevTools Protocol
│           ├── mod.rs
│           ├── server.rs
│           └── protocol.rs
│
├── # ===== Optional Features (新增能力) =====
│
├── auroraview-downloads/     # 新建：下载管理
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── item.rs           # DownloadItem
│       ├── queue.rs          # DownloadQueue
│       ├── manager.rs        # DownloadManager
│       └── storage.rs        # 下载历史持久化
│
├── auroraview-settings/      # 新建：设置管理
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── store.rs          # SettingsStore
│       ├── preferences.rs    # UserPreferences
│       ├── schema.rs         # ConfigSchema
│       └── migration.rs      # 配置迁移
│
├── auroraview-notifications/ # 新建：通知管理
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── notification.rs   # Notification
│       ├── permission.rs     # NotificationPermission
│       ├── manager.rs        # NotificationManager
│       └── history.rs        # 通知历史
│
└── auroraview-browser/       # 精简：组合 Controller + TabManager + Features
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── browser.rs        # Browser = Controller + TabManager + Features
        ├── config.rs
        └── ui/
            ├── mod.rs
            ├── controller.rs # Controller WebView
            └── theme.rs
```

#### 2.2 依赖关系

```toml
# crates/auroraview-tabs/Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
uuid = { version = "1", features = ["v4"] }
# 无其他 auroraview 依赖，完全独立

# crates/auroraview-extensions/Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
# 无其他 auroraview 依赖，完全独立

# crates/auroraview-bookmarks/Cargo.toml  
[dependencies]
serde = { version = "1", features = ["derive"] }
chrono = "0.4"
# 无其他 auroraview 依赖，完全独立

# crates/auroraview-downloads/Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["fs", "io-util"] }
# 无其他 auroraview 依赖，完全独立

# crates/auroraview-settings/Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
# 无其他 auroraview 依赖，完全独立

# crates/auroraview-notifications/Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
chrono = "0.4"
# 无其他 auroraview 依赖，完全独立

# crates/auroraview-browser/Cargo.toml
[dependencies]
auroraview-core = { path = "../auroraview-core" }

# Core features (默认启用)
auroraview-tabs = { path = "../auroraview-tabs", optional = true }
auroraview-extensions = { path = "../auroraview-extensions", optional = true }
auroraview-bookmarks = { path = "../auroraview-bookmarks", optional = true }
auroraview-history = { path = "../auroraview-history", optional = true }
auroraview-devtools = { path = "../auroraview-devtools", optional = true }

# Optional features
auroraview-downloads = { path = "../auroraview-downloads", optional = true }
auroraview-settings = { path = "../auroraview-settings", optional = true }
auroraview-notifications = { path = "../auroraview-notifications", optional = true }

[features]
default = ["tabs", "extensions", "bookmarks", "history", "devtools"]

# Core features
tabs = ["dep:auroraview-tabs"]
extensions = ["dep:auroraview-extensions"]
bookmarks = ["dep:auroraview-bookmarks"]
history = ["dep:auroraview-history"]
devtools = ["dep:auroraview-devtools"]

# Optional features
downloads = ["dep:auroraview-downloads"]
settings = ["dep:auroraview-settings"]
notifications = ["dep:auroraview-notifications"]

# Feature bundles
browser-full = ["tabs", "extensions", "bookmarks", "history", "devtools", 
                "downloads", "settings", "notifications"]
```

### 3. Browser 组合模式实现

```rust
// crates/auroraview-browser/src/browser.rs

use auroraview::WebView;

/// Browser = Controller(WebView) + TabManager + Shared Features
pub struct Browser {
    config: BrowserConfig,
    
    /// 浏览器 UI 控制器 - 也是一个 WebView
    controller: WebView,
    
    /// Tab 管理器 (使用独立 crate)
    #[cfg(feature = "tabs")]
    tabs: auroraview_tabs::TabManager,
    
    /// 共享的 Features (所有 Tab 共用)
    #[cfg(feature = "extensions")]
    extensions: auroraview_extensions::ExtensionRegistry,
    
    #[cfg(feature = "bookmarks")]
    bookmarks: auroraview_bookmarks::BookmarkManager,
    
    #[cfg(feature = "history")]
    history: auroraview_history::HistoryManager,
    
    #[cfg(feature = "devtools")]
    devtools: auroraview_devtools::DevToolsManager,
    
    // Optional features
    #[cfg(feature = "downloads")]
    downloads: auroraview_downloads::DownloadManager,
    
    #[cfg(feature = "settings")]
    settings: auroraview_settings::SettingsStore,
    
    #[cfg(feature = "notifications")]
    notifications: auroraview_notifications::NotificationManager,
}

impl Browser {
    pub fn new(config: BrowserConfig) -> Self {
        // Controller 也是一个 WebView，加载浏览器 UI
        let controller = WebView::builder()
            .title(&config.title)
            .html(get_controller_html())
            .build()
            .unwrap();
        
        Self {
            controller,
            #[cfg(feature = "tabs")]
            tabs: auroraview_tabs::TabManager::new(),
            #[cfg(feature = "extensions")]
            extensions: ExtensionRegistry::new(),
            #[cfg(feature = "bookmarks")]
            bookmarks: BookmarkManager::new(config.user_data_dir.as_deref()),
            #[cfg(feature = "history")]
            history: HistoryManager::new(config.user_data_dir.as_deref()),
            #[cfg(feature = "devtools")]
            devtools: DevToolsManager::new(config.devtools.clone()),
            #[cfg(feature = "downloads")]
            downloads: DownloadManager::new(config.download_dir.as_deref()),
            #[cfg(feature = "settings")]
            settings: SettingsStore::new(config.user_data_dir.as_deref()),
            #[cfg(feature = "notifications")]
            notifications: NotificationManager::new(),
            config,
        }
    }
    
    /// 创建新 Tab - 使用 TabManager
    #[cfg(feature = "tabs")]
    pub fn new_tab(&mut self, url: &str) -> auroraview_tabs::TabId {
        self.tabs.create(url)
    }
    
    /// 获取 Tab 管理器
    #[cfg(feature = "tabs")]
    pub fn tabs(&self) -> &auroraview_tabs::TabManager {
        &self.tabs
    }
    
    /// 获取下载管理器
    #[cfg(feature = "downloads")]
    pub fn downloads(&self) -> &auroraview_downloads::DownloadManager {
        &self.downloads
    }
    
    /// 获取设置
    #[cfg(feature = "settings")]
    pub fn settings(&self) -> &auroraview_settings::SettingsStore {
        &self.settings
    }
}
```

### 3.1 TabManager (独立 crate)

```rust
// crates/auroraview-tabs/src/manager.rs

use crate::{Tab, TabId, TabState, TabGroup};

/// 标签页管理器 - 不依赖具体 WebView 实现
pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab: Option<TabId>,
    groups: Vec<TabGroup>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            groups: Vec::new(),
        }
    }
    
    /// 创建新标签页 (返回 ID，具体 WebView 创建由上层处理)
    pub fn create(&mut self, url: &str) -> TabId {
        let id = TabId::new();
        let tab = Tab {
            id,
            url: url.to_string(),
            title: String::new(),
            state: TabState::Loading,
            group_id: None,
        };
        self.tabs.push(tab);
        self.active_tab = Some(id);
        id
    }
    
    /// 获取标签页
    pub fn get(&self, id: TabId) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == id)
    }
    
    /// 关闭标签页
    pub fn close(&mut self, id: TabId) -> bool {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(pos);
            if self.active_tab == Some(id) {
                self.active_tab = self.tabs.first().map(|t| t.id);
            }
            true
        } else {
            false
        }
    }
    
    /// 创建标签组
    pub fn create_group(&mut self, name: &str, tab_ids: Vec<TabId>) -> Option<TabGroup> {
        // ...
    }
}
```

### 4. WebView 组合 Features

```rust
// src/webview/mod.rs

pub struct WebView {
    // ... 现有字段 ...
    
    /// 可选的 Features
    features: Features,
}

/// Features 聚合 (按需启用)
pub struct Features {
    #[cfg(feature = "tabs")]
    pub tabs: Option<auroraview_tabs::TabManager>,
    
    #[cfg(feature = "extensions")]
    pub extensions: Option<auroraview_extensions::ExtensionRegistry>,
    
    #[cfg(feature = "bookmarks")]
    pub bookmarks: Option<auroraview_bookmarks::BookmarkManager>,
    
    #[cfg(feature = "history")]
    pub history: Option<auroraview_history::HistoryManager>,
    
    #[cfg(feature = "devtools")]
    pub devtools: Option<auroraview_devtools::DevToolsManager>,
    
    #[cfg(feature = "downloads")]
    pub downloads: Option<auroraview_downloads::DownloadManager>,
    
    #[cfg(feature = "settings")]
    pub settings: Option<auroraview_settings::SettingsStore>,
    
    #[cfg(feature = "notifications")]
    pub notifications: Option<auroraview_notifications::NotificationManager>,
}

impl WebView {
    /// 启用扩展系统
    #[cfg(feature = "extensions")]
    pub fn with_extensions(mut self) -> Self {
        self.features.extensions = Some(ExtensionRegistry::new());
        self
    }
    
    /// 启用书签
    #[cfg(feature = "bookmarks")]
    pub fn with_bookmarks(mut self, data_dir: Option<&Path>) -> Self {
        self.features.bookmarks = Some(BookmarkManager::new(data_dir));
        self
    }
    
    /// 启用下载管理
    #[cfg(feature = "downloads")]
    pub fn with_downloads(mut self, download_dir: Option<&Path>) -> Self {
        self.features.downloads = Some(DownloadManager::new(download_dir));
        self
    }
    
    /// 启用设置管理
    #[cfg(feature = "settings")]
    pub fn with_settings(mut self, data_dir: Option<&Path>) -> Self {
        self.features.settings = Some(SettingsStore::new(data_dir));
        self
    }
    
    /// 启用通知
    #[cfg(feature = "notifications")]
    pub fn with_notifications(mut self) -> Self {
        self.features.notifications = Some(NotificationManager::new());
        self
    }
    
    // ... 其他 features
}
```

### 4.1 下载管理器 (独立 crate)

```rust
// crates/auroraview-downloads/src/lib.rs

pub mod item;
pub mod queue;
pub mod manager;
pub mod storage;

pub use item::{DownloadItem, DownloadState, DownloadId};
pub use queue::DownloadQueue;
pub use manager::DownloadManager;

// crates/auroraview-downloads/src/item.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: DownloadId,
    pub url: String,
    pub filename: String,
    pub save_path: PathBuf,
    pub total_bytes: Option<u64>,
    pub received_bytes: u64,
    pub state: DownloadState,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadState {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

// crates/auroraview-downloads/src/manager.rs

pub struct DownloadManager {
    queue: DownloadQueue,
    download_dir: PathBuf,
    history: Vec<DownloadItem>,
    max_concurrent: usize,
}

impl DownloadManager {
    pub fn new(download_dir: Option<&Path>) -> Self { ... }
    
    /// 添加下载任务
    pub fn download(&mut self, url: &str, filename: Option<&str>) -> DownloadId { ... }
    
    /// 暂停下载
    pub fn pause(&mut self, id: DownloadId) -> bool { ... }
    
    /// 恢复下载
    pub fn resume(&mut self, id: DownloadId) -> bool { ... }
    
    /// 取消下载
    pub fn cancel(&mut self, id: DownloadId) -> bool { ... }
    
    /// 获取下载进度
    pub fn progress(&self, id: DownloadId) -> Option<f64> { ... }
    
    /// 获取所有下载
    pub fn list(&self) -> &[DownloadItem] { ... }
    
    /// 清除已完成的下载
    pub fn clear_completed(&mut self) { ... }
}
```

### 4.2 设置管理器 (独立 crate)

```rust
// crates/auroraview-settings/src/lib.rs

pub mod store;
pub mod preferences;
pub mod schema;
pub mod migration;

pub use store::SettingsStore;
pub use preferences::UserPreferences;
pub use schema::ConfigSchema;

// crates/auroraview-settings/src/store.rs

pub struct SettingsStore {
    path: PathBuf,
    data: HashMap<String, Value>,
    schema: Option<ConfigSchema>,
}

impl SettingsStore {
    pub fn new(data_dir: Option<&Path>) -> Self { ... }
    
    /// 获取设置值
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> { ... }
    
    /// 设置值
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), Error> { ... }
    
    /// 删除设置
    pub fn remove(&mut self, key: &str) -> bool { ... }
    
    /// 重置为默认值
    pub fn reset(&mut self) { ... }
    
    /// 保存到文件
    pub fn save(&self) -> Result<(), Error> { ... }
    
    /// 从文件加载
    pub fn load(&mut self) -> Result<(), Error> { ... }
    
    /// 导出设置
    pub fn export(&self) -> String { ... }
    
    /// 导入设置
    pub fn import(&mut self, data: &str) -> Result<(), Error> { ... }
}

// crates/auroraview-settings/src/preferences.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: Theme,
    pub language: String,
    pub font_size: u32,
    pub homepage: String,
    pub search_engine: String,
    pub enable_javascript: bool,
    pub enable_cookies: bool,
    pub block_popups: bool,
    // ...
}
```

### 4.3 通知管理器 (独立 crate)

```rust
// crates/auroraview-notifications/src/lib.rs

pub mod notification;
pub mod permission;
pub mod manager;
pub mod history;

pub use notification::{Notification, NotificationId};
pub use permission::{NotificationPermission, PermissionState};
pub use manager::NotificationManager;

// crates/auroraview-notifications/src/notification.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub tag: Option<String>,
    pub origin: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
}

// crates/auroraview-notifications/src/manager.rs

pub struct NotificationManager {
    permissions: HashMap<String, PermissionState>,
    history: Vec<Notification>,
    handlers: Vec<Box<dyn NotificationHandler>>,
}

impl NotificationManager {
    pub fn new() -> Self { ... }
    
    /// 请求通知权限
    pub fn request_permission(&mut self, origin: &str) -> PermissionState { ... }
    
    /// 检查权限
    pub fn check_permission(&self, origin: &str) -> PermissionState { ... }
    
    /// 显示通知
    pub fn show(&mut self, notification: Notification) -> Result<(), Error> { ... }
    
    /// 关闭通知
    pub fn close(&mut self, id: NotificationId) { ... }
    
    /// 获取通知历史
    pub fn history(&self) -> &[Notification] { ... }
    
    /// 标记为已读
    pub fn mark_read(&mut self, id: NotificationId) { ... }
    
    /// 清除所有通知
    pub fn clear_all(&mut self) { ... }
}
```

### 5. Python API 设计

```python
from auroraview import WebView

# 方式 1: 简单 WebView (无额外 features)
webview = WebView(title="Simple App")

# 方式 2: WebView + Features
webview = WebView(
    title="Advanced App",
    features={
        "extensions": True,
        "bookmarks": True,
        "history": True,
        "downloads": True,
        "settings": True,
        "notifications": True,
    },
)

# 或使用 builder 风格
webview = (WebView.builder()
    .title("My App")
    .with_extensions()
    .with_bookmarks()
    .with_downloads()
    .with_settings()
    .build())

# 使用 features
webview.extensions.register(my_extension)
webview.bookmarks.add("https://github.com", title="GitHub")
webview.history.get(limit=100)
webview.downloads.download("https://example.com/file.zip")
webview.settings.set("theme", "dark")
webview.notifications.show(title="Done", body="Download complete")

# Browser - 默认启用所有 features
from auroraview import Browser

browser = Browser(
    title="My Browser",
    home_url="https://google.com",
)

# Browser 的 Tab 管理
tab_id = browser.new_tab("https://example.com")
browser.tabs.switch(tab_id)
browser.tabs.close(tab_id)

# 标签组
group = browser.tabs.create_group("Work", [tab1, tab2])
browser.tabs.collapse_group(group.id)

# 共享的 features
browser.bookmarks.add("https://github.com", title="GitHub")
browser.extensions.register(ad_blocker)
browser.downloads.download("https://example.com/file.zip")
browser.settings.set("homepage", "https://google.com")
```

### 6. 迁移计划

#### Phase 1: 创建独立 Feature Crates (Week 1-2) ✅ Done

1. [x] 创建 `crates/auroraview-tabs/`
   - 已实现：`error.rs`, `event.rs`, `group.rs`, `lib.rs`, `manager.rs`, `session.rs`, `state.rs`
   - 包含 TabGroup 支持
   - 完全独立，无 auroraview 依赖

2. [x] 创建 `crates/auroraview-extensions/`
   - 已迁移到 `submodules/auroraview-extensions/`
   - 完全独立

3. [x] 创建 `crates/auroraview-bookmarks/`
   - 已实现：`bookmark.rs`, `error.rs`, `folder.rs`, `lib.rs`, `manager.rs`
   
4. [x] 创建 `crates/auroraview-history/`
   - 已实现：`entry.rs`, `error.rs`, `lib.rs`, `manager.rs`, `search.rs`

5. [x] 创建 `crates/auroraview-devtools/`
   - 已实现：`cdp.rs`, `config.rs`, `console.rs`, `error.rs`, `lib.rs`, `manager.rs`, `network.rs`

#### Phase 2: 创建可选 Feature Crates (Week 2-3) ✅ Done

1. [x] 创建 `crates/auroraview-downloads/`
   - 已实现：`error.rs`, `item.rs`, `lib.rs`, `manager.rs`, `queue.rs`
   - 包含下载队列、进度跟踪
   
2. [x] 创建 `crates/auroraview-settings/`
   - 已实现：`error.rs`, `lib.rs`, `manager.rs`, `schema.rs`, `store.rs`, `value.rs`
   - 包含配置存储、schema 验证

3. [x] 创建 `crates/auroraview-notifications/`
   - 已实现：`error.rs`, `lib.rs`, `manager.rs`, `notification.rs`, `permission.rs`
   - 包含权限管理

#### Phase 3: 重构 Browser (Week 3-4) ✅ Done

1. [x] 更新 `auroraview-browser` 依赖新 crates
   - Cargo.toml 已配置所有模块化依赖
   - Feature flags: `modular-tabs`, `modular-extensions`, `modular-bookmarks`, `modular-history`, `modular-devtools`
   - Optional features: `downloads`, `settings`, `notifications`
2. [x] 实现 Browser 组合模式 (Controller + TabManager + Features)
   - Browser struct 已组合：TabManager, BookmarkManager, HistoryManager, ExtensionRegistry, DevToolsManager
   - 通过 feature flags 可切换使用独立 crates 或内置实现
3. [x] 移除 Browser 中的重复代码
   - 通过 re-exports 统一接口

#### Phase 4: WebView 集成 Features (Week 4-5) ✅ Done

1. [x] 在 Rust WebView 中添加 `features` 字段
   - 实现位置：`src/webview/features.rs`
   - 创建 `Features` 聚合结构体
   - 创建 `FeaturesConfig` 用于配置和序列化
2. [x] 实现 `with_tabs()`、`with_bookmarks()`、`with_history()` 等方法
3. [x] 实现 `with_downloads()`、`with_settings()`、`with_notifications()`、`with_devtools()`
4. [x] Cargo feature flags 配置
   - `feature-tabs`, `feature-bookmarks`, `feature-history`, `feature-downloads`, `feature-settings`, `feature-notifications`, `feature-devtools`
   - Feature bundles: `features-core`, `features-all`
5. [ ] PyO3 绑定更新 (移到 Phase 5)

#### Phase 5: Python API 和文档 (Week 5-6) ✅ Done

1. [x] 创建 Python features API
   - 实现位置：`python/auroraview/features/`
   - 包含：`BookmarkManager`, `HistoryManager`, `DownloadManager`, `SettingsManager`, `NotificationManager`
   - 所有 managers 都有完整的 Python 类型提示
2. [x] 更新 Gallery 示例（通过 packed mode 和 AI Agent 集成）
3. [x] 更新文档（包含在 RFC 本身中）
4. [ ] PyO3 绑定（从 Rust features.rs 到 Python）- 未来优化

### 7. 依赖关系图 (最终)

```
                     ┌───────────────────────────────────────────────────────────────┐
                     │           Core Feature Crates (从 Browser 拆分)                 │
                     │                                                               │
                     │  ┌──────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐ │
                     │  │  tabs    │ │ extensions  │ │ bookmarks   │ │ history   │ │
                     │  └──────────┘ └─────────────┘ └─────────────┘ └───────────┘ │
                     │                                                               │
                     │  ┌─────────────┐ ┌─────────────┐                             │
                     │  │  devtools   │ │  plugins    │ (已存在)                     │
                     │  └─────────────┘ └─────────────┘                             │
                     └───────────────────────────────────────────────────────────────┘
                                              ↑
                     ┌───────────────────────────────────────────────────────────────┐
                     │           Optional Feature Crates (新增能力)                    │
                     │                                                               │
                     │  ┌─────────────┐ ┌─────────────┐ ┌───────────────────┐       │
                     │  │ downloads   │ │  settings   │ │   notifications   │       │
                     │  └─────────────┘ └─────────────┘ └───────────────────┘       │
                     └───────────────────────────────────────────────────────────────┘
                                              ↑
                                              │ 按需组合
                     ┌────────────────────────┴────────────────────────┐
                     │                                                 │
                     ↓                                                 ↓
          ┌─────────────────────────┐                  ┌───────────────────────────┐
          │        WebView          │                  │         Browser           │
          │                         │                  │                           │
          │  核心 WebView 实现       │      组合        │  Controller: WebView      │
          │  + 可选 Features        │  ←─────────────  │  TabManager: 管理标签     │
          │                         │                  │  Features: 共享           │
          └─────────────────────────┘                  └───────────────────────────┘
```

### 8. API 兼容性

#### 8.1 向后兼容

- 现有 `WebView` API 完全不变
- 现有 `Browser` API 完全不变
- 新 features 都是 opt-in

#### 8.2 Cargo Features

```toml
# 主 crate Cargo.toml
[features]
default = []

# Core features (从 Browser 拆分)
tabs = ["dep:auroraview-tabs"]
extensions = ["dep:auroraview-extensions"]
bookmarks = ["dep:auroraview-bookmarks"]
history = ["dep:auroraview-history"]
devtools = ["dep:auroraview-devtools"]

# Optional features (新增能力)
downloads = ["dep:auroraview-downloads"]
settings = ["dep:auroraview-settings"]
notifications = ["dep:auroraview-notifications"]

# 便捷组合
browser-core = ["tabs", "extensions", "bookmarks", "history", "devtools"]
browser-full = ["browser-core", "downloads", "settings", "notifications"]
```

## Alternatives Considered

### 1. 单一 auroraview-features crate

**优点**：简单，一个 crate 包含所有  
**缺点**：
- 无法单独使用某个 feature
- 违反单一职责
- 编译时间长

### 2. Browser 不使用 WebView

**优点**：更灵活控制底层  
**缺点**：
- 代码重复
- 无法复用 WebView 的 Python API、事件系统
- 维护两套实现

### 3. WebView 继承 Browser 的能力

**优点**：代码复用  
**缺点**：
- 方向错误：Browser 应该组合 WebView
- 循环依赖风险

## Open Questions

1. **Feature 之间的依赖**：某些 features 是否依赖其他 feature？
   - 建议：保持独立，通过接口通信

2. **持久化路径**：各 feature 的数据存储位置？
   - 建议：每个 feature 接受 `data_dir` 参数，由上层决定

3. **事件通信**：Features 如何与 WebView/Browser 通信？
   - 建议：通过回调/事件系统

## Success Metrics

1. 每个 Feature crate 可以独立使用（无 auroraview 依赖）
2. Browser 由 WebView + TabManager 组合实现
3. TabManager 独立为 auroraview-tabs crate
4. Gallery 能使用 extensions/downloads/settings/notifications features
5. 现有 API 100% 向后兼容
6. 可选 features 不影响核心 WebView 编译大小

## References

- [RFC 0006: AuroraView Browser Crate](./0006-auroraview-browser-crate.md)
- [RFC 0005: DCC Plugin Architecture](./0005-dcc-plugin-architecture.md)
- [Tauri Plugin System](https://tauri.app/v1/guides/features/plugin/)
- [Bevy ECS - 组合优于继承](https://bevyengine.org/learn/book/getting-started/ecs/)
