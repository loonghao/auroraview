---
name: gallery-mcp-tester
description: |
  通过 MCP 工具测试 AuroraView Python API，发现设计缺陷和代码味道。
  迭代式测试：启动服务 → 调用工具 → 发现问题 → 修复 → 重新测试。
---

# Gallery MCP Tester

通过 MCP 工具测试 AuroraView Python API，发现设计缺陷和代码味道。

## 工作流程

```
┌─────────────────────────────────────────────────────────────┐
│  1. 用户手动启动: vx just gallery-mcp 27168                  │
│  2. AI 调用 auroraview-gallery MCP 工具进行测试              │
│  3. 发现问题 → 在当前分支修复                                │
│  4. 重复测试直到所有问题解决                                 │
└─────────────────────────────────────────────────────────────┘
```

## Step 1: 启动服务（用户手动执行）

在终端中运行：

```bash
vx just gallery-mcp 27168
```

等待看到以下输出表示服务就绪：

```
[Python] MCP server listening on port 27168
```

## Step 2: 测试 MCP 工具

### 可用工具列表

| 工具名 | 描述 | 参数 |
|--------|------|------|
| `api.get_samples` | 获取所有示例列表 | 无 |
| `api.get_categories` | 获取分类信息 | 无 |
| `api.get_mcp_info` | 获取 MCP 服务器信息 | 无 |
| `api.get_source` | 获取示例源代码 | `sample_id: str` |
| `api.run_sample` | 运行示例 | `sample_id, show_console?, use_channel?` |
| `api.prepare_run_sample` | 准备运行示例 | `sample_id, show_console?, use_channel?` |
| `api.kill_process` | 终止进程 | `pid: int` |
| `api.list_processes` | 列出运行中的进程 | 无 |
| `api.send_to_process` | 发送数据到进程 | `pid, data` |
| `api.send_json_to_process` | 发送 JSON 到进程 | `pid, data` |
| `api.launch_example_as_child` | 作为子窗口启动示例 | `sample_id, extra_env?` |
| `api.close_child` | 关闭子窗口 | `child_id` |
| `api.get_children` | 获取所有子窗口 | 无 |
| `api.send_to_child` | 发送事件到子窗口 | `child_id, event, data` |
| `api.broadcast_to_children` | 广播事件 | `event, data` |
| `api.list_webview_extensions` | 列出已安装扩展 | 无 |
| `api.install_to_webview` | 安装扩展 | `path, name?` |
| `api.remove_webview_extension` | 移除扩展 | `id` |
| `api.open_extensions_dir` | 打开扩展目录 | 无 |
| `api.install_extension_from_url` | 从 URL 安装扩展 | `url` |
| `api.start_extension_bridge` | 启动扩展桥接 | 无 |
| `api.stop_extension_bridge` | 停止扩展桥接 | 无 |
| `api.get_extension_status` | 获取桥接状态 | 无 |
| `api.broadcast_to_extensions` | 广播到扩展 | `event, data` |
| `api.install_extension` | 安装浏览器扩展 | `path, browser` |
| `api.open_url` | 打开 URL | `url` |

### 测试顺序

1. **基础功能测试**
   - `api.get_samples` - 验证返回格式
   - `api.get_categories` - 验证分类结构
   - `api.get_mcp_info` - 验证 MCP 信息

2. **源代码获取测试**
   - `api.get_source(sample_id="hello_world")` - 验证源代码返回

3. **进程管理测试**
   - `api.list_processes` - 验证进程列表
   - `api.run_sample(sample_id="hello_world")` - 验证进程启动
   - `api.kill_process(pid=xxx)` - 验证进程终止

4. **子窗口测试**
   - `api.get_children` - 验证子窗口列表
   - `api.launch_example_as_child(sample_id="hello_world")` - 验证子窗口启动

5. **扩展管理测试**
   - `api.list_webview_extensions` - 验证扩展列表
   - `api.get_extension_status` - 验证桥接状态

## Step 3: 问题发现与修复

### 检查清单

#### API 设计
- [ ] 返回值格式是否一致（`{ok, data, error}`）
- [ ] 参数命名是否清晰
- [ ] 错误信息是否有帮助
- [ ] 是否有冗余的 API

#### 代码味道
- [ ] 函数是否过长
- [ ] 是否有重复代码
- [ ] 命名是否规范
- [ ] 是否有硬编码

#### 文档
- [ ] docstring 是否完整
- [ ] 参数类型是否标注
- [ ] 返回值是否说明

### 问题记录模板

```markdown
## Issue: [简短描述]

**类型**: Bug / 设计缺陷 / 代码味道
**严重性**: High / Medium / Low
**文件**: [affected file]

**问题**: [详细描述]

**根因**: [分析]

**修复方案**: [解决方案]
```

### 修复流程

1. 记录问题
2. 在当前分支修复
3. 验证语法：`python -c "import py_compile; py_compile.compile('path/to/file.py')"`
4. 重新测试 MCP 工具
5. 确认问题解决

## 快速参考

### 启动命令

```bash
vx just gallery-mcp 27168
```

### 相关文件

- `gallery/main.py` - Gallery 主入口
- `gallery/backend/` - API 实现
  - `process_api.py` - 进程管理
  - `child_api.py` - 子窗口管理
  - `extension_api.py` - 扩展管理
  - `webview_extension_api.py` - WebView 扩展
- `python/auroraview/core/` - 核心实现
  - `mixins/api.py` - bind_call 实现
  - `response.py` - 响应工具

### 响应格式

所有 API 应返回标准格式：

```python
# 成功
{"ok": True, "data": ...}

# 失败
{"ok": False, "error": "message"}
```

使用辅助函数：

```python
from auroraview import ok, err

return ok({"key": "value"})
return err("Error message")
```
