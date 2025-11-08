# AuroraView 跨平台改进总结

## 🎯 改进目标

解决 DCC 应用程序中 webview 窗口关闭问题，并提供更好的跨平台支持。

## 📦 新增依赖

### 1. `scopeguard` v1.2
- **用途**: 资源清理和 RAII 模式增强
- **优势**: 
  - 类似 Go 的 `defer` 机制
  - 即使发生 panic 也能保证清理
  - 零成本抽象
- **下载量**: 244M+

### 2. `flume` v0.11
- **用途**: 高性能异步/同步通道
- **优势**:
  - 比 `std::sync::mpsc` 更快
  - 支持异步和同步混用
  - 无 unsafe 代码
  - 支持显式关闭通道
- **性能**: 比标准库更快

## 🏗️ 新增模块

### 1. `src/webview/lifecycle.rs`
跨平台窗口生命周期管理器

**核心功能**:
- 生命周期状态管理 (Creating → Active → CloseRequested → Destroying → Destroyed)
- 事件驱动的关闭通知
- 清理处理器注册
- 线程安全

**关键类型**:
```rust
pub struct LifecycleManager {
    state: Arc<Mutex<LifecycleState>>,
    close_tx: Sender<CloseReason>,
    close_rx: Receiver<CloseReason>,
    cleanup_handlers: Arc<Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>>,
}

pub enum LifecycleState {
    Creating, Active, CloseRequested, Destroying, Destroyed
}

pub enum CloseReason {
    UserRequest, AppRequest, ParentClosed, SystemShutdown, Error
}
```

### 2. `src/webview/platform/` 模块

平台特定的窗口管理实现

**结构**:
```
platform/
├── mod.rs           # 平台抽象 trait
├── windows.rs       # Windows 实现 (完整)
├── macos.rs         # macOS 占位实现
└── linux.rs         # Linux 占位实现
```

**核心 Trait**:
```rust
pub trait PlatformWindowManager: Send + Sync {
    fn process_events(&self) -> bool;
    fn setup_close_handlers(&self, lifecycle: Arc<LifecycleManager>);
    fn cleanup(&self);
    fn is_window_valid(&self) -> bool;
}
```

### 3. Windows 平台实现 (`platform/windows.rs`)

**特性**:
- 使用 `scopeguard` 保证清理
- 检测多种关闭消息源:
  - `WM_CLOSE` - 标准关闭消息
  - `WM_SYSCOMMAND` + `SC_CLOSE` - 系统菜单关闭
  - `WM_NCLBUTTONUP`/`WM_NCLBUTTONDOWN` + `HTCLOSE` - 标题栏关闭按钮
  - `WM_DESTROY` - 窗口销毁
- 通过 `flume` 通道发送关闭信号
- 线程安全 (实现 Send + Sync)

**关键改进**:
```rust
// 存储 HWND 为 u64 而非 HWND 指针，确保 Send + Sync
pub struct WindowsWindowManager {
    hwnd_value: u64,  // ✅ 线程安全
    lifecycle: Arc<Mutex<Option<Arc<LifecycleManager>>>>,
}

// 使用 scopeguard 保证清理
fn process_windows_messages(&self) -> bool {
    defer! {
        trace!("Message processing completed");
    }
    // ... 处理消息
}
```

## 🔄 核心改进

### 1. `WebViewInner` 结构更新

**新增字段**:
```rust
pub struct WebViewInner {
    // ... 现有字段 ...
    
    /// 跨平台生命周期管理器
    pub(crate) lifecycle: Arc<LifecycleManager>,
    
    /// 平台特定窗口管理器
    pub(crate) platform_manager: Option<Box<dyn PlatformWindowManager>>,
}
```

### 2. `Drop` 实现改进

