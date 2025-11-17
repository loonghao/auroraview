# Maya Context Menu 问题修复指南

## 🔍 问题诊断

您在 Maya 中看到原生右键菜单的原因是：

**Maya 使用的是旧版本的 AuroraView (v0.2.3)**，该版本没有 `context_menu` 参数支持。

### 证据

1. **Maya 模块路径**: `C:\Users\hallo\Documents\maya\modules\auroraview`
2. **版本**: 0.2.3 (从 `dist/auroraview-0.2.3-cp37-abi3-win_amd64.whl` 可见)
3. **当前开发版本**: 0.2.6 (在 `c:\Users\hallo\Documents\augment-projects\dcc_webview`)

## ✅ 解决方案

### 方案 1: 更新 Maya 模块目录（推荐）

将新编译的 AuroraView 复制到 Maya 模块目录：

```powershell
# 1. 在开发目录构建最新版本
cd C:\Users\hallo\Documents\augment-projects\dcc_webview
maturin build --release --features ext-module,win-webview2

# 2. 复制 Python 文件
xcopy /E /Y python\auroraview C:\Users\hallo\Documents\maya\modules\auroraview\python\auroraview\

# 3. 复制编译的 _core.pyd
copy target\release\auroraview.pyd C:\Users\hallo\Documents\maya\modules\auroraview\python\auroraview\_core.pyd
```

### 方案 2: 修改 Maya 模块路径

让 Maya 直接使用开发目录：

**编辑**: `C:\Users\hallo\Documents\maya\modules\auroraview.mod`

```
+ MAYAVERSION:2024 auroraview 0.2.6 C:/Users/hallo/Documents/augment-projects/dcc_webview
PYTHONPATH +:= python
PYTHONPATH +:= C:/github/auroraview-maya-outliner
```

### 方案 3: 使用 Python 环境变量

在 Maya 启动前设置环境变量：

```python
# Maya Script Editor - 在导入前运行
import sys
sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\python')

# 现在导入会使用新版本
from maya_integration import maya_outliner
outliner = maya_outliner.main(context_menu=False)
```

## 🧪 验证步骤

### 步骤 1: 检查版本

在 Maya Script Editor 中运行：

```python
# 粘贴 check_version.py 的内容并运行
exec(open(r'C:\github\auroraview-maya-outliner\check_version.py').read())
```

**期望输出**:
```
✅ SUCCESS: context_menu parameter works!
✅ ALL CHECKS PASSED!
```

**如果失败**:
```
❌ FAILED: WebView.__new__() got an unexpected keyword argument 'context_menu'
```
→ 说明 Maya 仍在使用旧版本

### 步骤 2: 测试 Maya Outliner

```python
from maya_integration import maya_outliner

# 显式传递 context_menu=False
outliner = maya_outliner.main(context_menu=False)
```

### 步骤 3: 验证菜单被禁用

右键点击 Maya Outliner 中的节点，应该：
- ❌ **不显示** "刷新"、"另存为"、"打印" 等浏览器原生菜单
- ✅ **可以显示** 自定义的 JavaScript 菜单（如果前端实现了）

## 🔧 详细修复步骤（推荐方案 1）

### 1. 构建最新版本

```powershell
cd C:\Users\hallo\Documents\augment-projects\dcc_webview

# 构建 release 版本
maturin build --release --features ext-module,win-webview2
```

### 2. 找到编译的文件

```powershell
# _core.pyd 位置
dir target\release\*.pyd

# 应该看到: auroraview.pyd
```

### 3. 复制到 Maya 模块

```powershell
# 备份旧版本（可选）
xcopy /E C:\Users\hallo\Documents\maya\modules\auroraview C:\Users\hallo\Documents\maya\modules\auroraview.backup\

# 复制 Python 代码
xcopy /E /Y python\auroraview\*.py C:\Users\hallo\Documents\maya\modules\auroraview\python\auroraview\
xcopy /E /Y python\auroraview\*.pyi C:\Users\hallo\Documents\maya\modules\auroraview\python\auroraview\

# 复制编译的 Rust 模块
copy /Y target\release\auroraview.pyd C:\Users\hallo\Documents\maya\modules\auroraview\python\auroraview\_core.pyd
```

### 4. 重启 Maya

**重要**: 必须完全关闭并重启 Maya，以清除：
- Python 模块缓存
- WebView2 运行时缓存

### 5. 验证

在 Maya 中运行 `check_version.py`

## 📝 自动化脚本

创建一个 PowerShell 脚本来自动化更新过程：

```powershell
# update_maya_auroraview.ps1

$DEV_DIR = "C:\Users\hallo\Documents\augment-projects\dcc_webview"
$MAYA_MODULE = "C:\Users\hallo\Documents\maya\modules\auroraview"

Write-Host "Building AuroraView..." -ForegroundColor Cyan
cd $DEV_DIR
maturin build --release --features ext-module,win-webview2

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Copying Python files..." -ForegroundColor Cyan
xcopy /E /Y "$DEV_DIR\python\auroraview\*.py" "$MAYA_MODULE\python\auroraview\"
xcopy /E /Y "$DEV_DIR\python\auroraview\*.pyi" "$MAYA_MODULE\python\auroraview\"

Write-Host "Copying compiled module..." -ForegroundColor Cyan
copy /Y "$DEV_DIR\target\release\auroraview.pyd" "$MAYA_MODULE\python\auroraview\_core.pyd"

Write-Host "Done! Please restart Maya." -ForegroundColor Green
```

使用方法：
```powershell
.\update_maya_auroraview.ps1
```

## 🎯 快速测试（不重启 Maya）

如果不想重启 Maya，可以强制重新加载模块：

```python
# Maya Script Editor
import sys

# 移除所有 auroraview 相关模块
modules_to_remove = [k for k in sys.modules.keys() if 'auroraview' in k]
for mod in modules_to_remove:
    del sys.modules[mod]

# 添加新路径到最前面
sys.path.insert(0, r'C:\Users\hallo\Documents\augment-projects\dcc_webview\python')

# 重新导入
from maya_integration import maya_outliner
outliner = maya_outliner.main(context_menu=False)
```

**注意**: 这种方法可能不完全可靠，建议还是重启 Maya。

---

**Signed-off-by:** Hal Long <hal.long@outlook.com>

