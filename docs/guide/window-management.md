# Window Management API

The AuroraView SDK provides a comprehensive Window API for managing WebView windows from JavaScript.

## Overview

The Window API allows you to:
- Create, show, hide, and close windows
- Control window position, size, and state
- Query window information
- Handle window events
- Manage multiple windows

## JavaScript SDK

### Getting the Current Window

```typescript
import { Window, getCurrentWindow } from '@auroraview/sdk';

// Using static method
const current = Window.getCurrent();

// Using convenience function
const win = getCurrentWindow();
```

### Creating Windows

```typescript
import { Window, createWindow } from '@auroraview/sdk';

// Using static method
const win = await Window.create({
  label: 'settings',
  url: '/settings.html',
  title: 'Settings',
  width: 520,
  height: 650,
  center: true,
});

// Using convenience function
const win2 = await createWindow({
  label: 'preview',
  html: '<h1>Preview</h1>',
  width: 400,
  height: 300,
});
```

### Window Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `label` | `string` | auto | Unique window identifier |
| `url` | `string` | - | URL to load |
| `html` | `string` | - | HTML content (alternative to url) |
| `title` | `string` | "AuroraView" | Window title |
| `width` | `number` | 800 | Window width in pixels |
| `height` | `number` | 600 | Window height in pixels |
| `x` | `number` | - | X position |
| `y` | `number` | - | Y position |
| `center` | `boolean` | false | Center window on screen |
| `resizable` | `boolean` | true | Allow resizing |
| `frameless` | `boolean` | false | Hide window frame |
| `transparent` | `boolean` | false | Transparent background |
| `alwaysOnTop` | `boolean` | false | Keep window on top |
| `minimized` | `boolean` | false | Start minimized |
| `maximized` | `boolean` | false | Start maximized |
| `fullscreen` | `boolean` | false | Start fullscreen |
| `devtools` | `boolean` | false | Enable developer tools |

### Window Lifecycle

```typescript
// Show window
await win.show();

// Hide window
await win.hide();

// Close window
await win.close();

// Focus window (bring to front)
await win.focus();

// Close current window
import { closeCurrentWindow } from '@auroraview/sdk';
await closeCurrentWindow();
```

### Window State

```typescript
// Minimize
await win.minimize();

// Maximize
await win.maximize();

// Restore from minimized/maximized
await win.restore();

// Toggle fullscreen
await win.toggleFullscreen();
```

### Window Properties

```typescript
// Set title
await win.setTitle('New Title');

// Set position
await win.setPosition(100, 100);

// Set size
await win.setSize(800, 600);

// Set minimum size
await win.setMinSize(400, 300);

// Set maximum size
await win.setMaxSize(1920, 1080);

// Center on screen
await win.center();

// Set always on top
await win.setAlwaysOnTop(true);

// Set resizable
await win.setResizable(false);
```

### Window Queries

```typescript
// Get position
const pos = await win.getPosition();
// { x: 100, y: 100 }

// Get size
const size = await win.getSize();
// { width: 800, height: 600 }

// Get bounds (position + size)
const bounds = await win.getBounds();
// { x: 100, y: 100, width: 800, height: 600 }

// Get complete state
const state = await win.getState();
// {
//   label: 'main',
//   visible: true,
//   focused: true,
//   minimized: false,
//   maximized: false,
//   fullscreen: false,
//   bounds: { x: 100, y: 100, width: 800, height: 600 }
// }

// Check states
const visible = await win.isVisible();
const focused = await win.isFocused();
const minimized = await win.isMinimized();
const maximized = await win.isMaximized();
```

### Window Drag (Frameless Windows)

```typescript
import { startDrag } from '@auroraview/sdk';

// In your drag region's mousedown handler
document.querySelector('.title-bar').addEventListener('mousedown', (e) => {
  if (e.button === 0) {
    startDrag();
  }
});

// Or use CSS-based auto-drag
// Add CSS: -webkit-app-region: drag;
// Or class: drag-handle
```

### Finding Windows

```typescript
// Get window by label
const settings = await Window.getByLabel('settings');
if (settings) {
  await settings.focus();
}

// Get all windows
const windows = await Window.getAll();
for (const win of windows) {
  console.log(win.label);
}

// Get window count
const count = await Window.count();
```

### Window Events

```typescript
// Subscribe to window events
const unsubscribe = win.on('resized', (data) => {
  console.log('Window resized:', data.width, data.height);
});

// Available events
win.on('shown', () => {});
win.on('hidden', () => {});
win.on('focused', () => {});
win.on('blurred', () => {});
win.on('resized', (data) => {}); // { width, height }
win.on('moved', (data) => {});   // { x, y }
win.on('minimized', () => {});
win.on('maximized', () => {});
win.on('restored', () => {});
win.on('closing', () => {});
win.on('closed', () => {});

// Unsubscribe
unsubscribe();

// Or use off()
win.off('resized', handler);
```

