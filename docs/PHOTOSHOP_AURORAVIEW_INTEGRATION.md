# Photoshop + AuroraView 深度集成方案

> ✅ **实现状态**: 已完成 POC 实现,代码位于 `examples/photoshop_auroraview/`
>
> 📖 **快速开始**: 查看 [examples/photoshop_auroraview/QUICK_START.md](../examples/photoshop_auroraview/QUICK_START.md)

## 🎯 核心理念

**利用 AuroraView 的核心优势**:
1. ✅ **WebView UI**: 使用 AuroraView 的 WebView 作为 Photoshop 工具面板
2. ✅ **Python 生态**: 利用 Python 强大的图像处理库 (Pillow, OpenCV, NumPy)
3. ✅ **IPC 架构**: 复用 AuroraView 的双向通信机制
4. ✅ **快速开发**: 使用 HTML/CSS/JavaScript 快速构建 UI

## 🏗️ 新架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    Adobe Photoshop                          │
│  ┌────────────────────────────────────────────────────┐    │
│  │         UXP Plugin (Minimal Bridge)                │    │
│  │  - WebSocket Client (连接到 Python)                │    │
│  │  - Photoshop API Wrapper                           │    │
│  └──────────────────┬─────────────────────────────────┘    │
└─────────────────────┼──────────────────────────────────────┘
                      │ WebSocket
                      │
┌─────────────────────▼──────────────────────────────────────┐
│              Python Backend (核心逻辑层)                    │
│  ┌────────────────────────────────────────────────────┐   │
│  │  photoshop_bridge.py                               │   │
│  │  - WebSocket Server (接收 Photoshop 消息)          │   │
│  │  - 图像处理逻辑 (Pillow, OpenCV, NumPy)            │   │
│  │  - AuroraView WebView 控制                         │   │
│  └──────────────────┬─────────────────────────────────┘   │
└─────────────────────┼──────────────────────────────────────┘
                      │ Python API
                      │
┌─────────────────────▼──────────────────────────────────────┐
│              AuroraView WebView (UI 层)                    │
│  ┌────────────────────────────────────────────────────┐   │
│  │  HTML/CSS/JavaScript UI                            │   │
│  │  - React/Vue 组件                                   │   │
│  │  - 图像预览                                         │   │
│  │  - 参数调整面板                                     │   │
│  └──────────────────┬─────────────────────────────────┘   │
└─────────────────────┼──────────────────────────────────────┘
                      │ IPC (CustomEvent)
                      │
                  (双向通信)
```

## 🔑 关键优势

### 1. 使用 AuroraView WebView 作为 UI
**不再需要 UXP 插件的 UI**,只需要一个最小的桥接层:

```python
# photoshop_tool.py
from auroraview import WebView
import asyncio
import websockets

class PhotoshopTool:
    def __init__(self):
        # 创建 AuroraView WebView 作为工具面板
        self.webview = WebView(
            title="Photoshop AI Tools",
            width=400,
            height=800,
            url="http://localhost:5173",  # Vite dev server
            debug=True
        )
        
        # 注册 Python 回调
        self.webview.bind("process_image", self.process_image)
        self.webview.bind("apply_filter", self.apply_filter)
        
    def process_image(self, params):
        """使用 Python 图像处理库"""
        import cv2
        import numpy as np
        from PIL import Image
        
        # 从 Photoshop 获取图像数据
        image_data = self.get_photoshop_image()
        
        # 使用 OpenCV 处理
        img = cv2.imdecode(np.frombuffer(image_data, np.uint8), cv2.IMREAD_COLOR)
        processed = cv2.GaussianBlur(img, (5, 5), 0)
        
        # 发送回 Photoshop
        self.send_to_photoshop(processed)
        
        return {"status": "success"}
```

### 2. 利用 Python 图像处理生态

**可以直接使用**:
- **Pillow**: 图像基础操作
- **OpenCV**: 计算机视觉算法
- **NumPy**: 数值计算
- **scikit-image**: 科学图像处理
- **PyTorch/TensorFlow**: AI 模型推理

```python
def apply_ai_filter(self, image_data, model_name):
    """使用 AI 模型处理图像"""
    import torch
    from torchvision import transforms
    
    # 加载预训练模型
    model = torch.hub.load('pytorch/vision', model_name)
    
    # 处理图像
    transform = transforms.Compose([...])
    result = model(transform(image_data))
    
    return result
```

### 3. 快速 UI 开发

使用现代前端技术栈:

```typescript
// src/App.tsx (React + TypeScript)
import { useState } from 'react';
import { Button, Slider } from '@/components/ui';

function PhotoshopPanel() {
  const [blur, setBlur] = useState(5);
  
  const applyBlur = async () => {
    // 调用 Python 后端
    const result = await window.auroraview.call('apply_filter', {
      type: 'gaussian_blur',
      radius: blur
    });
    
    console.log('Filter applied:', result);
  };
  
  return (
    <div className="p-4">
      <h2>AI Image Tools</h2>
      <Slider value={blur} onChange={setBlur} />
      <Button onClick={applyBlur}>Apply Blur</Button>
    </div>
  );
}
```

## 📦 完整示例架构

### 文件结构

```
examples/photoshop_auroraview/
├── python/
│   ├── photoshop_bridge.py      # WebSocket 服务器
│   ├── image_processor.py       # 图像处理逻辑
│   ├── photoshop_tool.py        # 主入口
│   └── requirements.txt         # Python 依赖
├── uxp_plugin/                  # 最小 UXP 桥接
│   ├── manifest.json
│   └── index.js                 # 仅 WebSocket 客户端
├── ui/                          # WebView UI (Vite + React)
│   ├── src/
│   │   ├── App.tsx
│   │   ├── components/
│   │   └── main.tsx
│   ├── package.json
│   └── vite.config.ts
└── README.md
```

### Python 后端示例

```python
# photoshop_bridge.py
import asyncio
import websockets
import json
from auroraview import WebView

