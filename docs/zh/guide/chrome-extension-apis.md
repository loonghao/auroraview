# Chrome 扩展 API 兼容性

AuroraView 提供了完整的 Chrome Extension API polyfill 层，让你可以在 WebView 应用中运行 Chrome 扩展或使用熟悉的 Chrome API。

## 概述

Chrome Extension API 兼容层允许开发者：

- **迁移现有 Chrome 扩展**到 AuroraView，只需最小的代码改动
- **使用熟悉的 API**，如 `chrome.storage`、`chrome.notifications` 等
- **构建跨平台应用**，既可以作为 Chrome 扩展也可以作为独立应用运行
- **利用 WXT 框架**进行现代化扩展开发

## 支持的 API

### 核心 API（完整支持）

| API | 描述 | 主要方法 |
|-----|------|----------|
| `chrome.runtime` | 扩展生命周期和消息传递 | `sendMessage()`, `getManifest()`, `getURL()` |
| `chrome.storage` | 持久化数据存储 | `local.get/set()`, `sync.get/set()`, `session.get/set()` |
| `chrome.tabs` | 标签页管理 | `query()`, `create()`, `update()`, `remove()` |
| `chrome.action` | 工具栏按钮控制 | `setIcon()`, `setBadgeText()`, `setPopup()` |
| `chrome.notifications` | 系统通知 | `create()`, `update()`, `clear()` |
| `chrome.alarms` | 定时任务 | `create()`, `get()`, `clear()` |
| `chrome.contextMenus` | 右键菜单 | `create()`, `update()`, `remove()` |

### 数据与隐私 API

| API | 描述 | 主要方法 |
|-----|------|----------|
| `chrome.bookmarks` | 书签管理 | `create()`, `get()`, `search()`, `remove()` |
| `chrome.history` | 浏览历史 | `search()`, `addUrl()`, `deleteUrl()` |
| `chrome.cookies` | Cookie 管理 | `get()`, `set()`, `remove()`, `getAll()` |
| `chrome.downloads` | 下载管理 | `download()`, `pause()`, `resume()`, `cancel()` |
| `chrome.browsingData` | 清除浏览数据 | `remove()`, `removeCache()`, `removeCookies()` |

### UI 与交互 API

| API | 描述 | 主要方法 |
|-----|------|----------|
| `chrome.sidePanel` | 侧边栏控制 | `open()`, `setOptions()`, `getOptions()` |
| `chrome.omnibox` | 地址栏集成 | `setDefaultSuggestion()`, `onInputChanged` |
| `chrome.search` | 搜索功能 | `query()` |
| `chrome.fontSettings` | 字体自定义 | `getFont()`, `setFont()`, `getFontList()` |

### 系统 API

| API | 描述 | 主要方法 |
|-----|------|----------|
| `chrome.idle` | 用户活动检测 | `queryState()`, `setDetectionInterval()` |
| `chrome.power` | 电源管理 | `requestKeepAwake()`, `releaseKeepAwake()` |
| `chrome.tts` | 文字转语音 | `speak()`, `stop()`, `getVoices()` |
| `chrome.topSites` | 最常访问网站 | `get()` |

### 扩展管理 API

| API | 描述 | 主要方法 |
|-----|------|----------|
| `chrome.management` | 扩展控制 | `getAll()`, `get()`, `setEnabled()` |
| `chrome.sessions` | 会话管理 | `getRecentlyClosed()`, `restore()` |
| `chrome.tabGroups` | 标签组管理 | `get()`, `query()`, `update()` |
| `chrome.permissions` | 权限请求 | `request()`, `contains()`, `remove()` |

### 网络 API

| API | 描述 | 主要方法 |
|-----|------|----------|
| `chrome.webRequest` | 请求拦截 | `onBeforeRequest`, `onCompleted` |
| `chrome.scripting` | 脚本注入 | `executeScript()`, `insertCSS()` |
| `chrome.declarativeNetRequest` | 声明式请求规则 | `updateDynamicRules()` |

## 快速开始

### 基本用法

```javascript
// 等待桥接就绪
window.addEventListener('auroraviewready', async () => {
    // Storage API
    await chrome.storage.local.set({ key: 'value' });
    const data = await chrome.storage.local.get('key');
    console.log(data.key); // 'value'
    
    // Notifications API
    await chrome.notifications.create('my-notification', {
        type: 'basic',
        title: '你好',
        message: '世界',
        iconUrl: 'icon.png'
    });
    
    // Alarms API
    await chrome.alarms.create('my-alarm', { delayInMinutes: 1 });
    chrome.alarms.onAlarm.addListener((alarm) => {
        console.log('闹钟触发:', alarm.name);
    });
});
```

### Storage API 示例