**使用 scopeguard 保证清理**:
```rust
impl Drop for WebViewInner {
    fn drop(&mut self) {
        use scopeguard::defer;
        
        defer! {
            tracing::info!("Cleanup completed");
        }
        
        // 执行生命周期清理
        self.lifecycle.execute_cleanup();
        
        // 清理平台特定资源
        if let Some(platform_manager) = &self.platform_manager {
            platform_manager.cleanup();
        }
        
        // ... 其余清理代码
    }
}
```

### 3. `process_events()` 方法改进

**事件驱动架构**:
```rust
pub fn process_events(&self) -> bool {
    use scopeguard::defer;
    
    defer! {
        tracing::trace!("tick completed");
    }

    // 1. 检查生命周期状态
    match self.lifecycle.state() {
        LifecycleState::Destroyed => return true,
        LifecycleState::CloseRequested | LifecycleState::Destroying => return true,
        _ => {}
    }

    // 2. 检查关闭信号 (非阻塞)
    if let Some(reason) = self.lifecycle.check_close_requested() {
        return true;
    }

    // 3. 使用平台管理器处理事件
    if let Some(platform_manager) = &self.platform_manager {
        if platform_manager.process_events() {
            return true;
        }
        
        if !platform_manager.is_window_valid() {
            return true;
        }
    }
    
    // ... 其余代码
}
```

### 4. Embedded 模式初始化

**创建生命周期管理器和平台管理器**:
```rust
// 创建生命周期管理器
let lifecycle = Arc::new(LifecycleManager::new());
lifecycle.set_state(LifecycleState::Active);

// 创建平台特定窗口管理器
#[cfg(target_os = "windows")]
let platform_manager = {
    use crate::webview::platform;
    let manager = platform::create_platform_manager(parent_hwnd);
    manager.setup_close_handlers(lifecycle.clone());
    Some(manager)
};
```

## ✨ 主要优势

### 1. **跨平台支持**
- 统一的 API 适用于所有平台
- 平台特定实现隐藏在 trait 后面
- 易于添加新平台支持

### 2. **事件驱动架构**
- 使用 `flume` 通道进行高效通知
- 非阻塞操作
- 减少轮询开销

### 3. **保证资源清理**
- `scopeguard` 确保清理代码执行
- 即使发生 panic 也能正确清理
- RAII 模式增强

### 4. **更好的 DCC 集成**
- 尊重宿主应用的事件循环
- 非阻塞消息处理
- 清晰的职责分离

### 5. **线程安全**
- 所有组件实现 Send + Sync
- 使用 Arc 和 Mutex 进行同步
- 避免数据竞争

## 📊 性能影响

- **零开销**: `scopeguard` 编译为零成本抽象
- **高效通道**: `flume` 比标准库更快
- **非阻塞**: 所有操作都是非阻塞的
- **最小分配**: 谨慎使用 Arc 和 Mutex

## 🔮 未来计划

1. ✅ Windows 平台完整实现
2. ⏳ macOS 平台实现
3. ⏳ Linux 平台实现
4. ⏳ Python API 暴露生命周期事件
5. ⏳ 支持自定义关闭确认对话框
6. ⏳ 优雅关闭与超时机制

## 📝 使用示例

### Python 端使用

```python
from auroraview import AuroraView

# 创建嵌入式 webview
view = AuroraView.create_embedded(
    parent_hwnd=maya_window_handle,
    width=800,
    height=600,
    url="https://example.com"
)

# 定期处理事件
def on_timer():
    if view.process_events():
        print("窗口已关闭")
        view.close()
        return False
    return True
```

## 🎓 技术亮点

1. **Rust 最佳实践**: 使用 RAII、trait、泛型等现代 Rust 特性
2. **零成本抽象**: 性能与手写代码相当
3. **类型安全**: 编译时捕获错误
4. **内存安全**: 无数据竞争、无悬垂指针
5. **可维护性**: 清晰的模块划分和文档

## 📚 相关文档

- [生命周期管理详细文档](./lifecycle_management.md)
- [平台特定实现指南](./platform_implementation.md) (待创建)
- [API 参考](./api_reference.md) (待创建)

