# RFC 0005 实施指南：MCP Embedded 优先开发指导

> **相关 RFC**: [0005-mcp-architecture-pivot.md](./0005-mcp-architecture-pivot.md)  
> **目标版本**: v0.5.0  
> **更新日期**: 2026-01-07

## 开发者快速指南

### 🎯 核心原则

1. **Embedded First**: 默认使用 Embedded 模式，除非有明确的进程隔离需求
2. **主线程安全**: 所有 MCP 工具调用必须在主线程（或 DCC 的安全线程）执行
3. **渐进增强**: 保持向后兼容，提供清晰的迁移路径
4. **简单优于复杂**: 优先选择简单可靠的方案

### 🚀 立即行动项（按优先级）

#### Priority 1: 修复 Embedded 模式关键问题

**目标**: 确保 Embedded 模式在所有环境中稳定运行

```bash
# 1. 测试当前 Embedded 模式稳定性
just test-mcp-embedded

# 2. 运行 DCC 集成测试
just test-dcc-maya
just test-dcc-unreal
```

**关键文件**:
- `src/webview/core/main.rs` - `create_mcp_server` 绑定
- `src/webview/event_loop.rs` - 事件循环集成
- `python/auroraview/core/webview.py` - Python API

**必修任务**:
```rust
// 1. 确保 create_mcp_server 绑定稳定
#[pymethod]
fn create_mcp_server(&self, config: &PyAny) -> PyResult<PyObject> {
    // 添加更好的错误处理
    // 确保 dispatcher 正确连接到 MessageQueue
}

// 2. 添加工具执行超时机制
impl McpDispatcher {
    fn execute_tool_with_timeout(&self, timeout: Duration) {
        // 避免主线程被长时间运行的工具卡死
    }
}
```

#### Priority 2: 完善错误处理和用户体验

**目标**: 提供清晰的错误信息和调试能力

```python
# 改进前（容易误用）
server = McpServer()  # ❌ 可能在错误线程执行

# 改进后（引导正确使用）
webview = WebView(mcp=True)  # ✅ 自动选择最佳模式
# 或明确指定
webview = WebView(mcp={"mode": "embedded", "timeout": 30})
```

**实现要点**:
```python
class WebView:
    def _start_embedded_mcp_server(self):
        try:
            # 优先使用 Rust 的 create_mcp_server
            server = self._core.create_mcp_server(self._mcp_config)
            logger.info("✅ MCP Embedded mode started successfully")
        except AttributeError:
            # 提供清晰的降级路径和警告
            logger.warning(
                "⚠️  Using fallback MCP server. "
                "Tool calls may not be thread-safe in DCC environments. "
                "Consider updating to latest auroraview version."
            )
            server = McpServer(self._mcp_config)
```

#### Priority 3: 强化 DCC 线程安全

**目标**: 确保在 Maya/Houdini/Unreal 等环境中安全运行

**Maya 集成测试**:
```python
# tests/dcc/test_maya_mcp.py
def test_maya_mcp_thread_safety():
    """确保 MCP 工具在 Maya 主线程执行"""
    import maya.cmds as cmds
    
    webview = WebView(mcp=True)
    
    @webview.bind_call
    def create_cube():
        # 这个调用必须在 Maya 主线程
        return cmds.polyCube()[0]
    
    # 测试多次调用不会 crash
    for _ in range(10):
        result = webview.call_mcp_tool("create_cube")
        assert cmds.objExists(result)
```

**Unreal 集成测试**:
```python
# tests/dcc/test_unreal_mcp.py
def test_unreal_mcp_game_thread():
    """确保 MCP 工具在 Unreal Game Thread 执行"""
    import unreal
    
    webview = WebView(mcp=True)
    
    @webview.bind_call
    def spawn_actor():
        # 必须在 Game Thread 执行
        world = unreal.EditorLevelLibrary.get_editor_world()
        return unreal.EditorLevelLibrary.spawn_actor_from_class(
            unreal.Actor, unreal.Vector(0, 0, 0)
        )
    
    # 验证线程安全
    assert webview.call_mcp_tool("spawn_actor") is not None
```

### 🔧 Sidecar 模式修复（可选）

**仅在需要进程隔离时考虑**

#### 修复 Ready 协议
```rust
// crates/auroraview-mcp-server/src/main.rs
async fn main() -> Result<()> {
    // ... 启动服务器 ...
    
    // 明确向 stdout 输出 ready 信号
    println!("READY {}", actual_port);
    
    // 日志仍然写 stderr
    tracing::info!("MCP Sidecar ready on port {}", actual_port);
}
```

```python
# python/auroraview/mcp/sidecar.py
def _wait_for_ready(self, timeout: float = 10.0) -> int:
    """等待 sidecar ready 并返回端口"""
    while time.time() - start_time < timeout:
        line = self._process.stdout.readline()
        if line:
            text = line.decode("utf-8").strip()
            if text.startswith("READY "):
                return int(text.split()[1])
    raise TimeoutError("Sidecar failed to start within timeout")
```