```javascript
// 本地存储（跨会话持久化）
await chrome.storage.local.set({
    settings: { theme: 'dark', language: 'zh' },
    userData: { name: '张三', preferences: [] }
});

const { settings } = await chrome.storage.local.get('settings');
console.log(settings.theme); // 'dark'

// 监听变化
chrome.storage.onChanged.addListener((changes, areaName) => {
    for (const [key, { oldValue, newValue }] of Object.entries(changes)) {
        console.log(`${key} 从 ${oldValue} 变为 ${newValue}`);
    }
});
```

### 书签 API 示例

```javascript
// 创建书签
const bookmark = await chrome.bookmarks.create({
    parentId: '1', // 书签栏
    title: '我的网站',
    url: 'https://example.com'
});

// 搜索书签
const results = await chrome.bookmarks.search({ query: 'example' });

// 获取书签树
const tree = await chrome.bookmarks.getTree();

// 监听书签事件
chrome.bookmarks.onCreated.addListener((id, bookmark) => {
    console.log('新书签:', bookmark.title);
});
```

### 下载 API 示例

```javascript
// 开始下载
const downloadId = await chrome.downloads.download({
    url: 'https://example.com/file.pdf',
    filename: 'my-file.pdf',
    saveAs: true
});

// 监控下载进度
chrome.downloads.onChanged.addListener((delta) => {
    if (delta.state?.current === 'complete') {
        console.log('下载完成！');
    }
});

// 搜索下载记录
const downloads = await chrome.downloads.search({
    state: 'complete',
    limit: 10
});

// 暂停/恢复/取消
await chrome.downloads.pause(downloadId);
await chrome.downloads.resume(downloadId);
await chrome.downloads.cancel(downloadId);
```

### 历史记录 API 示例

```javascript
// 搜索历史
const historyItems = await chrome.history.search({
    text: 'github',
    startTime: Date.now() - 7 * 24 * 60 * 60 * 1000, // 最近7天
    maxResults: 100
});

// 添加 URL 到历史
await chrome.history.addUrl({ url: 'https://example.com' });

// 删除特定 URL
await chrome.history.deleteUrl({ url: 'https://example.com' });

// 清除所有历史
await chrome.history.deleteAll();
```

### Cookie API 示例

```javascript
// 获取特定 cookie
const cookie = await chrome.cookies.get({
    url: 'https://example.com',
    name: 'session_id'
});

// 设置 cookie
await chrome.cookies.set({
    url: 'https://example.com',
    name: 'user_pref',
    value: 'dark_mode',
    expirationDate: Date.now() / 1000 + 86400 // 1天后过期
});

// 获取域名下所有 cookie
const cookies = await chrome.cookies.getAll({ domain: 'example.com' });

// 删除 cookie
await chrome.cookies.remove({
    url: 'https://example.com',
    name: 'session_id'
});
```

### TTS（文字转语音）示例

```javascript
// 朗读文本
chrome.tts.speak('你好，世界！', {
    rate: 1.0,
    pitch: 1.0,
    volume: 1.0,
    onEvent: (event) => {
        if (event.type === 'end') {
            console.log('朗读完成');
        }
    }
});

// 获取可用语音
const voices = await chrome.tts.getVoices();
console.log('可用语音:', voices);

// 停止朗读
chrome.tts.stop();
```

### 空闲检测示例

```javascript
// 查询当前空闲状态
const state = await chrome.idle.queryState(60); // 60秒阈值
console.log('用户状态:', state); // 'active', 'idle', 或 'locked'

// 设置检测间隔
chrome.idle.setDetectionInterval(30);

// 监听状态变化
chrome.idle.onStateChanged.addListener((newState) => {
    console.log('空闲状态变为:', newState);
});
```

### 电源管理示例

```javascript
// 阻止系统休眠
chrome.power.requestKeepAwake('display'); // 保持显示器开启
// 或
chrome.power.requestKeepAwake('system'); // 阻止休眠

// 允许系统休眠
chrome.power.releaseKeepAwake();
```

## 最佳实践

### 1. 始终等待 Ready 事件

```javascript
// 正确：等待桥接就绪
window.addEventListener('auroraviewready', () => {
    // 这里可以安全使用 chrome.* API
});

// 错误：立即使用 API 可能失败
chrome.storage.local.get('key'); // 可能抛出错误
```

### 2. 优雅地处理错误

```javascript
try {
    const result = await chrome.storage.local.get('key');
    // 处理结果
} catch (error) {
    console.error('存储错误:', error.message);
    // 降级处理
}
```

### 3. 正确使用事件监听器

```javascript
// 添加监听器
const handler = (changes) => {
    console.log('变化:', changes);
};
chrome.storage.onChanged.addListener(handler);

// 不再需要时移除监听器
chrome.storage.onChanged.removeListener(handler);
```

### 4. 批量存储操作

