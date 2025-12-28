# TypeScript SDK

AuroraView provides a TypeScript SDK (`@auroraview/sdk`) for frontend development. This guide covers how to use the SDK in your web applications.

## Installation

```bash
npm install @auroraview/sdk
# or
yarn add @auroraview/sdk
# or
pnpm add @auroraview/sdk
```

## Quick Start

### Basic Usage

```typescript
import { createAuroraView, getAuroraView } from '@auroraview/sdk';

// Get the AuroraView client (auto-creates if needed)
const av = getAuroraView();

// Call Python API methods
const data = await av.call('api.get_data', { id: 123 });

// Listen for Python events
av.on('data_updated', (payload) => {
  console.log('Data updated:', payload);
});

// Send events to Python
av.sendEvent('user_action', { action: 'click', target: 'button' });
```

### React Integration

```tsx
import { useAuroraView, useAuroraViewEvent } from '@auroraview/sdk/react';

function MyComponent() {
  const av = useAuroraView();
  const [data, setData] = useState(null);
  
  // Listen for events
  useAuroraViewEvent('data_updated', (payload) => {
    setData(payload);
  });
  
  const handleClick = async () => {
    const result = await av.call('api.process', { value: 42 });
    console.log(result);
  };
  
  return (
    <button onClick={handleClick}>
      Process Data
    </button>
  );
}
```

### Vue Integration

```vue
<script setup lang="ts">
import { useAuroraView, useAuroraViewEvent } from '@auroraview/sdk/vue';
import { ref } from 'vue';

const av = useAuroraView();
const data = ref(null);

useAuroraViewEvent('data_updated', (payload) => {
  data.value = payload;
});

async function handleClick() {
  const result = await av.call('api.process', { value: 42 });
  console.log(result);
}
</script>

<template>
  <button @click="handleClick">Process Data</button>
</template>
```

## Core API

### AuroraViewClient

The main client for communicating with the Python backend.

#### `call(method, params?)`

Call a Python method and get the result.

```typescript
// Simple call
const result = await av.call('api.get_user', { id: 1 });

// With type safety
interface User {
  id: number;
  name: string;
}
const user = await av.call<User>('api.get_user', { id: 1 });
```

#### `sendEvent(event, data?)`

Send a fire-and-forget event to Python.

```typescript
av.sendEvent('button_clicked', { id: 'save-btn' });
av.sendEvent('viewport_changed', { zoom: 1.5, pan: [0, 0] });
```

#### `on(event, handler)`

Subscribe to events from Python.

```typescript
const unsubscribe = av.on('selection_changed', (items) => {
  console.log('Selected:', items);
});

// Later, unsubscribe
unsubscribe();
```

#### `once(event, handler)`

Subscribe to an event once.

```typescript
av.once('init_complete', (config) => {
  console.log('Initialized with:', config);
});
```

#### `off(event, handler?)`

Unsubscribe from events.

```typescript
// Remove specific handler
av.off('selection_changed', myHandler);

// Remove all handlers for event
av.off('selection_changed');
```

### API Proxy

Access Python API methods using a proxy object (pywebview-style).

```typescript
// These are equivalent:
await av.call('api.get_data');
await av.api.get_data();

// With parameters:
await av.call('api.save_file', { path: '/tmp/test.txt', content: 'Hello' });
await av.api.save_file({ path: '/tmp/test.txt', content: 'Hello' });
```

### State Management

Access shared state between Python and JavaScript.

```typescript
// Read state
const theme = av.state.theme;

// Write state (syncs to Python)
av.state.theme = 'dark';

// Watch for changes
av.state.onChange('theme', (newValue, oldValue) => {
  console.log(`Theme changed from ${oldValue} to ${newValue}`);
});
```

## Plugins

The SDK includes built-in plugins for common operations.

### File System Plugin

```typescript
import { fs } from '@auroraview/sdk';

// Read file
const content = await fs.readFile('/path/to/file.txt');

// Write file
await fs.writeFile('/path/to/file.txt', 'Hello, World!');

// Check if file exists
const exists = await fs.exists('/path/to/file.txt');

// List directory
const files = await fs.readDir('/path/to/dir');
```

### Dialog Plugin

```typescript
import { dialog } from '@auroraview/sdk';

// Open file dialog
const files = await dialog.open({
  multiple: true,
  filters: [
    { name: 'Images', extensions: ['png', 'jpg', 'gif'] }
  ]
});

// Save file dialog
const path = await dialog.save({
  defaultPath: 'untitled.txt',
  filters: [
    { name: 'Text Files', extensions: ['txt'] }
  ]
});

// Message dialog
await dialog.message('Operation complete!', {
  title: 'Success',
  type: 'info'
});

// Confirm dialog
const confirmed = await dialog.confirm('Are you sure?', {
  title: 'Confirm',
  type: 'warning'
});
```

### Shell Plugin

```typescript
import { shell } from '@auroraview/sdk';

// Open URL in default browser
await shell.openUrl('https://example.com');

// Open file with default application
await shell.openPath('/path/to/document.pdf');

// Reveal in file manager
await shell.showInFolder('/path/to/file.txt');
```

### Clipboard Plugin

