# TypeScript SDK

AuroraView 提供了一个功能完整的 TypeScript SDK，用于在前端与 Python 后端进行通信。

## 安装

```bash
npm install @auroraview/sdk
# or
yarn add @auroraview/sdk
# or
pnpm add @auroraview/sdk
```

## 快速开始

```typescript
import { createAuroraView, getAuroraView } from '@auroraview/sdk';

// 创建实例
const av = createAuroraView();

// 或获取现有实例
const av = getAuroraView();

// 调用后端方法
const result = await av.call('api.get_data', { id: 123 });

// 监听事件
av.on('data_updated', (payload) => {
  console.log('数据已更新:', payload);
});
```

## 核心 API

### AuroraViewClient

主客户端类，提供与后端通信的所有方法。

#### call(method, params?)

调用后端注册的方法。

```typescript
// 调用 api 命名空间的方法
const user = await av.call('api.get_user', { id: 1 });

// 调用工具方法
await av.call('tool.apply', { strength: 0.8 });
```

#### on(event, handler)

订阅后端事件。

```typescript
const unsubscribe = av.on('progress', (data) => {
  console.log(`进度: ${data.percent}%`);
});

// 取消订阅
unsubscribe();
```

#### off(event, handler)

取消事件订阅。

```typescript
const handler = (data) => console.log(data);
av.on('event', handler);
av.off('event', handler);
```

#### emit(event, data)

向后端发送事件。

```typescript
av.emit('user_action', { action: 'click', target: 'button' });
```

### API 代理

SDK 提供了 `api` 代理，可以像调用本地方法一样调用后端 API：

```typescript
// 等价于 av.call('api.get_users', [])
const users = await av.api.get_users();

// 等价于 av.call('api.create_user', { name: 'John', email: 'john@example.com' })
const newUser = await av.api.create_user({ name: 'John', email: 'john@example.com' });
```

## 插件系统

SDK 包含多个内置插件，扩展了核心功能。

### 剪贴板插件

```typescript
// 写入剪贴板
await av.clipboard.writeText('Hello World');

// 读取剪贴板
const text = await av.clipboard.readText();
```

### 对话框插件

```typescript
// 打开文件选择器
const files = await av.dialog.open({
  multiple: true,
  filters: [{ name: 'Images', extensions: ['png', 'jpg'] }]
});

// 保存文件对话框
const savePath = await av.dialog.save({
  defaultPath: 'document.txt'
});

// 消息对话框
await av.dialog.message('操作完成', { type: 'info' });
```

### 文件系统插件

```typescript
// 读取文件
const content = await av.fs.readTextFile('/path/to/file.txt');

// 写入文件
await av.fs.writeTextFile('/path/to/file.txt', 'content');

// 检查文件是否存在
const exists = await av.fs.exists('/path/to/file.txt');
```

### Shell 插件

```typescript
// 在默认浏览器中打开 URL
await av.shell.open('https://example.com');

// 在文件管理器中显示文件
await av.shell.showItemInFolder('/path/to/file');
```

## 框架适配器

### React

```tsx
import { useAuroraView, useAuroraViewEvent } from '@auroraview/sdk/react';

function MyComponent() {
  const av = useAuroraView();
  
  // 监听事件
  useAuroraViewEvent('data_updated', (data) => {
    console.log('数据更新:', data);
  });
  
  const handleClick = async () => {
    const result = await av.api.do_something();
    console.log(result);
  };
  
  return <button onClick={handleClick}>执行</button>;
}
```

### Vue

```vue
<script setup lang="ts">
import { useAuroraView, useAuroraViewEvent } from '@auroraview/sdk/vue';

const av = useAuroraView();

useAuroraViewEvent('data_updated', (data) => {
  console.log('数据更新:', data);
});

const handleClick = async () => {
  const result = await av.api.do_something();
  console.log(result);
};
</script>

<template>
  <button @click="handleClick">执行</button>
</template>
```

## 类型定义

SDK 提供完整的 TypeScript 类型支持：

```typescript
import type { 
  AuroraViewClient,
  CallOptions,
  EventHandler,
  DialogOptions 
} from '@auroraview/sdk';

// 自定义 API 类型
interface MyAPI {
  get_user(id: number): Promise<User>;
  create_user(data: CreateUserData): Promise<User>;
}

// 类型安全的调用
const av = getAuroraView<MyAPI>();
const user = await av.api.get_user(1); // 类型推断为 User
```

## 错误处理

```typescript
import { AuroraViewError } from '@auroraview/sdk';

try {
  await av.call('api.risky_operation');
} catch (error) {
  if (error instanceof AuroraViewError) {
    console.error(`错误代码: ${error.code}`);
    console.error(`错误消息: ${error.message}`);
    console.error(`错误数据: ${error.data}`);
  }
}
```

## 配置选项

```typescript
const av = createAuroraView({
  // 调用超时时间（毫秒）
  timeout: 30000,
  
  // 是否启用调试日志
  debug: true,
  
  // 自定义错误处理
  onError: (error) => {
    console.error('AuroraView 错误:', error);
  }
});
```

## 最佳实践

### 1. 单例模式

在应用中使用单一的 AuroraView 实例：

```typescript
// aurora.ts
import { createAuroraView } from '@auroraview/sdk';

export const av = createAuroraView();
```

### 2. 类型安全

定义 API 接口以获得类型提示：

```typescript
interface BackendAPI {
  users: {
    list(): Promise<User[]>;
    get(id: number): Promise<User>;
    create(data: CreateUserData): Promise<User>;
  };
}
```

### 3. 错误边界

在 React 中使用错误边界处理通信错误：

```tsx
class AuroraViewErrorBoundary extends React.Component {
  componentDidCatch(error: Error) {
    if (error instanceof AuroraViewError) {
      // 处理 AuroraView 特定错误
    }
  }
}
```

### 4. 事件清理

确保在组件卸载时清理事件监听：

```typescript
useEffect(() => {
  const unsubscribe = av.on('event', handler);
  return () => unsubscribe();
}, []);
```

## 调试

启用调试模式查看通信日志：

```typescript
const av = createAuroraView({ debug: true });
```

在浏览器控制台中，你将看到：
- 所有 `call` 请求和响应
- 事件订阅和触发
- 错误详情

## 下一步

- [通信机制](./communication.md) - 深入了解 Python-JS 通信
- [高级用法](./advanced-usage) - 探索更多高级功能
- [API 参考](/api/) - 完整的 API 文档
