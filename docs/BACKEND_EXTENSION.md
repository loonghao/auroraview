# 后端扩展指南

## 概述

AuroraView 采用插件化的后端架构，允许轻松集成新的渲染引擎。本文档说明如何添加新的渲染后端。

## 架构设计

### 核心抽象层

```rust
pub trait WebViewBackend {
    // 创建后端实例
    fn create(...) -> Result<Self>;
    
    // 加载内容
    fn load_html(&mut self, html: &str) -> Result<()>;
    fn load_url(&mut self, url: &str) -> Result<()>;
    
    // JavaScript 交互
    fn eval_js(&mut self, script: &str) -> Result<()>;
    fn emit(&mut self, event: &str, data: Value) -> Result<()>;
    
    // 事件循环
    fn process_events(&self) -> bool;
    fn run_event_loop_blocking(&mut self);
}
```

### 后端类型

```rust
pub enum RenderingEngine {
    SystemWebView,      // 当前：Wry + WebView2/WebKit
    Servo,              // 未来：Servo 渲染引擎
    Custom,             // 未来：自定义渲染器
}

pub enum BackendType {
    Native { engine: RenderingEngine },
    Qt { engine: RenderingEngine },
}
```

## 添加新后端

### 步骤 1: 创建后端模块

在 `src/webview/backend/` 创建新文件：

```rust
// src/webview/backend/my_engine.rs

use super::WebViewBackend;

pub struct MyEngineBackend {
    // 你的实现
}

impl WebViewBackend for MyEngineBackend {
    fn create(...) -> Result<Self> {
        // 初始化你的渲染引擎
    }
    
    fn load_html(&mut self, html: &str) -> Result<()> {
        // 渲染 HTML
    }
    
    // ... 实现其他方法
}
```

### 步骤 2: 添加 Feature Flag

在 `Cargo.toml` 中添加：

```toml
[features]
my-engine-backend = ["my-engine-crate"]

[dependencies]
my-engine-crate = { version = "1.0", optional = true }
```

### 步骤 3: 注册后端

在 `src/webview/backend/mod.rs` 中：

```rust
#[cfg(feature = "my-engine-backend")]
pub mod my_engine;

pub enum RenderingEngine {
    SystemWebView,
    #[cfg(feature = "my-engine-backend")]
    MyEngine,
}
```

### 步骤 4: 更新配置

允许用户选择后端：

```python
from auroraview import NativeWebView

webview = NativeWebView.standalone(
    title="My App",
    backend="my-engine",  # 指定后端
)
```

## 示例：Servo 后端

### 1. 依赖配置

```toml
[features]
servo-backend = ["servo", "winit"]

[dependencies]
servo = { git = "https://github.com/servo/servo", optional = true }
winit = { version = "0.29", optional = true }
```

### 2. 后端实现

```rust
// src/webview/backend/servo.rs

use servo::Servo;
use winit::window::Window;

pub struct ServoBackend {
    servo: Servo,
    window: Window,
    event_loop: EventLoop<()>,
}

impl WebViewBackend for ServoBackend {
    fn create(config: WebViewConfig, ...) -> Result<Self> {
        // 1. 创建 winit 窗口
        let window = WindowBuilder::new()
            .with_title(&config.title)
            .with_inner_size(LogicalSize::new(config.width, config.height))
            .build(&event_loop)?;
        
        // 2. 初始化 Servo
        let servo = Servo::new(ServoConfig {
            url: config.url.clone(),
            window: &window,
        })?;
        
        Ok(Self { servo, window, event_loop })
    }
    
    fn load_html(&mut self, html: &str) -> Result<()> {
        // 使用 Servo API 加载 HTML
        self.servo.load_html(html)
    }
    
    fn eval_js(&mut self, script: &str) -> Result<()> {
        // 使用 SpiderMonkey 执行 JavaScript
        self.servo.execute_script(script)
    }
    
    fn run_event_loop_blocking(&mut self) {
        // 运行 winit 事件循环
        self.event_loop.run(|event, _, control_flow| {
            // 处理事件
            self.servo.handle_event(&event);
        });
    }
}
```

### 3. IPC 桥接

```rust
impl ServoBackend {
    fn setup_ipc_bridge(&mut self) {
        // 注册 JavaScript → Rust 通信
        self.servo.register_ipc_handler(|message| {
            // 解析消息并调用 Python 回调
            let event: IpcMessage = serde_json::from_str(&message)?;
            self.ipc_handler.handle(event);
        });
    }
    
    fn emit_to_js(&mut self, event: &str, data: Value) -> Result<()> {
        // Rust → JavaScript 通信
        let script = format!(
            "window.dispatchEvent(new CustomEvent('{}', {{ detail: {} }}))",
            event, data
        );
        self.eval_js(&script)
    }
}
```