```typescript
import { clipboard } from '@auroraview/sdk';

// Read text
const text = await clipboard.readText();

// Write text
await clipboard.writeText('Hello, World!');
```

## TypeScript Types

### Defining API Types

```typescript
// Define your API interface
interface MyAPI {
  getUser(params: { id: number }): Promise<User>;
  saveUser(params: { user: User }): Promise<{ success: boolean }>;
  deleteUser(params: { id: number }): Promise<void>;
}

// Use with type safety
const av = getAuroraView<MyAPI>();

const user = await av.api.getUser({ id: 1 });
// user is typed as User
```

### Event Types

```typescript
// Define event types
interface MyEvents {
  selection_changed: { items: string[] };
  progress_updated: { percent: number; message: string };
  error: { code: string; message: string };
}

// Use with type safety
const av = getAuroraView<unknown, MyEvents>();

av.on('selection_changed', (data) => {
  // data is typed as { items: string[] }
  console.log(data.items);
});
```

## React Hooks

### `useAuroraView()`

Get the AuroraView client instance.

```tsx
function MyComponent() {
  const av = useAuroraView();
  
  const handleClick = async () => {
    await av.call('api.doSomething');
  };
  
  return <button onClick={handleClick}>Do Something</button>;
}
```

### `useAuroraViewEvent(event, handler)`

Subscribe to events with automatic cleanup.

```tsx
function MyComponent() {
  const [items, setItems] = useState<string[]>([]);
  
  useAuroraViewEvent('selection_changed', (data) => {
    setItems(data.items);
  });
  
  return <ul>{items.map(item => <li key={item}>{item}</li>)}</ul>;
}
```

### `useAuroraViewState(key)`

Sync with shared state.

```tsx
function ThemeToggle() {
  const [theme, setTheme] = useAuroraViewState('theme');
  
  return (
    <button onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}>
      Toggle Theme ({theme})
    </button>
  );
}
```

### `useAuroraViewCall(method)`

Create a callable function with loading state.

```tsx
function UserProfile({ userId }: { userId: number }) {
  const { data, loading, error, call } = useAuroraViewCall<User>('api.getUser');
  
  useEffect(() => {
    call({ id: userId });
  }, [userId]);
  
  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;
  if (!data) return null;
  
  return <div>{data.name}</div>;
}
```

## Vue Composables

### `useAuroraView()`

```vue
<script setup>
import { useAuroraView } from '@auroraview/sdk/vue';

const av = useAuroraView();

async function handleClick() {
  await av.call('api.doSomething');
}
</script>
```

### `useAuroraViewEvent(event, handler)`

```vue
<script setup>
import { useAuroraViewEvent } from '@auroraview/sdk/vue';
import { ref } from 'vue';

const items = ref([]);

useAuroraViewEvent('selection_changed', (data) => {
  items.value = data.items;
});
</script>
```

### `useAuroraViewState(key)`

```vue
<script setup>
import { useAuroraViewState } from '@auroraview/sdk/vue';

const theme = useAuroraViewState('theme');
</script>

<template>
  <button @click="theme = theme === 'dark' ? 'light' : 'dark'">
    Toggle Theme ({{ theme }})
  </button>
</template>
```

## Error Handling

### Try-Catch Pattern

```typescript
try {
  const result = await av.call('api.riskyOperation');
} catch (error) {
  if (error instanceof AuroraViewError) {
    console.error('API Error:', error.message);
    console.error('Code:', error.code);
  }
}
```

### Global Error Handler

```typescript
av.onError((error) => {
  console.error('AuroraView Error:', error);
  // Show notification, log to analytics, etc.
});
```

## Best Practices

### 1. Type Your APIs

Always define TypeScript interfaces for your API:

```typescript
interface SceneAPI {
  getObjects(): Promise<SceneObject[]>;
  selectObjects(params: { ids: string[] }): Promise<void>;
  deleteObjects(params: { ids: string[] }): Promise<{ deleted: number }>;
}

const av = getAuroraView<SceneAPI>();
```

### 2. Use Hooks for Cleanup

React and Vue hooks automatically clean up subscriptions:

```tsx
// Good - automatic cleanup
useAuroraViewEvent('event', handler);

// Manual - must remember to unsubscribe
useEffect(() => {
  const unsub = av.on('event', handler);
  return () => unsub();
}, []);
```

### 3. Handle Loading States

```tsx
function DataView() {
  const { data, loading, error, call } = useAuroraViewCall('api.getData');
  
  if (loading) return <Spinner />;
  if (error) return <ErrorMessage error={error} />;
  return <DataDisplay data={data} />;
}
```

### 4. Debounce Frequent Calls

```typescript
import { debounce } from 'lodash-es';

const debouncedSearch = debounce(async (query: string) => {
  const results = await av.call('api.search', { query });
  setResults(results);
}, 300);
```

## Migration from pywebview

If you're migrating from pywebview, the SDK provides compatible APIs:

```javascript
// pywebview style
const result = await pywebview.api.myMethod(arg1, arg2);

// AuroraView SDK (compatible)
const result = await auroraview.api.myMethod(arg1, arg2);
```

The main differences:
- Use `@auroraview/sdk` package instead of global `pywebview`
- TypeScript support with full type inference
- React and Vue integrations
- Built-in plugins for common operations
