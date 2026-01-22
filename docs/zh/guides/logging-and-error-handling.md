# 日志与错误处理指南

本指南介绍如何在 AuroraView 应用中正确使用日志和错误处理，涵盖 Python、Rust 和 JavaScript 三层。

## 概述

AuroraView 使用统一的日志和错误处理架构，在以下三层保持一致：
- **Python**（后端 API 处理器）
- **Rust**（WebView 宿主和插件系统）
- **JavaScript**（前端 SDK）

### 核心原则

1. **使用结构化日志** - 日志消息带有可解析的级别前缀
2. **区分日志和错误** - 并非所有 stderr 输出都是错误
3. **Packed Mode 感知** - 在打包模式下，stdout 专用于 JSON-RPC

## Python 日志

### 使用日志器

始终使用 `auroraview.utils.logging` 中的 `get_logger()` 而非 `print()`：

```python
from auroraview.utils.logging import get_logger

logger = get_logger(__name__)

# 不同日志级别
logger.debug("详细调试信息")
logger.info("一般信息")
logger.warning("可能有问题")
logger.error("出错了")
logger.critical("致命错误")
```

### 为什么不用 `print()`？

在 **Packed Mode**（应用打包为可执行文件时）：
- `stdout` 用于与 Rust 宿主的 JSON-RPC 通信
- `stderr` 被监控以检测错误
- 使用 `print(..., file=sys.stderr)` 输出调试日志会触发 `backend_error` 事件

`get_logger()` 函数自动：
- 通过 `AURORAVIEW_PACKED` 环境变量检测打包模式
- 使用结构化日志格式：`[LEVEL:module] message`
- 只有 ERROR 和 CRITICAL 级别才会在前端触发后端错误

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `AURORAVIEW_LOG_LEVEL` | 日志级别 (DEBUG, INFO, WARNING, ERROR, CRITICAL) | WARNING |
| `AURORAVIEW_LOG_ENABLED` | 启用/禁用日志 (1/0) | 1 |
| `AURORAVIEW_LOG_VERBOSE` | 启用详细调试输出 | 0 |
| `AURORAVIEW_PACKED` | 在打包模式下自动设置 | - |

### 配置

```python
from auroraview.utils.logging import configure_logging

# 开发时启用调试日志
configure_logging(level="DEBUG", verbose=True)

# 使用 JSON 格式进行结构化日志
configure_logging(level="INFO", use_json=True)
```

## Rust 层行为

Rust 层读取 Python 的 stderr 并应用智能过滤：

```
[DEBUG:module] message  → 本地记录，不发送到前端
[INFO:module] message   → 本地记录，不发送到前端
[WARNING:module] message → 本地记录，不发送到前端
[ERROR:module] message  → 记录 + 作为 backend_error 发送
[CRITICAL:module] message → 记录 + 作为 backend_error 发送
```

没有识别到级别前缀的消息会扫描错误关键词：
- `error`、`exception`、`traceback`、`fatal` → 视为错误
- 其他消息 → 仅本地记录

## JavaScript 前端

### 处理后端错误

前端 SDK 区分致命和非致命错误：

```typescript
import { createAuroraView } from '@auroraview/sdk';

const av = createAuroraView();

// 配置错误处理行为
av.setConfig({
  callTimeoutMs: 30000,           // RPC 超时
  backendFailFast: true,          // 致命错误时取消待处理调用
  failFastSeverity: 'fatal',      // 仅在致命错误时取消
});

// 监听后端错误
av.on('backend_error', (detail) => {
  console.warn('后端错误:', detail.message);
  // 非致命错误不会清除待处理调用
});
```

### 错误码

SDK 提供与 Rust `PluginErrorCode` 匹配的标准错误码：

```typescript
import type { PluginErrorCode } from '@auroraview/sdk';

try {
  await av.call('api.some_method');
} catch (error) {
  if (error.code === 'TIMEOUT') {
    // 处理超时
  } else if (error.code === 'FILE_NOT_FOUND') {
    // 处理文件未找到
  }
}
```

### 致命 vs 非致命错误

触发 fail-fast 的致命错误：
- `process has exited`
- `backend ready timeout`
- `stdout closed`
- `connection lost`
- `fatal error`
- `crash`

非致命错误（记录但不清除待处理调用）：
- 调试/信息级日志输出
- 警告
- 临时错误

## 最佳实践

### 1. 始终使用结构化日志

```python
# ✅ 正确
logger = get_logger(__name__)
logger.info(f"正在处理文件: {filename}")

# ❌ 错误
print(f"正在处理文件: {filename}", file=sys.stderr)
```

### 2. 选择适当的日志级别

```python
# DEBUG: 用于调试的详细信息
logger.debug(f"变量状态: {vars}")

# INFO: 一般操作信息
logger.info(f"开始处理 {count} 个项目")

# WARNING: 意外但非关键的情况
logger.warning(f"未找到配置，使用默认值")

# ERROR: 操作失败但应用可继续
logger.error(f"保存文件失败: {e}")

# CRITICAL: 应用无法继续
logger.critical(f"数据库连接失败: {e}")
```

### 3. 在错误消息中包含上下文

```python
try:
    result = process_data(data)
except Exception as e:
    logger.error(f"处理数据失败 (id={data.id}): {e}")
    raise
```

### 4. 在 API 处理器中使用 Try/Except

```python
@webview.bind_call("api.process_file")
def process_file(path: str) -> dict:
    logger = get_logger(__name__)
    try:
        logger.info(f"正在处理: {path}")
        result = do_processing(path)
        return {"ok": True, "result": result}
    except FileNotFoundError:
        logger.warning(f"文件未找到: {path}")
        return {"ok": False, "error": "文件未找到"}
    except Exception as e:
        logger.error(f"处理失败: {e}")
        return {"ok": False, "error": str(e)}
```

## 调试技巧

### 启用详细日志

```bash
# 运行前设置环境变量
export AURORAVIEW_LOG_LEVEL=DEBUG
export AURORAVIEW_LOG_VERBOSE=1
```

### 检查日志输出

在打包模式下，日志带有结构化前缀输出到 stderr：
```
[DEBUG:dependency_api] 开始安装
[INFO:dependency_api] 发现 3 个包需要安装
[WARNING:dependency_api] 包 xyz 已弃用
[ERROR:dependency_api] 安装失败: 网络错误
```

### 前端控制台

打开 DevTools (F12) 可以看到：
- `[AuroraView] Backend error received: ...` 非致命错误
- `[AuroraView] Fatal backend error: ...` 致命错误
- `[AuroraView] No pending call for id: ...` IPC 同步问题

## 迁移指南

如果你要从 `print()` 迁移到 `logging`：

```python
# 之前
import sys
print(f"[MyModule] 正在处理 {item}", file=sys.stderr)

# 之后
from auroraview.utils.logging import get_logger
logger = get_logger(__name__)
logger.info(f"正在处理 {item}")
```

新方法的优势：
- 在普通模式和打包模式下都能正确工作
- 支持日志级别过滤
- 不会触发误报的 backend_error 事件
- 支持 JSON 结构化日志
