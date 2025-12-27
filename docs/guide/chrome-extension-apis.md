# Chrome Extension API Compatibility

AuroraView provides a comprehensive polyfill layer for Chrome Extension APIs, enabling you to run Chrome extensions or use familiar Chrome APIs in your WebView applications.

## Overview

The Chrome Extension API compatibility layer allows developers to:

- **Migrate existing Chrome extensions** to AuroraView with minimal code changes
- **Use familiar APIs** like `chrome.storage`, `chrome.notifications`, etc.
- **Build cross-platform applications** that work both as Chrome extensions and standalone apps
- **Leverage WXT framework** for modern extension development

## Supported APIs

### Core APIs (Full Support)

| API | Description | Key Methods |
|-----|-------------|-------------|
| `chrome.runtime` | Extension lifecycle and messaging | `sendMessage()`, `getManifest()`, `getURL()` |
| `chrome.storage` | Persistent data storage | `local.get/set()`, `sync.get/set()`, `session.get/set()` |
| `chrome.tabs` | Tab management | `query()`, `create()`, `update()`, `remove()` |
| `chrome.action` | Toolbar button control | `setIcon()`, `setBadgeText()`, `setPopup()` |
| `chrome.notifications` | System notifications | `create()`, `update()`, `clear()` |
| `chrome.alarms` | Scheduled tasks | `create()`, `get()`, `clear()` |
| `chrome.contextMenus` | Right-click menus | `create()`, `update()`, `remove()` |

### Data & Privacy APIs

| API | Description | Key Methods |
|-----|-------------|-------------|
| `chrome.bookmarks` | Bookmark management | `create()`, `get()`, `search()`, `remove()` |
| `chrome.history` | Browsing history | `search()`, `addUrl()`, `deleteUrl()` |
| `chrome.cookies` | Cookie management | `get()`, `set()`, `remove()`, `getAll()` |
| `chrome.downloads` | Download management | `download()`, `pause()`, `resume()`, `cancel()` |
| `chrome.browsingData` | Clear browsing data | `remove()`, `removeCache()`, `removeCookies()` |

### UI & Interaction APIs

| API | Description | Key Methods |
|-----|-------------|-------------|
| `chrome.sidePanel` | Side panel control | `open()`, `setOptions()`, `getOptions()` |
| `chrome.omnibox` | Address bar integration | `setDefaultSuggestion()`, `onInputChanged` |
| `chrome.search` | Search functionality | `query()` |
| `chrome.fontSettings` | Font customization | `getFont()`, `setFont()`, `getFontList()` |

### System APIs

| API | Description | Key Methods |
|-----|-------------|-------------|
| `chrome.idle` | User activity detection | `queryState()`, `setDetectionInterval()` |
| `chrome.power` | Power management | `requestKeepAwake()`, `releaseKeepAwake()` |
| `chrome.tts` | Text-to-speech | `speak()`, `stop()`, `getVoices()` |
| `chrome.topSites` | Most visited sites | `get()` |

### Extension Management APIs

| API | Description | Key Methods |
|-----|-------------|-------------|
| `chrome.management` | Extension control | `getAll()`, `get()`, `setEnabled()` |
| `chrome.sessions` | Session management | `getRecentlyClosed()`, `restore()` |
| `chrome.tabGroups` | Tab group management | `get()`, `query()`, `update()` |
| `chrome.permissions` | Permission requests | `request()`, `contains()`, `remove()` |

### Network APIs

| API | Description | Key Methods |
|-----|-------------|-------------|
| `chrome.webRequest` | Request interception | `onBeforeRequest`, `onCompleted` |
| `chrome.scripting` | Script injection | `executeScript()`, `insertCSS()` |
| `chrome.declarativeNetRequest` | Declarative request rules | `updateDynamicRules()` |

## Quick Start

### Basic Usage

```javascript
// Wait for the bridge to be ready
window.addEventListener('auroraviewready', async () => {
    // Storage API
    await chrome.storage.local.set({ key: 'value' });
    const data = await chrome.storage.local.get('key');
    console.log(data.key); // 'value'
    
    // Notifications API
    await chrome.notifications.create('my-notification', {
        type: 'basic',
        title: 'Hello',
        message: 'World',
        iconUrl: 'icon.png'
    });
    
    // Alarms API
    await chrome.alarms.create('my-alarm', { delayInMinutes: 1 });
    chrome.alarms.onAlarm.addListener((alarm) => {
        console.log('Alarm fired:', alarm.name);
    });
});
```

### Storage API Example

