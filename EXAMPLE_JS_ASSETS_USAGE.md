# JavaScript Assets 使用示例

## 📝 如何使用新的 js_assets 模块

### 示例 1: 在 embedded.rs 中使用

**修改前**（旧代码）：
```rust
// src/webview/embedded.rs (约 275-500 行)

let mut event_bridge_script = r#"
    (function() {
        console.log('[AuroraView] Initializing event bridge...');
        
        const eventHandlers = new Map();
        let auroraviewCallIdCounter = 0;
        const auroraviewPendingCalls = new Map();
        
        // ... 200+ 行 JavaScript 代码 ...
        
        window.auroraview = {
            call: function(method, params) { /* ... */ },
            send_event: function(event, detail) { /* ... */ },
            on: function(event, handler) { /* ... */ },
            trigger: function(event, detail) { /* ... */ },
            api: {}
        };
    })();
"#.to_string();

// 添加禁用右键菜单的代码
if !config.context_menu {
    tracing::info!("[OK] [create_embedded] Adding JavaScript to disable context menu");
    event_bridge_script.push_str(
        r#"
    // Disable native context menu
    (function() {
        document.addEventListener('contextmenu', function(e) {
            e.preventDefault();
            console.log('[AuroraView] Native context menu disabled');
            return false;
        }, false);
        console.log('[AuroraView] ✓ Context menu disabled');
    })();
    "#,
    );
}
```

**修改后**（新代码）：
```rust
// src/webview/embedded.rs

use super::js_assets;

// 使用 js_assets 模块构建初始化脚本
let event_bridge_script = js_assets::build_init_script(&config);

// 就这么简单！所有逻辑都在 js_assets 模块中处理
```

**代码减少**：从 ~50 行减少到 **2 行**！

---

### 示例 2: 在 backend/native.rs 中使用

**修改前**：
```rust
// src/webview/backend/native.rs (约 545-757 行)

let mut event_bridge_script = r#"
    (function() {
        // ... 同样的 200+ 行 JavaScript 代码（重复！）...
    })();
"#.to_string();

if !config.context_menu {
    event_bridge_script.push_str(/* ... 同样的禁用菜单代码 ... */);
}
```

**修改后**：
```rust
// src/webview/backend/native.rs

use crate::webview::js_assets;

let event_bridge_script = js_assets::build_init_script(&config);
```

---

### 示例 3: 在 standalone.rs 中使用

**修改前**：
```rust
// src/webview/standalone.rs (约 78-270 行)

let event_bridge_script = r#"
    (function() {
        // ... 又是同样的 200+ 行代码（第三次重复！）...
    })();
"#;
```

**修改后**：
```rust
// src/webview/standalone.rs

use super::js_assets;

let event_bridge_script = js_assets::build_init_script(&config);
```

---

## 🎯 完整的重构示例

### 文件：`src/webview/embedded.rs`

```rust
// 在文件顶部添加 import
use super::js_assets;

// 找到这段代码（约 275 行）：
pub fn create_embedded(
    config: WebViewConfig,
    parent_hwnd: isize,
) -> Result<WebViewInner, Box<dyn std::error::Error>> {
    // ... 前面的代码保持不变 ...

    // ===== 修改这里 =====
    // 旧代码：删除整个 event_bridge_script 字符串定义（约 200 行）
    // 新代码：使用 js_assets 模块
    let event_bridge_script = js_assets::build_init_script(&config);
    // ===== 修改结束 =====

    // 后面的代码保持不变
    let webview_builder = webview_builder
        .with_initialization_script(&event_bridge_script);
    
    // ... 剩余代码 ...
}
```

---

## 📊 重构前后对比

### 代码量对比

| 文件 | 修改前 | 修改后 | 减少 |
|------|--------|--------|------|
| `embedded.rs` | ~500 行 | ~300 行 | **-200 行** |
| `backend/native.rs` | ~757 行 | ~557 行 | **-200 行** |
| `standalone.rs` | ~421 行 | ~221 行 | **-200 行** |
| **总计** | **1678 行** | **1078 行** | **-600 行** |

### 新增文件

| 文件 | 行数 | 说明 |
|------|------|------|
| `assets/js/core/event_bridge.js` | ~150 行 | 核心 event bridge |
| `assets/js/features/context_menu.js` | ~25 行 | 禁用右键菜单 |
| `assets/js/features/legacy_compat.js` | ~65 行 | 向后兼容 |
| `src/webview/js_assets.rs` | ~150 行 | Rust 资源管理 |
| **总计** | **~390 行** | |

### 净收益

- **代码减少**: 600 - 390 = **210 行**
- **重复消除**: 3 份重复代码 → 1 份
- **可维护性**: ⭐⭐⭐⭐⭐ (大幅提升)

---

## ✅ 测试验证

### 1. 编译测试

```bash
# 确保所有文件都能正确编译
cargo build --release --features ext-module,win-webview2
```

### 2. 单元测试

```bash
# 运行 js_assets 模块的单元测试
cargo test js_assets
```

### 3. 集成测试

```bash
# 测试 WebView 创建
python -c "from auroraview._core import WebView; w = WebView(context_menu=False); print('OK')"

# 测试 Maya 集成
# 在 Maya 中运行
from maya_integration import maya_outliner
outliner = maya_outliner.main(context_menu=False)
```

### 4. JavaScript 验证

在 WebView 中打开开发者工具，检查控制台输出：

```
[AuroraView] Initializing event bridge...
[AuroraView] ✓ Event bridge initialized
[AuroraView] ✓ API: window.auroraview.call() / .send_event() / .on()
[AuroraView] Disabling native context menu...
[AuroraView] ✓ Context menu disabled
[AuroraView] Initializing legacy compatibility layer...
[AuroraView] ✓ Legacy compatibility layer initialized
```

---

## 🚀 下一步优化（可选）

### 1. 添加 JavaScript 工具链

```bash
# 在项目根目录创建 package.json
npm init -y

# 安装开发工具
npm install --save-dev eslint prettier

# 配置 ESLint
npx eslint --init
```

### 2. 添加 pre-commit hook

```bash
# .git/hooks/pre-commit
#!/bin/bash
# 检查 JavaScript 代码格式
npx prettier --check "assets/js/**/*.js"
npx eslint "assets/js/**/*.js"
```

### 3. 代码压缩（可选）

如果需要减小二进制文件大小，可以使用 `build.rs` 压缩 JavaScript：

```rust
// build.rs
use std::fs;

fn main() {
    // 读取 JS 文件
    let js = fs::read_to_string("assets/js/core/event_bridge.js").unwrap();
    
    // 简单的压缩：移除注释和多余空白
    let minified = js
        .lines()
        .filter(|line| !line.trim().starts_with("//"))
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ");
    
    // 写入到 OUT_DIR
    let out_dir = std::env::var("OUT_DIR").unwrap();
    fs::write(format!("{}/event_bridge.min.js", out_dir), minified).unwrap();
}
```

---

**Signed-off-by: Hal Long <hal.long@outlook.com>**

