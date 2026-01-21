# 窗口管理 API

AuroraView SDK 提供全面的窗口 API，用于从 JavaScript 管理 WebView 窗口。

## 概述

窗口 API 允许你：
- 创建、显示、隐藏和关闭窗口
- 控制窗口位置、大小和状态
- 查询窗口信息
- 处理窗口事件
- 管理多个窗口

## JavaScript SDK

### 获取当前窗口

```typescript
import { Window, getCurrentWindow } from '@auroraview/sdk';

// 使用静态方法
const current = Window.getCurrent();

// 使用便捷函数
const win = getCurrentWindow();
```

### 创建窗口

```typescript
import { Window, createWindow } from '@auroraview/sdk';

// 使用静态方法
const win = await Window.create({
  label: 'settings',
  url: '/settings.html',
  title: '设置',
  width: 520,
  height: 650,
  center: true,
});

// 使用便捷函数
const win2 = await createWindow({
  label: 'preview',
  html: '<h1>预览</h1>',
  width: 400,
  height: 300,
});
```

### 窗口选项

| 选项 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `label` | `string` | 自动 | 唯一窗口标识符 |
| `url` | `string` | - | 要加载的 URL |
| `html` | `string` | - | HTML 内容（url 的替代方案） |
| `title` | `string` | "AuroraView" | 窗口标题 |
| `width` | `number` | 800 | 窗口宽度（像素） |
| `height` | `number` | 600 | 窗口高度（像素） |
| `x` | `number` | - | X 位置 |
| `y` | `number` | - | Y 位置 |
| `center` | `boolean` | false | 居中显示 |
| `resizable` | `boolean` | true | 允许调整大小 |
| `frameless` | `boolean` | false | 隐藏窗口边框 |
| `transparent` | `boolean` | false | 透明背景 |
| `alwaysOnTop` | `boolean` | false | 保持窗口置顶 |
| `minimized` | `boolean` | false | 启动时最小化 |
| `maximized` | `boolean` | false | 启动时最大化 |
| `fullscreen` | `boolean` | false | 启动时全屏 |
| `devtools` | `boolean` | false | 启用开发者工具 |

### 窗口生命周期

```typescript
// 显示窗口
await win.show();

// 隐藏窗口
await win.hide();

// 关闭窗口
await win.close();

// 聚焦窗口（置于前台）
await win.focus();

// 关闭当前窗口
import { closeCurrentWindow } from '@auroraview/sdk';
await closeCurrentWindow();
```

### 窗口状态

```typescript
// 最小化
await win.minimize();

// 最大化
await win.maximize();

// 从最小化/最大化恢复
await win.restore();

// 切换全屏
await win.toggleFullscreen();
```

### 窗口属性

```typescript
// 设置标题
await win.setTitle('新标题');

// 设置位置
await win.setPosition(100, 100);

// 设置大小
await win.setSize(800, 600);

// 设置最小大小
await win.setMinSize(400, 300);

// 设置最大大小
await win.setMaxSize(1920, 1080);

// 居中显示
await win.center();

// 设置置顶
await win.setAlwaysOnTop(true);

// 设置是否可调整大小
await win.setResizable(false);
```

### 窗口查询

```typescript
// 获取位置
const pos = await win.getPosition();
// { x: 100, y: 100 }

// 获取大小
const size = await win.getSize();
// { width: 800, height: 600 }

// 获取边界（位置 + 大小）
const bounds = await win.getBounds();
// { x: 100, y: 100, width: 800, height: 600 }

// 获取完整状态
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

// 检查状态
const visible = await win.isVisible();
const focused = await win.isFocused();
const minimized = await win.isMinimized();
const maximized = await win.isMaximized();
```

### 窗口拖拽（无边框窗口）

```typescript
import { startDrag } from '@auroraview/sdk';

// 在拖拽区域的 mousedown 处理器中
document.querySelector('.title-bar').addEventListener('mousedown', (e) => {
  if (e.button === 0) {
    startDrag();
  }
});

// 或者使用基于 CSS 的自动拖拽
// 添加 CSS: -webkit-app-region: drag;
// 或者添加 class: drag-handle
```