```javascript
// Local storage (persists across sessions)
await chrome.storage.local.set({
    settings: { theme: 'dark', language: 'en' },
    userData: { name: 'John', preferences: [] }
});

const { settings } = await chrome.storage.local.get('settings');
console.log(settings.theme); // 'dark'

// Listen for changes
chrome.storage.onChanged.addListener((changes, areaName) => {
    for (const [key, { oldValue, newValue }] of Object.entries(changes)) {
        console.log(`${key} changed from ${oldValue} to ${newValue}`);
    }
});
```

### Bookmarks API Example

```javascript
// Create a bookmark
const bookmark = await chrome.bookmarks.create({
    parentId: '1', // Bookmarks bar
    title: 'My Website',
    url: 'https://example.com'
});

// Search bookmarks
const results = await chrome.bookmarks.search({ query: 'example' });

// Get bookmark tree
const tree = await chrome.bookmarks.getTree();

// Listen for bookmark events
chrome.bookmarks.onCreated.addListener((id, bookmark) => {
    console.log('New bookmark:', bookmark.title);
});
```

### Downloads API Example

```javascript
// Start a download
const downloadId = await chrome.downloads.download({
    url: 'https://example.com/file.pdf',
    filename: 'my-file.pdf',
    saveAs: true
});

// Monitor download progress
chrome.downloads.onChanged.addListener((delta) => {
    if (delta.state?.current === 'complete') {
        console.log('Download completed!');
    }
});

// Search downloads
const downloads = await chrome.downloads.search({
    state: 'complete',
    limit: 10
});

// Pause/Resume/Cancel
await chrome.downloads.pause(downloadId);
await chrome.downloads.resume(downloadId);
await chrome.downloads.cancel(downloadId);
```

### History API Example

```javascript
// Search history
const historyItems = await chrome.history.search({
    text: 'github',
    startTime: Date.now() - 7 * 24 * 60 * 60 * 1000, // Last 7 days
    maxResults: 100
});

// Add URL to history
await chrome.history.addUrl({ url: 'https://example.com' });

// Delete specific URL
await chrome.history.deleteUrl({ url: 'https://example.com' });

// Clear all history
await chrome.history.deleteAll();
```

### Cookies API Example

```javascript
// Get a specific cookie
const cookie = await chrome.cookies.get({
    url: 'https://example.com',
    name: 'session_id'
});

// Set a cookie
await chrome.cookies.set({
    url: 'https://example.com',
    name: 'user_pref',
    value: 'dark_mode',
    expirationDate: Date.now() / 1000 + 86400 // 1 day
});

// Get all cookies for a domain
const cookies = await chrome.cookies.getAll({ domain: 'example.com' });

// Remove a cookie
await chrome.cookies.remove({
    url: 'https://example.com',
    name: 'session_id'
});
```

### TTS (Text-to-Speech) Example

```javascript
// Speak text
chrome.tts.speak('Hello, world!', {
    rate: 1.0,
    pitch: 1.0,
    volume: 1.0,
    onEvent: (event) => {
        if (event.type === 'end') {
            console.log('Speech finished');
        }
    }
});

// Get available voices
const voices = await chrome.tts.getVoices();
console.log('Available voices:', voices);

// Stop speaking
chrome.tts.stop();
```

### Idle Detection Example

```javascript
// Query current idle state
const state = await chrome.idle.queryState(60); // 60 seconds threshold
console.log('User state:', state); // 'active', 'idle', or 'locked'

// Set detection interval
chrome.idle.setDetectionInterval(30);

// Listen for state changes
chrome.idle.onStateChanged.addListener((newState) => {
    console.log('Idle state changed to:', newState);
});
```

### Power Management Example

```javascript
// Prevent system from sleeping
chrome.power.requestKeepAwake('display'); // Keep display on
// or
chrome.power.requestKeepAwake('system'); // Prevent sleep

// Allow system to sleep again
chrome.power.releaseKeepAwake();
```

## Best Practices

### 1. Always Wait for Ready Event

```javascript
// GOOD: Wait for the bridge to be ready
window.addEventListener('auroraviewready', () => {
    // Safe to use chrome.* APIs here
});

// BAD: Using APIs immediately may fail
chrome.storage.local.get('key'); // May throw error
```

### 2. Handle Errors Gracefully

```javascript
try {
    const result = await chrome.storage.local.get('key');
    // Process result
} catch (error) {
    console.error('Storage error:', error.message);
    // Fallback behavior
}
```

### 3. Use Event Listeners Properly

```javascript
// Add listener
const handler = (changes) => {
    console.log('Changes:', changes);
};
chrome.storage.onChanged.addListener(handler);

// Remove listener when no longer needed
chrome.storage.onChanged.removeListener(handler);
```

### 4. Batch Storage Operations

