# Photoshop + AuroraView Integration

[![中文文档](https://img.shields.io/badge/docs-中文-blue)](./README_zh.md)

**Deep integration of Adobe Photoshop with AuroraView WebView and Python ecosystem.**

## 🎯 Key Features

- ✅ **AuroraView WebView UI**: Modern React UI with Vite hot reload
- ✅ **Python Image Processing**: Leverage Pillow, OpenCV, NumPy
- ✅ **Minimal UXP Bridge**: Lightweight Photoshop plugin (just WebSocket)
- ✅ **Bidirectional Communication**: Python ↔ Photoshop ↔ WebView
- ✅ **Fast Development**: TypeScript + React + Vite

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│         Adobe Photoshop (UXP Plugin)                    │
│  - Minimal WebSocket bridge                             │
│  - Photoshop API wrapper                                │
└──────────────────┬──────────────────────────────────────┘
                   │ WebSocket
┌──────────────────▼──────────────────────────────────────┐
│         Python Backend (photoshop_tool.py)              │
│  - WebSocket Server                                     │
│  - Image Processing (Pillow, OpenCV)                    │
│  - AuroraView WebView Control                           │
└──────────────────┬──────────────────────────────────────┘
                   │ Python API
┌──────────────────▼──────────────────────────────────────┐
│         AuroraView WebView (React UI)                   │
│  - Modern UI (React + TypeScript)                       │
│  - Real-time preview                                    │
│  - Filter controls                                      │
└─────────────────────────────────────────────────────────┘
```

## 📦 Project Structure

```
photoshop_auroraview/
├── python/                      # Python backend
│   ├── photoshop_bridge.py     # WebSocket server
│   ├── image_processor.py      # Image processing (Pillow, OpenCV)
│   ├── photoshop_tool.py       # Main entry point
│   └── requirements.txt        # Python dependencies
├── ui/                         # WebView UI (React + Vite)
│   ├── src/
│   │   ├── App.tsx            # Main React component
│   │   ├── App.css            # Styles
│   │   └── main.tsx           # Entry point
│   ├── package.json
│   └── vite.config.ts
├── uxp_plugin/                # Minimal UXP bridge
│   ├── manifest.json
│   ├── index.html
│   └── index.js               # WebSocket client only
└── README.md
```

## 🚀 Quick Start

### Prerequisites

- Python 3.8+
- Node.js 18+
- Adobe Photoshop 2024+
- UXP Developer Tool

### Step 1: Install Python Dependencies

```bash
cd python
pip install -r requirements.txt
```

### Step 2: Install UI Dependencies

```bash
cd ui
npm install
```

### Step 3: Start Development Servers

**Terminal 1 - Start UI Dev Server:**
```bash
cd ui
npm run dev
```

**Terminal 2 - Start Python Backend:**
```bash
cd python
python photoshop_tool.py
```

You should see:
- Vite dev server running on `http://localhost:5173`
- AuroraView WebView window opens
- WebSocket server listening on `ws://localhost:9001`

### Step 4: Load UXP Plugin

1. Open **UXP Developer Tool**
2. Click **Add Plugin**
3. Select `uxp_plugin/manifest.json`
4. Click **Load**
5. In Photoshop: **Plugins → AuroraView (Minimal)**
6. Click **Connect to Python**

## 🎨 Usage Examples

### Apply Gaussian Blur

1. Open an image in Photoshop
2. In AuroraView UI, click **Get Image from Photoshop**
3. Adjust blur radius slider
4. Click **Apply Blur**
5. See real-time preview in WebView

### Enhance Contrast

1. Load image
2. Adjust contrast factor slider
3. Click **Enhance Contrast**
4. Preview updates instantly

### Edge Detection

1. Load image
2. Click **Detect Edges**
3. See Canny edge detection result

## 🔧 Development

### Add New Image Filter

**1. Add Python function in `image_processor.py`:**

```python
def my_custom_filter(self, image_data: str, param: float) -> Dict[str, Any]:
    img = self.base64_to_image(image_data)
    # Your processing logic here
    result = self.image_to_base64(processed_img)
    return {"status": "success", "preview": f"data:image/png;base64,{result}"}
```

**2. Register in `photoshop_tool.py`:**

```python
def apply_filter(params):
    if filter_type == 'my_custom_filter':
        result = self.processor.my_custom_filter(image_data, param)
    return result
```

**3. Add UI control in `App.tsx`:**

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

### Add Photoshop Command

**1. In `uxp_plugin/index.js`:**

```javascript
async function myPhotoshopCommand(params) {
    await app.batchPlay([...], {});
    sendMessage('command_result', { ... });
}
```

**2. Call from Python:**

```python
self.bridge.execute_photoshop_command('my_command', {'param': value})
```

## 🎯 Advantages Over Pure UXP

| Feature | Pure UXP | AuroraView Integration |
|---------|----------|----------------------|
| UI Framework | Limited HTML/CSS | Full React + TypeScript |
| Image Processing | JavaScript (slow) | Python + NumPy (fast) |
| AI/ML Support | ❌ | ✅ PyTorch/TensorFlow |
| Dev Experience | UXP reload | Vite HMR (instant) |
| Debugging | UXP DevTool | Chrome DevTools |
| Python Ecosystem | ❌ | ✅ Full access |

## 📚 Python Libraries Available

- **Pillow**: Image manipulation
- **OpenCV**: Computer vision
- **NumPy**: Numerical computing
- **scikit-image**: Scientific image processing
- **PyTorch/TensorFlow**: Deep learning (optional)

## 🔍 Troubleshooting

### WebView doesn't open

- Check Python backend is running
- Verify AuroraView is installed: `pip install auroraview`

### UXP plugin can't connect

- Ensure Python backend is running
- Check WebSocket server is on port 9001
- Verify network permissions in `manifest.json`

### Image processing fails

- Install required libraries: `pip install Pillow opencv-python numpy`
- Check Python console for errors

## 📖 Next Steps

- Add more image filters
- Integrate AI models (style transfer, super-resolution)
- Implement batch processing
- Add export functionality
- Create custom Photoshop actions

## 🔗 References

- [AuroraView Documentation](../../README.md)
- [Adobe UXP](https://developer.adobe.com/photoshop/uxp/)
- [Pillow Docs](https://pillow.readthedocs.io/)
- [OpenCV Python](https://docs.opencv.org/4.x/d6/d00/tutorial_py_root.html)

## 📄 License

Part of the AuroraView project.