### Navigation

```typescript
// Navigate to URL
await win.navigate('https://example.com');

// Load HTML content
await win.loadHtml('<h1>Hello</h1>');

// Execute JavaScript
const result = await win.eval('document.title');

// Emit event to window
await win.emit('custom_event', { data: 'value' });
```

## Python Backend API

### Quick Start (Recommended)

Use `create_webview()` with a single line of code to create a fully-featured WebView with built-in window API enabled automatically:

```python
from auroraview import create_webview

# One line to create a fully-featured WebView
webview = create_webview(url="http://localhost:3000")
webview.show()

# JavaScript can directly call window.* APIs:
# await auroraview.call('window.minimize')
# await auroraview.call('window.setTitle', { title: 'New Title' })
```

### Advanced Usage

If you need to disable built-in window API or provide custom implementation:

```python
from auroraview import create_webview

# Disable built-in window API
webview = create_webview(url="...", window_api=False)

# Or use low-level API for full control
from auroraview.core import WebView
from auroraview.core.window_api import setup_window_api, WindowAPI

webview = WebView.create("My App")

# Optional: manually setup window API
setup_window_api(webview)

# Or provide custom implementation
class MyWindowAPI(WindowAPI):
    def close(self, label=None):
        # Custom close logic
        print("Window is about to close...")
        return super().close(label)

webview.show()
```

### Window Manager

```python
from auroraview.core.window_manager import get_window_manager

wm = get_window_manager()

# Register window
uid = wm.register(webview)  # Returns 'wv_a1b2c3d4'
uid = wm.register(webview, uid='settings')  # Custom ID

# Get windows
wm.get(uid)           # Get by ID
wm.get_active()       # Get active window
wm.get_all()          # Get all windows

# Window operations
wm.unregister(uid)    # Unregister
wm.set_active(uid)    # Set active
wm.close_all()        # Close all

# Event broadcasting
wm.broadcast('event_name', {'data': 'value'})

# Change notifications
wm.on_change(callback)
```

### WindowAPI Class

```python
from auroraview.core.window_api import WindowAPI, setup_window_api

# The WindowAPI class provides these methods:
# - show, hide, close, focus
# - minimize, maximize, restore, toggle_fullscreen
# - set_title, set_position, set_size
# - set_min_size, set_max_size, center
# - set_always_on_top, set_resizable
# - get_position, get_size, get_bounds, get_state
# - is_visible, is_focused, is_minimized, is_maximized
# - exists, list, count, create
# - navigate, load_html, eval, emit

# All methods accept an optional 'label' parameter
# to target a specific window
```

## Complete Example

### settings-window.tsx

```tsx
import React from 'react';
import { Window, getCurrentWindow, startDrag } from '@auroraview/sdk';

export function SettingsWindow() {
  const win = getCurrentWindow();

  const handleClose = async () => {
    await win.close();
  };

  const handleMinimize = async () => {
    await win.minimize();
  };

  const handleTitleBarMouseDown = (e: React.MouseEvent) => {
    if (e.button === 0) {
      startDrag();
    }
  };

  return (
    <div className="settings-window">
      <div 
        className="title-bar drag-handle"
        onMouseDown={handleTitleBarMouseDown}
      >
        <span>Settings</span>
        <div className="window-controls no-drag">
          <button onClick={handleMinimize}>−</button>
          <button onClick={handleClose}>×</button>
        </div>
      </div>
      <div className="content">
        {/* Settings content */}
      </div>
    </div>
  );
}
```

### app.py

```python
from auroraview import WebView, setup_window_api

def main():
    # Main window
    main_window = WebView.create(
        "My App",
        url="http://localhost:3000",
        width=1200,
        height=800,
    )
    setup_window_api(main_window)
    
    # Settings will be created from JavaScript
    # using Window.create()
    
    main_window.show()

if __name__ == "__main__":
    main()
```

### Creating settings from JavaScript

```typescript
import { Window } from '@auroraview/sdk';

async function openSettings() {
  // Check if already open
  const existing = await Window.getByLabel('settings');
  if (existing) {
    await existing.focus();
    return;
  }

  // Create new settings window
  const settings = await Window.create({
    label: 'settings',
    url: '/settings.html',
    title: 'Settings',
    width: 520,
    height: 650,
    center: true,
    frameless: true,
  });

  // Handle close event
  settings.on('closed', () => {
    console.log('Settings window closed');
  });
}
```