```javascript
// GOOD: Single operation for multiple keys
await chrome.storage.local.set({
    key1: 'value1',
    key2: 'value2',
    key3: 'value3'
});

// BAD: Multiple operations
await chrome.storage.local.set({ key1: 'value1' });
await chrome.storage.local.set({ key2: 'value2' });
await chrome.storage.local.set({ key3: 'value3' });
```

### 5. Clean Up Resources

```javascript
// Clean up alarms when no longer needed
await chrome.alarms.clear('my-alarm');

// Clear notifications
await chrome.notifications.clear('my-notification');

// Remove context menus
await chrome.contextMenus.removeAll();
```

## Limitations and Differences

### Single-Tab Mode

AuroraView operates in **single-tab mode** by default. This means:

- `chrome.tabs.query()` returns only the current tab
- `chrome.tabs.create()` navigates the current tab instead of opening a new one
- Tab IDs are simplified (typically just `1`)

```javascript
// In AuroraView, this returns a single-item array
const tabs = await chrome.tabs.query({ active: true });
// tabs = [{ id: 1, url: '...', title: '...' }]
```

### Storage Limitations

| Storage Area | Chrome Limit | AuroraView |
|--------------|--------------|------------|
| `local` | 10 MB | Unlimited (filesystem-based) |
| `sync` | 100 KB | Same as local (no sync) |
| `session` | 10 MB | Memory-based, cleared on close |

::: warning Sync Storage
`chrome.storage.sync` is available but does not actually sync across devices. It functions identically to `local` storage.
:::

### Permissions

Unlike Chrome extensions, AuroraView doesn't require manifest permissions. All APIs are available by default:

```javascript
// No need for "permissions": ["storage", "notifications"] in manifest
// Just use the APIs directly
await chrome.storage.local.set({ key: 'value' });
await chrome.notifications.create('id', { ... });
```

### Network APIs

- `chrome.webRequest` provides basic interception but cannot modify requests
- `chrome.declarativeNetRequest` has limited rule support

### Platform-Specific Features

Some APIs behave differently based on the platform:

| API | Windows | macOS | Linux |
|-----|---------|-------|-------|
| `chrome.tts` | Full | Full | Requires espeak |
| `chrome.power` | Full | Full | Limited |
| `chrome.idle` | Full | Full | Full |

## Migration from Chrome Extension

### Step 1: Remove Manifest Permissions

```diff
// manifest.json (no longer needed)
- {
-   "permissions": ["storage", "notifications", "alarms"]
- }
```

### Step 2: Update Background Script

```javascript
// Chrome extension background.js
chrome.runtime.onInstalled.addListener(() => {
    // Setup code
});

// AuroraView - wrap in ready event
window.addEventListener('auroraviewready', () => {
    // Same setup code works here
});
```

### Step 3: Handle Tab Differences

```javascript
// Chrome extension - opens new tab
chrome.tabs.create({ url: 'https://example.com' });

// AuroraView - navigates current view
// Option 1: Use the same API (navigates current tab)
chrome.tabs.create({ url: 'https://example.com' });

// Option 2: Use AuroraView's native navigation
auroraview.call('navigate', { url: 'https://example.com' });
```

## WXT Framework Compatibility

AuroraView is compatible with the [WXT framework](https://wxt.dev/) for modern extension development:

```javascript
// wxt.config.ts
export default defineConfig({
    // WXT configuration works with AuroraView
    manifest: {
        name: 'My Extension',
        version: '1.0.0'
    }
});
```

```javascript
// entrypoints/background.ts
export default defineBackground(() => {
    // WXT background script works in AuroraView
    chrome.storage.local.set({ initialized: true });
});
```

## API Reference

For detailed API documentation, refer to:

- [Chrome Extension API Reference](https://developer.chrome.com/docs/extensions/reference/api)
- [AuroraView API Documentation](/api/)

## Troubleshooting

### API Not Available

```javascript
// Check if API is available
if (typeof chrome !== 'undefined' && chrome.storage) {
    // API is available
} else {
    console.warn('Chrome APIs not available');
}
```

### Events Not Firing

```javascript
// Ensure you're adding listeners before the event occurs
window.addEventListener('auroraviewready', () => {
    // Add listeners here
    chrome.storage.onChanged.addListener(handler);
    
    // Then perform operations that trigger events
    chrome.storage.local.set({ key: 'value' });
});
```

### Storage Data Not Persisting

```javascript
// Check if storage is working
await chrome.storage.local.set({ test: 'value' });
const result = await chrome.storage.local.get('test');
console.log('Storage test:', result.test === 'value' ? 'OK' : 'Failed');
```
