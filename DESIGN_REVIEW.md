# AuroraView 代码规范审查报告

## 项目规范要求

根据 `.augment/rules/架构.md`，项目有以下关键要求：

1. **避免使用 'optimized', 'fixed' 等词** - 函数和模块名称应该简短精炼
2. **函数名称设计** - 应该简短精炼，多使用行业标准
3. **代码兼容性** - 需要兼容 Python 3.7 和所有 DCC 环境
4. **代码质量** - 每次提交需要解决 Lint 和 clippy 的风格问题

## 发现的规范违规

### 1. ❌ 模块命名违规

#### `src/ipc/optimized.rs` - 使用了禁用词 "optimized"
- **问题**: 模块名称包含 "optimized"，违反规范
- **当前结构**:
  ```
  src/ipc/
  ├── backend.rs
  ├── handler.rs
  ├── message_queue.rs
  ├── optimized.rs          ← 违规
  ├── threaded.rs
  └── process.rs
  ```
- **建议**: 重命名为 `batching.rs` 或 `batch.rs`（更准确反映功能）
- **影响范围**:
  - `src/ipc/mod.rs` - 需要更新模块声明
  - 所有导入该模块的文件
  - 相关单元测试

#### `src/performance/` - 模块名称过于通用
- **问题**: "performance" 是通用术语，不够精炼
- **当前结构**:
  ```
  src/
  ├── performance/
  │   └── mod.rs            ← 通用名称
  ```
- **建议**: 重命名为 `metrics.rs` 或 `timing.rs`（更具体）
- **影响范围**:
  - `src/lib.rs` - 需要更新模块声明
  - 所有导入该模块的文件
  - 相关单元测试

### 2. ⚠️ 函数命名冗长

#### `src/performance/mod.rs` 中的函数名称
- `mark_window_created()` - 可简化为 `mark_window()`
- `mark_webview_created()` - 可简化为 `mark_webview()`
- `mark_html_loaded()` - 可简化为 `mark_html()`
- `mark_js_initialized()` - 可简化为 `mark_js()`
- `mark_first_paint()` - 已经简洁 ✓
- `mark_window_shown()` - 可简化为 `mark_shown()`
- `time_to_window()` - 可简化为 `window_time()`
- `time_to_webview()` - 可简化为 `webview_time()`
- `time_to_html()` - 可简化为 `html_time()`
- `time_to_js()` - 已经简洁 ✓
- `time_to_first_paint()` - 可简化为 `paint_time()`
- `time_to_shown()` - 可简化为 `shown_time()`
- `print_report()` - 已经简洁 ✓

#### `src/ipc/optimized.rs` 中的函数名称
- `json_to_python()` - 已经简洁 ✓
- `OptimizedPythonCallback` - 应改为 `BatchedCallback` 或 `PythonCallback`
- `OptimizedIpcMessage` - 应改为 `BatchedMessage` 或 `IpcMessage`
- `OptimizedIpcHandler` - 应改为 `BatchedHandler` 或 `IpcHandler`

### 3. ⚠️ 类型别名冗长

#### `src/performance/mod.rs`
```rust
pub type PerformanceTracker = Arc<Mutex<PerformanceMetrics>>;
```
- **问题**: 类型别名名称过长
- **建议**: 改为 `pub type Tracker = Arc<Mutex<Metrics>>;`

### 4. ⚠️ 模块结构不清晰

#### `src/webview/mod.rs` 中的 TODO
```rust
pub(crate) mod embedded; // TODO: Remove after migration to backend::native
```
- **问题**: 有待清理的遗留代码
- **建议**: 完成迁移或明确标记为已弃用

### 5. ⚠️ 未使用的导出

#### `src/webview/mod.rs`
```rust
#[allow(unused_imports)]
pub use backend::{BackendType, WebViewBackend};
```
- **问题**: 导出但未使用，需要 `#[allow]` 属性
- **建议**: 要么使用，要么删除

## 改进优先级

### 高优先级 (必须修复)
1. ✅ 重命名 `src/ipc/optimized.rs` → `src/ipc/batch.rs`
2. ✅ 重命名 `src/performance/` → `src/metrics/`
3. ✅ 更新所有导入和引用

### 中优先级 (应该修复)
1. 简化函数名称（mark_* 和 time_to_* 系列）
2. 重命名类型别名（PerformanceTracker → Tracker）
3. 重命名结构体（OptimizedXxx → BatchedXxx）

### 低优先级 (可以考虑)
1. 清理 embedded 模块的 TODO
2. 移除未使用的导出或明确其用途

## 影响分析

### 需要更新的文件

#### 模块重命名影响
- `src/lib.rs` - 模块声明
- `src/ipc/mod.rs` - 模块声明和导出
- `src/ipc/batch.rs` (原 optimized.rs) - 所有内部引用
- `src/metrics/mod.rs` (原 performance/mod.rs) - 所有内部引用
- 所有导入这些模块的文件

#### 函数重命名影响
- `src/metrics/mod.rs` - 函数定义
- 所有调用这些函数的代码
- 所有相关的单元测试

#### 测试文件需要更新
- `src/utils/mod.rs` - 如果有相关测试
- `src/performance/mod.rs` 中的测试（需要迁移到 metrics）
- 任何集成测试

## 建议的重构步骤

1. **第一步**: 重命名模块
   - `src/ipc/optimized.rs` → `src/ipc/batch.rs`
   - `src/performance/` → `src/metrics/`

2. **第二步**: 更新所有导入
   - 更新 `src/lib.rs`
   - 更新 `src/ipc/mod.rs`
   - 更新所有 use 语句

3. **第三步**: 简化函数名称
   - 在 metrics 模块中重命名函数
   - 在 batch 模块中重命名类型和函数

4. **第四步**: 更新单元测试
   - 更新所有测试中的导入
   - 更新所有测试中的函数调用

5. **第五步**: 验证
   - 运行 `cargo test`
   - 运行 `cargo clippy`
   - 运行 Python 测试

## 预期收益

- ✅ 代码更符合项目规范
- ✅ 函数名称更简洁易读
- ✅ 模块结构更清晰
- ✅ 减少 clippy 警告
- ✅ 提高代码可维护性