### 查找窗口

```typescript
// 通过标签获取窗口
const settings = await Window.getByLabel('settings');
if (settings) {
  await settings.focus();
}

// 获取所有窗口
const windows = await Window.getAll();
for (const win of windows) {
  console.log(win.label);
}

// 获取窗口数量
const count = await Window.count();
```

### 窗口事件

```typescript
// 订阅窗口事件
const unsubscribe = win.on('resized', (data) => {
  console.log('窗口大小改变:', data.width, data.height);
});

// 可用事件
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

// 取消订阅
unsubscribe();

// 或者使用 off()
win.off('resized', handler);
```

### 导航

```typescript
// 导航到 URL
await win.navigate('https://example.com');

// 加载 HTML 内容
await win.loadHtml('<h1>你好</h1>');

// 执行 JavaScript
const result = await win.eval('document.title');

// 向窗口发送事件
await win.emit('custom_event', { data: 'value' });
```

## Python 后端 API

### 设置窗口 API

```python
from auroraview import WebView, setup_window_api

# 创建 WebView
webview = WebView.create("我的应用")

# 设置窗口 API（启用 JS window.* 调用）
setup_window_api(webview)

webview.show()
```

### 窗口管理器

```python
from auroraview.core.window_manager import get_window_manager

wm = get_window_manager()

# 注册窗口
uid = wm.register(webview)  # 返回 'wv_a1b2c3d4'
uid = wm.register(webview, uid='settings')  # 自定义 ID

# 获取窗口
wm.get(uid)           # 通过 ID 获取
wm.get_active()       # 获取活动窗口
wm.get_all()          # 获取所有窗口

# 窗口操作
wm.unregister(uid)    # 注销
wm.set_active(uid)    # 设置活动窗口
wm.close_all()        # 关闭所有

# 事件广播
wm.broadcast('event_name', {'data': 'value'})

# 变更通知
wm.on_change(callback)
```

### WindowAPI 类

```python
from auroraview.core.window_api import WindowAPI, setup_window_api

# WindowAPI 类提供以下方法：
# - show, hide, close, focus
# - minimize, maximize, restore, toggle_fullscreen
# - set_title, set_position, set_size
# - set_min_size, set_max_size, center
# - set_always_on_top, set_resizable
# - get_position, get_size, get_bounds, get_state
# - is_visible, is_focused, is_minimized, is_maximized
# - exists, list, count, create
# - navigate, load_html, eval, emit

# 所有方法都接受可选的 'label' 参数
# 来指定目标窗口
```

## 完整示例

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
        <span>设置</span>
        <div className="window-controls no-drag">
          <button onClick={handleMinimize}>−</button>
          <button onClick={handleClose}>×</button>
        </div>
      </div>
      <div className="content">
        {/* 设置内容 */}
      </div>
    </div>
  );
}
```

### app.py

```python
from auroraview import WebView, setup_window_api

def main():
    # 主窗口
    main_window = WebView.create(
        "我的应用",
        url="http://localhost:3000",
        width=1200,
        height=800,
    )
    setup_window_api(main_window)
    
    # 设置窗口将从 JavaScript 创建
    # 使用 Window.create()
    
    main_window.show()

if __name__ == "__main__":
    main()
```

### 从 JavaScript 创建设置窗口

```typescript
import { Window } from '@auroraview/sdk';

async function openSettings() {
  // 检查是否已打开
  const existing = await Window.getByLabel('settings');
  if (existing) {
    await existing.focus();
    return;
  }

  // 创建新的设置窗口
  const settings = await Window.create({
    label: 'settings',
    url: '/settings.html',
    title: '设置',
    width: 520,
    height: 650,
    center: true,
    frameless: true,
  });

  // 处理关闭事件
  settings.on('closed', () => {
    console.log('设置窗口已关闭');
  });
}
```
