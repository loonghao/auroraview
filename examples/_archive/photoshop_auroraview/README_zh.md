# Photoshop + AuroraView 深度集成

[![English Docs](https://img.shields.io/badge/docs-English-blue)](./README.md)

**Adobe Photoshop 与 AuroraView WebView 和 Python 生态的深度集成。**

## 🎯 核心特性

- ✅ **AuroraView WebView UI**: 现代 React UI,支持 Vite 热更新
- ✅ **Python 图像处理**: 利用 Pillow、OpenCV、NumPy
- ✅ **最小 UXP 桥接**: 轻量级 Photoshop 插件(仅 WebSocket)
- ✅ **双向通信**: Python ↔ Photoshop ↔ WebView
- ✅ **快速开发**: TypeScript + React + Vite

## 🏗️ 架构设计

```
┌─────────────────────────────────────────────────────────┐
│         Adobe Photoshop (UXP 插件)                      │
│  - 最小 WebSocket 桥接                                  │
│  - Photoshop API 包装                                   │
└──────────────────┬──────────────────────────────────────┘
                   │ WebSocket
┌──────────────────▼──────────────────────────────────────┐
│         Python 后端 (photoshop_tool.py)                 │
│  - WebSocket 服务器                                     │
│  - 图像处理 (Pillow, OpenCV)                            │
│  - AuroraView WebView 控制                              │
└──────────────────┬──────────────────────────────────────┘
                   │ Python API
┌──────────────────▼──────────────────────────────────────┐
│         AuroraView WebView (React UI)                   │
│  - 现代 UI (React + TypeScript)                         │
│  - 实时预览                                             │
│  - 滤镜控制                                             │
└─────────────────────────────────────────────────────────┘
```

## 📦 项目结构

```
photoshop_auroraview/
├── python/                      # Python 后端
│   ├── photoshop_bridge.py     # WebSocket 服务器
│   ├── image_processor.py      # 图像处理 (Pillow, OpenCV)
│   ├── photoshop_tool.py       # 主入口
│   └── requirements.txt        # Python 依赖
├── ui/                         # WebView UI (React + Vite)
│   ├── src/
│   │   ├── App.tsx            # 主 React 组件
│   │   ├── App.css            # 样式
│   │   └── main.tsx           # 入口
│   ├── package.json
│   └── vite.config.ts
├── uxp_plugin/                # 最小 UXP 桥接
│   ├── manifest.json
│   ├── index.html
│   └── index.js               # 仅 WebSocket 客户端
└── README.md
```

## 🚀 快速开始

### 环境要求

- Python 3.8+
- Node.js 18+
- Adobe Photoshop 2024+
- UXP Developer Tool

### 步骤 1: 安装 Python 依赖

```bash
cd python
pip install -r requirements.txt
```

### 步骤 2: 安装 UI 依赖

```bash
cd ui
npm install
```

### 步骤 3: 启动开发服务器

**终端 1 - 启动 UI 开发服务器:**
```bash
cd ui
npm run dev
```

**终端 2 - 启动 Python 后端:**
```bash
cd python
python photoshop_tool.py
```

你应该看到:
- Vite 开发服务器运行在 `http://localhost:5173`
- AuroraView WebView 窗口打开
- WebSocket 服务器监听 `ws://localhost:9001`

### 步骤 4: 加载 UXP 插件

1. 打开 **UXP Developer Tool**
2. 点击 **Add Plugin**
3. 选择 `uxp_plugin/manifest.json`
4. 点击 **Load**
5. 在 Photoshop 中: **插件 → AuroraView (Minimal)**
6. 点击 **Connect to Python**

## 🎨 使用示例

### 应用高斯模糊

1. 在 Photoshop 中打开图像
2. 在 AuroraView UI 中,点击 **Get Image from Photoshop**
3. 调整模糊半径滑块
4. 点击 **Apply Blur**
5. 在 WebView 中查看实时预览

### 增强对比度

1. 加载图像
2. 调整对比度因子滑块
3. 点击 **Enhance Contrast**
4. 预览立即更新

### 边缘检测

1. 加载图像
2. 点击 **Detect Edges**
3. 查看 Canny 边缘检测结果

## 🔧 开发指南

### 添加新的图像滤镜

**1. 在 `image_processor.py` 中添加 Python 函数:**

```python
def my_custom_filter(self, image_data: str, param: float) -> Dict[str, Any]:
    img = self.base64_to_image(image_data)
    # 你的处理逻辑
    result = self.image_to_base64(processed_img)
    return {"status": "success", "preview": f"data:image/png;base64,{result}"}
```

**2. 在 `photoshop_tool.py` 中注册:**

```python
def apply_filter(params):
    if filter_type == 'my_custom_filter':
        result = self.processor.my_custom_filter(image_data, param)
    return result
```

**3. 在 `App.tsx` 中添加 UI 控制:**

```typescript
const applyCustomFilter = async () => {
  const result = await window.auroraview.call('apply_filter', {
    type: 'my_custom_filter',
    param: value,
    image: preview
  });
  setPreview(result.preview);
};
```

## 🎯 相比纯 UXP 的优势

| 特性 | 纯 UXP | AuroraView 集成 |
|------|--------|----------------|
| UI 框架 | 受限的 HTML/CSS | 完整 React + TypeScript |
| 图像处理 | JavaScript (慢) | Python + NumPy (快) |
| AI/ML 支持 | ❌ | ✅ PyTorch/TensorFlow |
| 开发体验 | UXP 重载 | Vite HMR (即时) |
| 调试 | UXP DevTool | Chrome DevTools |
| Python 生态 | ❌ | ✅ 完全访问 |

## 📚 可用的 Python 库

- **Pillow**: 图像处理
- **OpenCV**: 计算机视觉
- **NumPy**: 数值计算
- **scikit-image**: 科学图像处理
- **PyTorch/TensorFlow**: 深度学习 (可选)

## 🔍 故障排除

### WebView 无法打开

- 检查 Python 后端是否运行
- 验证 AuroraView 已安装: `pip install auroraview`

### UXP 插件无法连接

- 确保 Python 后端正在运行
- 检查 WebSocket 服务器在端口 9001
- 验证 `manifest.json` 中的网络权限

### 图像处理失败

- 安装所需库: `pip install Pillow opencv-python numpy`
- 检查 Python 控制台错误

## 📖 下一步

- 添加更多图像滤镜
- 集成 AI 模型 (风格迁移、超分辨率)
- 实现批处理
- 添加导出功能
- 创建自定义 Photoshop 动作

## 🔗 参考资料

- [AuroraView 文档](../../README.md)
- [Adobe UXP](https://developer.adobe.com/photoshop/uxp/)
- [Pillow 文档](https://pillow.readthedocs.io/)
- [OpenCV Python](https://docs.opencv.org/4.x/d6/d00/tutorial_py_root.html)

## 📄 许可证

AuroraView 项目的一部分。