class PhotoshopBridge:
    def __init__(self):
        self.photoshop_clients = set()
        self.webview = None
        
    async def start_server(self):
        """启动 WebSocket 服务器接收 Photoshop 消息"""
        async with websockets.serve(self.handle_photoshop, "localhost", 9001):
            await asyncio.Future()  # Run forever
            
    async def handle_photoshop(self, websocket):
        """处理来自 Photoshop UXP 的消息"""
        self.photoshop_clients.add(websocket)
        try:
            async for message in websocket:
                data = json.loads(message)
                await self.process_message(data, websocket)
        finally:
            self.photoshop_clients.remove(websocket)
            
    async def process_message(self, data, websocket):
        """处理消息并调用图像处理"""
        action = data.get('action')
        
        if action == 'get_image':
            # 通知 WebView 更新预览
            if self.webview:
                self.webview.evaluate_js(f"updatePreview({json.dumps(data)})")
                
        elif action == 'layer_created':
            # 使用 Python 处理图层数据
            result = self.process_layer(data['layer_data'])
            await websocket.send(json.dumps(result))
            
    def create_ui(self):
        """创建 AuroraView WebView UI"""
        self.webview = WebView(
            title="Photoshop AI Tools",
            width=400,
            height=800,
            url="http://localhost:5173",
            debug=True
        )
        
        # 绑定 Python 函数到 JavaScript
        self.webview.bind("apply_filter", self.apply_filter)
        self.webview.bind("process_ai", self.process_ai)
        
        self.webview.show()
        
    def apply_filter(self, params):
        """应用图像滤镜"""
        import cv2
        import numpy as np
        
        # 实现滤镜逻辑
        # ...
        
        return {"status": "success", "preview": "base64_image"}
```

## 🚀 使用流程

### 1. 启动 Python 后端

```bash
cd examples/photoshop_auroraview/python
pip install -r requirements.txt
python photoshop_tool.py
```

### 2. 启动前端开发服务器

```bash
cd examples/photoshop_auroraview/ui
npm install
npm run dev
```

### 3. 加载 UXP 插件

UXP 插件现在非常简单,只负责转发消息:

```javascript
// uxp_plugin/index.js
const socket = new WebSocket('ws://localhost:9001');

socket.onmessage = (event) => {
    const data = JSON.parse(event.data);
    // 执行 Photoshop 操作
    executePhotoshopAction(data);
};

// 发送图层数据到 Python
function sendLayerData(layer) {
    socket.send(JSON.stringify({
        action: 'layer_created',
        layer_data: layer
    }));
}
```

## 🎨 实际应用场景

### 场景 1: AI 图像增强

```python
def enhance_image(self, image_data):
    """使用 AI 模型增强图像"""
    from PIL import Image, ImageEnhance
    import io
    
    img = Image.open(io.BytesIO(image_data))
    
    # 自动调整对比度
    enhancer = ImageEnhance.Contrast(img)
    img = enhancer.enhance(1.5)
    
    # 锐化
    enhancer = ImageEnhance.Sharpness(img)
    img = enhancer.enhance(2.0)
    
    # 返回处理后的图像
    return self.image_to_base64(img)
```

### 场景 2: 批量处理

```python
def batch_process(self, layers, operation):
    """批量处理多个图层"""
    results = []
    
    for layer in layers:
        # 使用 NumPy 加速处理
        processed = self.apply_operation(layer, operation)
        results.append(processed)
        
        # 更新 UI 进度
        self.webview.evaluate_js(f"updateProgress({len(results)}/{len(layers)})")
    
    return results
```

### 场景 3: 实时预览

```typescript
// UI 组件
function FilterPreview() {
  const [preview, setPreview] = useState(null);
  
  const updatePreview = async (params) => {
    // 调用 Python 生成预览
    const result = await window.auroraview.call('generate_preview', params);
    setPreview(result.preview);
  };
  
  return <img src={preview} alt="Preview" />;
}
```

## 📊 对比优势

| 特性 | 纯 UXP 方案 | AuroraView 集成方案 |
|------|------------|-------------------|
| UI 开发 | UXP 限制的 HTML/CSS | 完整现代前端技术栈 |
| 图像处理 | JavaScript (慢) | Python + NumPy/OpenCV (快) |
| AI 模型 | 不支持 | PyTorch/TensorFlow |
| 开发速度 | 慢 (UXP 限制多) | 快 (Vite HMR) |
| 调试 | UXP Developer Tool | Chrome DevTools |
| 生态系统 | 有限 | Python 完整生态 |

## 🎯 下一步实现

我将创建完整的示例代码,包括:
1. ✅ Python 后端 (WebSocket + 图像处理)
2. ✅ AuroraView WebView UI (React + TypeScript)
3. ✅ 最小 UXP 桥接插件
4. ✅ 完整的双向通信示例

这样你就可以:
- 使用 Python 的强大图像处理能力
- 用现代前端技术快速开发 UI
- 复用 AuroraView 的 IPC 架构
- 享受 Vite 的热更新开发体验

准备好了吗? 我现在就开始创建完整的示例! 🚀

