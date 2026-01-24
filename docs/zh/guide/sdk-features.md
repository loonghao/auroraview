# SDK 特性检测 API

AuroraView SDK 提供特性检测和环境信息 API，帮助你编写健壮的跨平台代码。

## 特性检测

### hasFeature

在运行时检查特定功能是否可用。

```typescript
import { hasFeature, Features } from '@auroraview/sdk';

// 使用常量检查
if (hasFeature(Features.WINDOW_DRAG)) {
  window.auroraview.startDrag();
}

// 使用字符串检查
if (hasFeature('clipboard')) {
  const text = await window.auroraview.clipboard.readText();
}
```

### 可用特性

| 特性 | 描述 |
|------|------|
| `windowDrag` | 原生窗口拖拽支持（无边框窗口） |
| `multiWindow` | 多窗口管理 API |
| `clipboard` | 剪贴板读写操作 |
| `shell` | Shell 命令执行、打开 URL/文件 |
| `fileSystem` | 文件系统读写操作 |
| `dialog` | 原生对话框（打开、保存、消息） |
| `state` | Python 和 JavaScript 之间的共享状态 |
| `invoke` | 插件命令调用 |
| `api` | 绑定的 Python API 方法 |

### hasFeatures

同时检查多个特性。

```typescript
import { hasFeatures } from '@auroraview/sdk';

const available = hasFeatures(['clipboard', 'shell', 'dialog']);
// { clipboard: true, shell: true, dialog: false }
```

### getAvailableFeatures

获取所有可用特性。

```typescript
import { getAvailableFeatures } from '@auroraview/sdk';

const features = getAvailableFeatures();
// ['windowDrag', 'clipboard', 'shell', ...]
```

### waitForFeature

等待特定特性变为可用。

```typescript
import { waitForFeature } from '@auroraview/sdk';

try {
  await waitForFeature('clipboard', 3000);
  const text = await window.auroraview.clipboard.readText();
} catch (e) {
  console.log('剪贴板不可用');
}
```

## 环境检测

### getEnvironment

获取全面的环境信息。

```typescript
import { getEnvironment } from '@auroraview/sdk';

const env = getEnvironment();
console.log(env);
// {
//   mode: 'standalone',
//   platform: 'windows',
//   dccHost: null,
//   embedded: false,
//   version: '0.4.5',
//   features: ['windowDrag', 'clipboard', 'shell', ...],
//   userAgent: 'Mozilla/5.0...',
//   debug: false
// }
```

### 环境属性

| 属性 | 类型 | 描述 |
|------|------|------|
| `mode` | `'standalone' \| 'dcc' \| 'browser' \| 'packed'` | 运行模式 |
| `platform` | `'windows' \| 'macos' \| 'linux' \| 'unknown'` | 操作系统平台 |
| `dccHost` | `string \| null` | DCC 宿主应用名称 |
| `embedded` | `boolean` | 是否在另一个应用内运行 |
| `version` | `string` | AuroraView 版本 |
| `features` | `string[]` | 可用特性名称列表 |
| `userAgent` | `string` | 浏览器用户代理 |
| `debug` | `boolean` | 是否启用调试模式 |

### 便捷函数

```typescript
import { isAuroraView, isDCC, isStandalone, isPacked } from '@auroraview/sdk';

// 检查是否在 AuroraView 中运行
if (isAuroraView()) {
  // 使用原生功能
  await window.auroraview.clipboard.writeText('你好');
} else {
  // 回退到浏览器 API
  await navigator.clipboard.writeText('你好');
}

// 检查是否在 DCC 环境中
if (isDCC()) {
  // Maya/Houdini/Blender 特定代码
}

// 检查是否为独立模式
if (isStandalone()) {
  // 桌面应用特定代码
}

// 检查是否为打包模式
if (isPacked()) {
  // 打包应用特定代码
}
```

## 使用示例

### 跨平台特性支持

```typescript
import { hasFeature, getEnvironment } from '@auroraview/sdk';

async function copyToClipboard(text: string) {
  if (hasFeature('clipboard')) {
    // 使用原生剪贴板
    await window.auroraview.clipboard.writeText(text);
  } else if (navigator.clipboard) {
    // 回退到浏览器 API
    await navigator.clipboard.writeText(text);
  } else {
    // 手动回退方案
    const textarea = document.createElement('textarea');
    textarea.value = text;
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand('copy');
    document.body.removeChild(textarea);
  }
}
```

### DCC 特定代码

```typescript
import { getEnvironment } from '@auroraview/sdk';

const env = getEnvironment();

if (env.mode === 'dcc' && env.dccHost === 'maya') {
  // Maya 特定初始化
  console.log('在 Maya 中运行');
} else if (env.mode === 'dcc' && env.dccHost === 'houdini') {
  // Houdini 特定初始化
  console.log('在 Houdini 中运行');
}
```

### 基于特性的 UI

```typescript
import { hasFeature, getAvailableFeatures } from '@auroraview/sdk';

function initializeUI() {
  const features = getAvailableFeatures();
  
  // 根据可用特性显示/隐藏 UI 元素
  if (hasFeature('clipboard')) {
    document.getElementById('copy-btn')?.classList.remove('hidden');
  }
  
  if (hasFeature('dialog')) {
    document.getElementById('save-btn')?.classList.remove('hidden');
  }
  
  if (hasFeature('shell')) {
    document.getElementById('open-folder-btn')?.classList.remove('hidden');
  }
}
```

## TypeScript 类型

```typescript
// 特性名称类型
type FeatureName =
  | 'windowDrag'
  | 'multiWindow'
  | 'clipboard'
  | 'shell'
  | 'fileSystem'
  | 'dialog'
  | 'state'
  | 'invoke'
  | 'api';

// 平台类型
type Platform = 'windows' | 'macos' | 'linux' | 'unknown';

// 运行模式类型
type RuntimeMode = 'standalone' | 'dcc' | 'browser' | 'packed';

// DCC 宿主类型
type DCCHost =
  | 'maya'
  | 'houdini'
  | 'blender'
  | 'nuke'
  | '3dsmax'
  | 'photoshop'
  | 'unreal'
  | null;

// 环境接口
interface Environment {
  mode: RuntimeMode;
  platform: Platform;
  dccHost: DCCHost;
  embedded: boolean;
  version: string;
  features: FeatureName[];
  userAgent: string;
  debug: boolean;
}
```