```javascript
// 正确：单次操作多个键
await chrome.storage.local.set({
    key1: 'value1',
    key2: 'value2',
    key3: 'value3'
});

// 错误：多次操作
await chrome.storage.local.set({ key1: 'value1' });
await chrome.storage.local.set({ key2: 'value2' });
await chrome.storage.local.set({ key3: 'value3' });
```

### 5. 清理资源

```javascript
// 不再需要时清理闹钟
await chrome.alarms.clear('my-alarm');

// 清除通知
await chrome.notifications.clear('my-notification');

// 移除右键菜单
await chrome.contextMenus.removeAll();
```

## 限制与差异

### 单标签页模式

AuroraView 默认运行在**单标签页模式**。这意味着：

- `chrome.tabs.query()` 只返回当前标签页
- `chrome.tabs.create()` 导航当前标签页而不是打开新标签
- 标签页 ID 简化（通常只有 `1`）

```javascript
// 在 AuroraView 中，这返回单元素数组
const tabs = await chrome.tabs.query({ active: true });
// tabs = [{ id: 1, url: '...', title: '...' }]
```

### 存储限制

| 存储区域 | Chrome 限制 | AuroraView |
|----------|-------------|------------|
| `local` | 10 MB | 无限制（基于文件系统） |
| `sync` | 100 KB | 与 local 相同（无同步） |
| `session` | 10 MB | 基于内存，关闭时清除 |

::: warning 同步存储
`chrome.storage.sync` 可用但不会真正跨设备同步。它的功能与 `local` 存储完全相同。
:::

### 权限

与 Chrome 扩展不同，AuroraView 不需要 manifest 权限声明。所有 API 默认可用：

```javascript
// 不需要在 manifest 中声明 "permissions": ["storage", "notifications"]
// 直接使用 API 即可
await chrome.storage.local.set({ key: 'value' });
await chrome.notifications.create('id', { ... });
```

### 网络 API

- `chrome.webRequest` 提供基本拦截但不能修改请求
- `chrome.declarativeNetRequest` 规则支持有限

### 平台特定功能

某些 API 在不同平台上表现不同：

| API | Windows | macOS | Linux |
|-----|---------|-------|-------|
| `chrome.tts` | 完整 | 完整 | 需要 espeak |
| `chrome.power` | 完整 | 完整 | 有限 |
| `chrome.idle` | 完整 | 完整 | 完整 |

## 从 Chrome 扩展迁移

### 步骤 1：移除 Manifest 权限

```diff
// manifest.json（不再需要）
- {
-   "permissions": ["storage", "notifications", "alarms"]
- }
```

### 步骤 2：更新后台脚本

```javascript
// Chrome 扩展 background.js
chrome.runtime.onInstalled.addListener(() => {
    // 初始化代码
});

// AuroraView - 包装在 ready 事件中
window.addEventListener('auroraviewready', () => {
    // 相同的初始化代码在这里也能工作
});
```

### 步骤 3：处理标签页差异

```javascript
// Chrome 扩展 - 打开新标签页
chrome.tabs.create({ url: 'https://example.com' });

// AuroraView - 导航当前视图
// 方式 1：使用相同 API（导航当前标签页）
chrome.tabs.create({ url: 'https://example.com' });

// 方式 2：使用 AuroraView 原生导航
auroraview.call('navigate', { url: 'https://example.com' });
```

## WXT 框架兼容性

AuroraView 与 [WXT 框架](https://wxt.dev/) 兼容，支持现代化扩展开发：

```javascript
// wxt.config.ts
export default defineConfig({
    // WXT 配置与 AuroraView 兼容
    manifest: {
        name: 'My Extension',
        version: '1.0.0'
    }
});
```

```javascript
// entrypoints/background.ts
export default defineBackground(() => {
    // WXT 后台脚本在 AuroraView 中正常工作
    chrome.storage.local.set({ initialized: true });
});
```

## API 参考

详细 API 文档请参考：

- [Chrome Extension API 参考](https://developer.chrome.com/docs/extensions/reference/api)
- [AuroraView API 文档](/zh/api/)

## 故障排除

### API 不可用

```javascript
// 检查 API 是否可用
if (typeof chrome !== 'undefined' && chrome.storage) {
    // API 可用
} else {
    console.warn('Chrome API 不可用');
}
```

### 事件不触发

```javascript
// 确保在事件发生前添加监听器
window.addEventListener('auroraviewready', () => {
    // 在这里添加监听器
    chrome.storage.onChanged.addListener(handler);
    
    // 然后执行触发事件的操作
    chrome.storage.local.set({ key: 'value' });
});
```

### 存储数据不持久化

```javascript
// 检查存储是否正常工作
await chrome.storage.local.set({ test: 'value' });
const result = await chrome.storage.local.get('test');
console.log('存储测试:', result.test === 'value' ? '正常' : '失败');
```
