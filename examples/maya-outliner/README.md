# Maya Outliner - AuroraView Example

[![Vue 3](https://img.shields.io/badge/Vue-3.5-4FC08D?logo=vue.js&logoColor=white)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.7-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Vite](https://img.shields.io/badge/Vite-6.0-646CFF?logo=vite&logoColor=white)](https://vitejs.dev/)
[![AuroraView](https://img.shields.io/badge/AuroraView-Rust-orange?logo=rust&logoColor=white)](../../README.md)

[中文文档](./README_zh.md) | [Quick Start](./QUICKSTART.md)

A modern, web-based Maya Outliner built with **AuroraView**, **Vue 3**, and **TypeScript**. This example demonstrates how to create high-performance DCC tools with modern web technologies embedded directly in Maya.

## ✨ Features

- 🌳 **Hierarchical Scene Tree** - Display Maya's scene hierarchy with expandable nodes
- 🎯 **Real-time Selection Sync** - Bidirectional selection synchronization between Maya and UI
- 👁️ **Visibility Toggle** - Show/hide objects directly from the outliner
- 🔍 **Search & Filter** - Quickly find nodes by name
- ⚡ **High Performance** - Handle 10,000+ nodes smoothly with AuroraView's optimized IPC
- 🎨 **Modern UI** - Clean, dark-themed interface built with Vue 3
- 🔄 **Live Updates** - Automatic UI updates when scene changes

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│           Maya (Python)                 │
│  ┌───────────────────────────────────┐  │
│  │   maya_outliner.py                │  │
│  │   - Scene hierarchy queries       │  │
│  │   - Selection management          │  │
│  │   - Visibility control            │  │
│  │   - Maya callbacks                │  │
│  └───────────┬───────────────────────┘  │
│              │ AuroraView IPC            │
│              │ (Thread-based, <1μs)      │
│  ┌───────────▼───────────────────────┐  │
│  │   WebView (Embedded)              │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │  Vue 3 Frontend             │  │  │
│  │  │  - OutlinerTree.vue         │  │  │
│  │  │  - TreeNode.vue             │  │  │
│  │  │  - useMayaIPC composable    │  │  │
│  │  └─────────────────────────────┘  │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## 📦 Installation

### Prerequisites

- **Maya 2020+** (with Python 3.7+)
- **Node.js 18+** and npm
- **AuroraView** installed in Maya's Python environment

### Install AuroraView

**Option A: Qt Backend (Recommended)**
```bash
# Install with Qt support for better Maya integration
mayapy -m pip install auroraview[qt]
```
- ✅ Non-blocking by default
- ✅ Better integration with Maya's UI
- ✅ Seamless window management

**Option B: Native Backend (Fallback)**
```bash
# Install without Qt dependencies
mayapy -m pip install auroraview
```
- ✅ Works without Qt
- ✅ Uses `show_async()` for non-blocking
- ⚠️ Standalone window

**Local Development:**
```bash
cd /path/to/dcc_webview
mayapy -m pip install -e python/[qt]  # With Qt
# or
mayapy -m pip install -e python/      # Without Qt
```

### Install Frontend Dependencies

```bash
cd examples/maya-outliner
npm install
```

## 🚀 Usage

### Development Workflow (Recommended for Contributors)

**One-time setup:**
```bash
just maya-setup-dev
```

This creates symlinks so your code changes are immediately available in Maya.

**Daily development:**
```bash
just maya-dev
```

This rebuilds Rust core and launches Maya. Click the "Outliner" button on the AuroraView shelf!

📖 See [QUICKSTART.md](./QUICKSTART.md) for a concise development guide

### Quick Debug Workflow (For Testing)

**Using Just command:**
```bash
just maya-debug
```

This will:
1. Kill all Maya processes
2. Rebuild Rust core
3. Set PYTHONPATH
4. Launch Maya 2024

**Using batch script (Recommended):**
```bash
cd examples/maya-outliner
launch_maya_debug.bat
```

This script will:
- Kill all Maya processes
- Rebuild Rust core
- Set PYTHONPATH
- Launch Maya with AuroraView available

**Verify PYTHONPATH in Maya:**
After Maya starts, run this in Script Editor:
```python
exec(open(r'C:\path\to\examples\maya-outliner\test_pythonpath.py').read())
```

You should see:
```
✅ SUCCESS: auroraview imported successfully!
✅ SUCCESS: WebView class imported!
```

**Auto-load on Maya startup:**
Copy `userSetup.py` to Maya's scripts folder:
```bash
# Windows
copy userSetup.py "C:\Users\<username>\Documents\maya\2024\scripts\"
```

Then restart Maya - you'll see an "Outliner" button on the AuroraView shelf!

### Development Mode

1. **Start the Vite dev server:**

```bash
npm run dev
```

This will start the development server at `http://localhost:5173` with hot-reload.

2. **Run in Maya:**

Open Maya's Script Editor and run:

```python
import sys
sys.path.append(r"C:\path\to\dcc_webview\examples\maya-outliner")

from maya_integration import maya_outliner
maya_outliner.main()
```

The outliner window will open and connect to the Vite dev server automatically.

### Production Mode

1. **Build the frontend:**

```bash
npm run build
```

2. **Serve the built files:**

```bash
npm run preview
```

3. **Run in Maya** (same as development mode)

## 🎯 Features Demonstration

## 🪟 Windows WebView2 Backend (Experimental)

AuroraView provides a Windows-native WebView2 backend designed for DCC hosts (Qt event loop). Build the core with the feature enabled:

```bash
# Build wheel with WebView2 (Windows only)
# Example with maturin (inside repo root)
set RUSTFLAGS=
set CARGO_BUILD_TARGET=
python -m pip uninstall -y auroraview || echo .
maturin develop --release --features win-webview2
```

Then inside Maya's Script Editor (Python):

```python
import sys
sys.path.append(r"C:\path\to\dcc_webview\examples\maya-outliner")
from maya_integration.launch_webview2 import launch
h = launch("http://localhost:5173", 1000, 700)
print("WebView2 handle:", h)
```

If you see an error about the feature not enabled, rebuild AuroraView with `--features win-webview2`.



### Complete Maya example (WebView2, embedded with IPC)

The example now includes a fully interactive WebView2 variant that talks to Maya using the same CustomEvent API as the Qt/native wrapper. Use it when you want a Windows-native, Qt-loop-friendly control without QtWebEngine.

Usage in Maya Script Editor (Python):

````python
import sys
sys.path.append(r"C:\path\to\dcc_webview\examples\maya-outliner")

from maya_integration.maya_outliner_webview2 import main
outliner = main(url="http://localhost:5173")
# Tip: outliner.close() to close programmatically
````

How it works:
- Injects a tiny JS bridge so your existing frontend (CustomEvent-based) works unchanged
- Page -> Maya: window.dispatchEvent(CustomEvent(name, {detail})) → chrome.webview.postMessage → Python
- Maya -> Page: Python posts {type:"emit", event, detail} → page dispatchEvent(CustomEvent(event, {detail}))

Supported events (already wired):
- get_scene_hierarchy → scene_updated
- select_node
- set_visibility → scene_updated

Note: This backend requires building AuroraView with `--features win-webview2`.

### Scene Hierarchy

The outliner displays your Maya scene as a hierarchical tree:

- **Transform nodes** 📁
- **Mesh nodes** 🔷
- **Camera nodes** 📷
- **Light nodes** 💡
- **Joint nodes** 🦴
- **Locator nodes** 📍

### Selection Synchronization

- **Click a node** in the outliner → Maya selects it
- **Select in Maya** → Outliner highlights it
- **Real-time updates** with <1ms latency

### Visibility Control

- Click the 👁️ icon to toggle visibility
- Changes reflect immediately in Maya viewport
- Supports hierarchical visibility

### Search & Filter

- Type in the search box to filter nodes
- Matches node names (case-insensitive)
- Shows matching nodes and their parents

## 📊 Performance Benchmarks

Tested on Windows 10, Intel i7-9700K, 32GB RAM:

| Nodes | Load Time | Selection Latency | Memory Usage |
|-------|-----------|-------------------|--------------|
| 100   | <10ms     | <1ms              | ~15MB        |
| 1,000 | ~50ms     | <1ms              | ~25MB        |
| 10,000| ~300ms    | <2ms              | ~80MB        |

**Why so fast?**

- **Thread-based IPC** instead of HTTP/WebSocket
- **Crossbeam channels** for lock-free communication
- **Message batching** (16ms window, ~60 FPS)
- **Efficient Vue 3 rendering** with virtual DOM
- **EventTimer** for automatic event processing at 60 FPS

## 🔄 Event Processing

This example uses **EventTimer** for automatic event processing, replacing the traditional `scriptJob` + `QTimer` approach:

```python
from auroraview import WebView, EventTimer

# Create WebView
webview = WebView(parent_hwnd=maya_hwnd, embedded=True)
webview.show()

# Create EventTimer - automatic event processing!
timer = EventTimer(webview, interval_ms=16)  # 60 FPS

@timer.on_close
def handle_close():
    print("Window closed")
    timer.stop()

timer.start()
```

**Benefits:**
- ✅ **50% less code** compared to scriptJob + QTimer
- ✅ **60 FPS refresh rate** (vs 30 FPS with QTimer)
- ✅ **Multi-strategy close detection** (window messages + validity check)
- ✅ **Automatic resource cleanup**
- ✅ **Cross-platform compatible**

EventTimer is now built-in to AuroraView examples; no separate migration document is required.

## 🛠️ Development

### Project Structure

```
maya-outliner/
├── src/
│   ├── components/
│   │   ├── OutlinerTree.vue    # Main tree component
│   │   └── TreeNode.vue         # Individual node component
│   ├── composables/
│   │   └── useMayaIPC.ts        # IPC communication layer
│   ├── types.ts                 # TypeScript type definitions
│   ├── App.vue                  # Root component
│   ├── main.ts                  # Entry point
│   └── style.css                # Global styles
├── maya/
│   ├── maya_outliner.py         # Maya backend
│   └── __init__.py
├── package.json
├── vite.config.ts
├── tsconfig.json
└── README.md
```

### Adding New Features

**1. Add a new IPC event:**

Frontend (`useMayaIPC.ts`):
```typescript
sendToMaya('my_event', { data: 'value' })
```

Backend (`maya_outliner.py`):
```python
@self.webview.on("my_event")
def handle_my_event(data):
    print(f"Received: {data}")
```

**2. Add a new UI component:**

Create `src/components/MyComponent.vue` and import it in `App.vue`.

**3. Add new node properties:**

Update `MayaNode` interface in `src/types.ts` and modify `get_scene_hierarchy()` in `maya_outliner.py`.

## 🐛 Troubleshooting

### Common Issues

**Blank WebView / Cannot read data:**
- Ensure the dev server is running (npm run dev) or use a packaged build (npm run build && npm run preview)

**Module not found:**
```bash
npm install  # Install frontend dependencies
mayapy -m pip install auroraview  # Install AuroraView
```

**Vite server not running:**
```bash
npm run dev  # Start development server
```

## 📚 Learn More

- [AuroraView Documentation](../../README.md)
- [Vue 3 Documentation](https://vuejs.org/)
- [Vite Documentation](https://vitejs.dev/)
- [Maya Python API](https://help.autodesk.com/view/MAYAUL/2024/ENU/?guid=Maya_SDK_py_ref_index_html)

## 📄 License

This example is part of the AuroraView project and follows the same license.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

---

**Built with ❤️ using AuroraView**

