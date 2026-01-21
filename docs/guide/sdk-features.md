# SDK Features API

The AuroraView SDK provides feature detection and environment information APIs to help you write robust cross-platform code.

## Feature Detection

### hasFeature

Check if a specific feature is available at runtime.

```typescript
import { hasFeature, Features } from '@auroraview/sdk';

// Check using constant
if (hasFeature(Features.WINDOW_DRAG)) {
  window.auroraview.startDrag();
}

// Check using string
if (hasFeature('clipboard')) {
  const text = await window.auroraview.clipboard.readText();
}
```

### Available Features

| Feature | Description |
|---------|-------------|
| `windowDrag` | Native window drag support (frameless windows) |
| `multiWindow` | Multi-window management API |
| `clipboard` | Clipboard read/write operations |
| `shell` | Shell command execution, open URLs/files |
| `fileSystem` | File system read/write operations |
| `dialog` | Native dialog boxes (open, save, message) |
| `state` | Shared state between Python and JavaScript |
| `invoke` | Plugin command invocation |
| `api` | Bound Python API methods |

### hasFeatures

Check multiple features at once.

```typescript
import { hasFeatures } from '@auroraview/sdk';

const available = hasFeatures(['clipboard', 'shell', 'dialog']);
// { clipboard: true, shell: true, dialog: false }
```

### getAvailableFeatures

Get all available features.

```typescript
import { getAvailableFeatures } from '@auroraview/sdk';

const features = getAvailableFeatures();
// ['windowDrag', 'clipboard', 'shell', ...]
```

### waitForFeature

Wait for a specific feature to become available.

```typescript
import { waitForFeature } from '@auroraview/sdk';

try {
  await waitForFeature('clipboard', 3000);
  const text = await window.auroraview.clipboard.readText();
} catch (e) {
  console.log('Clipboard not available');
}
```

## Environment Detection

### getEnvironment

Get comprehensive environment information.

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

### Environment Properties

| Property | Type | Description |
|----------|------|-------------|
| `mode` | `'standalone' \| 'dcc' \| 'browser' \| 'packed'` | Runtime mode |
| `platform` | `'windows' \| 'macos' \| 'linux' \| 'unknown'` | OS platform |
| `dccHost` | `string \| null` | DCC host application name |
| `embedded` | `boolean` | Whether running inside another app |
| `version` | `string` | AuroraView version |
| `features` | `string[]` | Available feature names |
| `userAgent` | `string` | Browser user agent |
| `debug` | `boolean` | Debug mode enabled |

### Convenience Functions

```typescript
import { isAuroraView, isDCC, isStandalone, isPacked } from '@auroraview/sdk';

// Check if running in AuroraView
if (isAuroraView()) {
  // Use native features
  await window.auroraview.clipboard.writeText('Hello');
} else {
  // Fallback to browser APIs
  await navigator.clipboard.writeText('Hello');
}

// Check for DCC environment
if (isDCC()) {
  // Maya/Houdini/Blender specific code
}

// Check for standalone mode
if (isStandalone()) {
  // Desktop app specific code
}

// Check for packed mode
if (isPacked()) {
  // Packed app specific code
}
```

## Usage Examples

### Cross-Platform Feature Support

```typescript
import { hasFeature, getEnvironment } from '@auroraview/sdk';

async function copyToClipboard(text: string) {
  if (hasFeature('clipboard')) {
    // Use native clipboard
    await window.auroraview.clipboard.writeText(text);
  } else if (navigator.clipboard) {
    // Fallback to browser API
    await navigator.clipboard.writeText(text);
  } else {
    // Manual fallback
    const textarea = document.createElement('textarea');
    textarea.value = text;
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand('copy');
    document.body.removeChild(textarea);
  }
}
```

### DCC-Specific Code

```typescript
import { getEnvironment } from '@auroraview/sdk';

const env = getEnvironment();

if (env.mode === 'dcc' && env.dccHost === 'maya') {
  // Maya-specific initialization
  console.log('Running inside Maya');
} else if (env.mode === 'dcc' && env.dccHost === 'houdini') {
  // Houdini-specific initialization
  console.log('Running inside Houdini');
}
```

### Feature-Based UI

```typescript
import { hasFeature, getAvailableFeatures } from '@auroraview/sdk';

function initializeUI() {
  const features = getAvailableFeatures();
  
  // Show/hide UI elements based on available features
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

## TypeScript Types

```typescript
// Feature name type
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

// Platform type
type Platform = 'windows' | 'macos' | 'linux' | 'unknown';

// Runtime mode type
type RuntimeMode = 'standalone' | 'dcc' | 'browser' | 'packed';

// DCC host type
type DCCHost =
  | 'maya'
  | 'houdini'
  | 'blender'
  | 'nuke'
  | '3dsmax'
  | 'photoshop'
  | 'unreal'
  | null;

// Environment interface
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