## 自定义渲染器

### 使用场景

- 集成专有渲染引擎
- 实验性渲染技术
- DCC 特定的渲染管线

### 实现步骤

```rust
use auroraview::backend::{CustomRenderer, CustomBackend};

// 1. 实现 CustomRenderer trait
struct MyRenderer {
    // 你的渲染器状态
}

impl CustomRenderer for MyRenderer {
    fn initialize(&mut self, config: &WebViewConfig) -> Result<()> {
        // 初始化渲染器
    }
    
    fn render_html(&mut self, html: &str) -> Result<()> {
        // 渲染 HTML
    }
    
    fn execute_script(&mut self, script: &str) -> Result<()> {
        // 执行 JavaScript
    }
    
    fn process_events(&mut self) -> bool {
        // 处理事件
        false
    }
    
    fn run_event_loop(&mut self) {
        // 运行事件循环
    }
}

// 2. 创建自定义后端
let renderer = MyRenderer::new();
let backend = CustomBackend::new(
    renderer,
    config,
    ipc_handler,
    message_queue,
)?;
```

## 性能考虑

### 1. 首屏加载

不同后端的首屏加载时间：

| 后端 | 初始化时间 | 首次渲染 | 总计 |
|------|-----------|---------|------|
| Wry (WebView2) | 200-300ms | 200-300ms | 400-600ms |
| Servo | 100-200ms | 100-200ms | 200-400ms |
| Custom | 取决于实现 | 取决于实现 | - |

### 2. 内存占用

| 后端 | 基础内存 | 每页面 | 总计 |
|------|---------|--------|------|
| Wry | 50-100MB | +20-50MB | 70-150MB |
| Servo | 150-300MB | +30-60MB | 180-360MB |

### 3. 优化建议

```rust
impl MyBackend {
    // 使用 loading 页面
    fn show_loading(&mut self) {
        self.load_html(LOADING_HTML);
    }
    
    // 异步加载实际内容
    async fn load_content_async(&mut self, html: &str) {
        tokio::spawn(async move {
            // 加载内容
        });
    }
    
    // 批量处理事件
    fn process_events_batch(&mut self) {
        let events = self.collect_events();
        for event in events {
            self.handle_event(event);
        }
    }
}
```

## 测试

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_backend_creation() {
        let backend = MyBackend::create(config, ipc, queue).unwrap();
        assert!(backend.is_initialized());
    }
    
    #[test]
    fn test_html_loading() {
        let mut backend = create_test_backend();
        backend.load_html("<h1>Test</h1>").unwrap();
        // 验证 HTML 已加载
    }
}
```

### 集成测试

```python
# tests/test_backends.py

def test_servo_backend():
    webview = NativeWebView.standalone(
        title="Servo Test",
        backend="servo",
    )
    webview.load_html("<h1>Hello Servo</h1>")
    # 验证渲染结果
```

## 最佳实践

### 1. 错误处理

```rust
impl WebViewBackend for MyBackend {
    fn load_html(&mut self, html: &str) -> Result<()> {
        self.renderer.load_html(html)
            .map_err(|e| {
                tracing::error!("Failed to load HTML: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })
    }
}
```

### 2. 日志记录

```rust
impl MyBackend {
    fn initialize(&mut self) -> Result<()> {
        tracing::info!("[LAUNCH] Initializing MyBackend");
        
        let start = Instant::now();
        self.setup()?;
        
        tracing::info!("[OK] MyBackend initialized in {:?}", start.elapsed());
        Ok(())
    }
}
```

### 3. 性能监控

```rust
use crate::performance::PerformanceTracker;

impl MyBackend {
    fn load_html(&mut self, html: &str) -> Result<()> {
        let tracker = PerformanceTracker::new();
        
        tracker.mark("parse_start");
        self.parse_html(html)?;
        tracker.mark("parse_end");
        
        tracker.mark("render_start");
        self.render()?;
        tracker.mark("render_end");
        
        tracker.print_report();
        Ok(())
    }
}
```

## 发布清单

在发布新后端之前，确保：

- [ ] 实现了所有 `WebViewBackend` 方法
- [ ] 添加了 feature flag
- [ ] 编写了单元测试
- [ ] 编写了集成测试
- [ ] 添加了文档和示例
- [ ] 性能测试通过
- [ ] 内存泄漏检查通过
- [ ] 跨平台测试通过

## 参考资料

- [WebViewBackend Trait 文档](../src/webview/backend/mod.rs)
- [Servo 后端示例](../src/webview/backend/servo.rs)
- [自定义后端示例](../src/webview/backend/custom.rs)
- [性能优化指南](./PERFORMANCE_OPTIMIZATION.md)
- [Servo 评估报告](./SERVO_EVALUATION.md)