#### 修复 IPC BufReader
```rust
// crates/auroraview-mcp-server/src/ipc/server.rs
fn handle_connection(&self, mut stream: LocalSocketStream) -> IpcResult<()> {
    let mut authenticated = false;
    let mut reader = BufReader::new(&mut stream);  // 连接级别创建
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {  // 复用同一个 reader
            // ... 处理逻辑 ...
        }
    }
}
```

### 📚 文档和示例更新

#### 更新 README 示例
```python
# README.md 主要示例
from auroraview import WebView

# 🎯 推荐用法：Embedded 模式
webview = WebView(mcp=True)

@webview.bind_call
def hello(name: str) -> str:
    """Say hello to someone."""
    return f"Hello, {name}!"

webview.show()
# MCP 工具自动可用：http://localhost:8000/mcp
```

#### 创建迁移指南
```markdown
# docs/migration/mcp-v0.5.md

## 从 Sidecar 优先迁移到 Embedded 优先

### 无需更改的情况
- 使用 `WebView(mcp=True)` - 自动选择最佳模式
- 使用 `@webview.bind_call` 注册工具

### 推荐更改
```python
# 旧方式（仍支持）
webview = WebView(mcp=True)

# 新方式（更明确）
webview = WebView(mcp={
    "mode": "embedded",  # 明确指定模式
    "timeout": 30,       # 工具执行超时
    "auto_expose_api": True
})
```

### 🧪 测试策略

#### 自动化测试矩阵
```yaml
# .github/workflows/mcp-test.yml
strategy:
  matrix:
    os: [windows-latest, macos-latest, ubuntu-latest]
    python: ['3.7', '3.8', '3.9', '3.10', '3.11', '3.12']
    mode: [embedded]  # 主要测试 embedded
    include:
      - os: windows-latest
        python: '3.9'
        mode: sidecar  # 少量 sidecar 测试
```

#### 性能基准测试
```python
# tests/benchmarks/test_mcp_performance.py
def test_embedded_vs_sidecar_latency():
    """对比 Embedded 和 Sidecar 模式的延迟"""
    
    # Embedded 模式基准
    webview_embedded = WebView(mcp={"mode": "embedded"})
    embedded_latency = measure_tool_call_latency(webview_embedded)
    
    # Sidecar 模式基准（如果可用）
    if sidecar_available():
        webview_sidecar = WebView(mcp={"mode": "sidecar"})
        sidecar_latency = measure_tool_call_latency(webview_sidecar)
        
        # Embedded 应该更快
        assert embedded_latency < sidecar_latency
```

### 🎯 成功指标和验收标准

#### Phase 1 验收标准
- [ ] `WebView(mcp=True)` 在所有支持平台启动成功率 > 99%
- [ ] DCC 环境（Maya/Unreal）中 MCP 工具调用无 crash
- [ ] 工具执行超时机制正常工作
- [ ] 内存使用增量 < 50MB

#### Phase 2 验收标准（可选）
- [ ] Sidecar 启动成功率 > 95%
- [ ] IPC 并发测试无消息丢失
- [ ] 安全机制（auth/origins）正常工作

#### 用户体验指标
- [ ] 新用户 10 分钟内完成第一个 MCP 工具
- [ ] MCP 相关 GitHub issues 减少 > 50%
- [ ] 文档和示例满意度 > 4.5/5

### 🚨 风险缓解

#### 性能风险
```python
# 如果 Embedded 模式出现性能问题
@webview.bind_call(async_execution=True)  # 未来功能
def heavy_computation():
    # 在后台线程执行，结果回调主线程
    pass
```

#### 兼容性风险
```python
# 保持向后兼容的降级路径
def _start_mcp_server(self):
    try:
        # 优先 Embedded
        return self._start_embedded_mcp_server()
    except Exception as e:
        logger.warning(f"Embedded mode failed: {e}")
        if self._mcp_mode == "auto":
            # 自动降级到 Sidecar
            return self._try_start_sidecar()
        raise
```

### 📋 开发检查清单

#### 提交前检查
- [ ] 运行 `just test-mcp` 通过
- [ ] 运行 `just lint` 无错误
- [ ] 更新相关文档
- [ ] 添加/更新测试用例
- [ ] 检查向后兼容性

#### PR 审查要点
- [ ] MCP 相关更改是否优先考虑 Embedded 模式？
- [ ] 是否添加了适当的错误处理和用户提示？
- [ ] 是否考虑了 DCC 环境的线程安全？
- [ ] 是否更新了相关文档和示例？

### 🔗 相关资源

- [RFC 0005 完整文档](./0005-mcp-architecture-pivot.md)
- [MCP 协议规范](https://modelcontextprotocol.io/)
- [AuroraView 线程安全指南](../best-practices/threading.md)
- [DCC 集成最佳实践](../integration/dcc.md)

---

**记住**: 简单可靠胜过复杂完美。优先让 Embedded 模式在所有环境中稳定工作，再考虑 Sidecar 的高级功能。