# Project Status

## [OK] Completed Features

### Core Features
- [OK] **Rust WebView Core** - High-performance WebView using Wry 0.47
- [OK] **Python Bindings** - PyO3 with abi3 support (Python 3.7+)
- [OK] **Bidirectional IPC** - JavaScript ↔ Python communication
- [OK] **Event System** - `@webview.on()` and `webview.emit()`
- [OK] **Developer Tools** - F12 to open DevTools in embedded mode
- [OK] **Resource Cleanup** - Automatic cleanup on Maya exit (no process leaks)

### DCC Integration
- [OK] **Maya Embedded Mode** - Child/Owner window modes
- [OK] **Thread-Safe Event Handling** - `executeDeferred` pattern
- [OK] **Event Processing Loop** - `scriptJob` integration
- [OK] **Window Lifecycle Management** - Proper show/hide/close

### Examples
- [OK] **Maya Outliner** - Scene hierarchy viewer with selection sync
- [OK] **AI Chat Integration** - Code execution from AI chat
- [OK] **Simple Panel** - Basic WebView panel template

## [CONFIG] Recent Fixes

### Developer Tools Support (2025-01-29)
**Problem**: DevTools couldn't be opened with F12 or right-click menu

**Root Cause**:
- Python API had `dev_tools` parameter but didn't pass it to Rust
- Embedded mode didn't call `with_devtools(true)`

**Solution**:
1. Modified `python/auroraview/webview.py` to pass `dev_tools` parameter
2. Modified `src/webview/aurora_view.rs` to accept and use the parameter
3. Modified `src/webview/embedded.rs` to enable devtools in WebView builder

**Result**: [OK] DevTools now accessible via F12 or right-click → Inspect

### Process Leak Fix (2025-01-29)
**Problem**: Maya exit left WebView processes running

**Root Cause**:
- Embedded mode created `event_loop` but never cleaned it up
- Window resources not properly destroyed

**Solution**:
Added `Drop` implementation to `WebViewInner` in `src/webview/webview_inner.rs`:
- Hides window before destruction
- Calls `DestroyWindow()` on Windows
- Drops event loop to release resources

**Result**: [OK] Clean exit, no process leaks

##  Known Issues

### Window Close Button (In Progress)
**Status**: Under investigation

**Symptoms**:
- Clicking window X button doesn't close the window
- Clicking custom "Close" button doesn't work

**Investigation**:
- [OK] DevTools enabled for debugging
- [OK] Debug scripts created
-  Waiting for JavaScript console logs to diagnose event flow

**Next Steps**:
1. Run debug script in Maya
2. Open DevTools (F12)
3. Click close button
4. Analyze event flow: JavaScript → Python → Rust

## [GOAL] Architecture

### IPC Architecture
```
JavaScript (CustomEvent)
    ↓ Event Bridge
Rust IpcHandler
    ↓ PyO3
Python Callbacks
```

**Key Components**:
- `src/ipc/handler.rs` - Event routing and callback management
- `src/ipc/message_queue.rs` - Thread-safe message queue
- Event Bridge Script - JavaScript event interception

**Performance**:
- Lock-free data structures (DashMap, crossbeam-channel)
- Batch message processing
- Lazy initialization

### Module Structure
```
src/
├── ipc/              # IPC system (separated from WebView)
│   ├── handler.rs    # Event callbacks
│   ├── message_queue.rs  # Message queue
│   ├── backend.rs    # Backend abstraction
│   ├── threaded.rs   # Threaded backend
│   └── process.rs    # Process backend
├── webview/          # WebView core
│   ├── aurora_view.rs    # Python API
│   ├── webview_inner.rs  # Core implementation
│   ├── embedded.rs   # DCC integration
│   ├── standalone.rs # Standalone mode
│   ├── config.rs     # Configuration
│   ├── event_loop.rs # Event loop
│   └── message_pump.rs   # Windows message pump
└── lib.rs            # PyO3 module
```

## [STATS] Code Quality

### Metrics
- [OK] **Clippy Warnings**: 0
- [OK] **Compilation**: Success
- [OK] **Build Time**: ~20-30s (release)
- [OK] **Python Compatibility**: 3.7+

### Testing
- [OK] Manual testing in Maya 2024
-  Automated tests (planned)
-  CI/CD pipeline (planned)

## [LAUNCH] Roadmap

### Short Term (Next 2 Weeks)
- [ ] Fix window close button issue
- [ ] Add automated tests
- [ ] Improve error messages
- [ ] Add more examples

### Medium Term (Next Month)
- [ ] Support for 3ds Max
- [ ] Support for Houdini
- [ ] Performance benchmarks
- [ ] Documentation improvements

### Long Term (Next Quarter)
- [ ] Cross-platform testing (macOS, Linux)
- [ ] Plugin marketplace integration
- [ ] Advanced UI components library
- [ ] Video tutorials

## [DOCS] Documentation

### Available Docs
- [OK] `README.md` - Project overview
- [OK] `DCC_INTEGRATION_GUIDE.md` - Integration guide
- [OK] `IPC_ARCHITECTURE.md` - IPC system design
- [OK] `TECHNICAL_DESIGN.md` - Technical details
- [OK] `examples/maya/README.md` - Maya examples guide
- [OK] `ROADMAP.md` - Future plans

### Removed Docs (Consolidated)
- [ERROR] Debug guides (issues fixed)
- [ERROR] IPC refactoring docs (consolidated into IPC_ARCHITECTURE.md)
- [ERROR] Temporary fix summaries (integrated into PROJECT_STATUS.md)

##  Code Cleanup

### Recent Cleanup (2025-01-29)
**Removed Test Files**:
- `examples/maya/test_close_debug.py`
- `examples/maya/simple_test.py`
- `examples/maya/minimal_test.py`
- `examples/maya/test_auto_refresh.py`
- `examples/maya/test_outliner_fixes.py`
- `examples/maya/test_outliner_simple.py`
- `examples/maya/test_scene_objects.py`

**Removed Docs**:
- `docs/ASYNC_DCC_INTEGRATION.md`
- `docs/CLOSE_BUTTON_DEBUG.md`
- `docs/DEBUG_GUIDE.md`
- `docs/QUICK_DEBUG_GUIDE.md`
- `docs/CURRENT_STATUS.md`
- `docs/MAYA_OUTLINER_FIXES.md`
- `docs/MAYA_TESTING_GUIDE.md`
- `docs/SUMMARY.md`
- 8 IPC-related docs (consolidated)

**Consolidated Docs**:
- IPC docs → `IPC_ARCHITECTURE.md`
- Maya fixes → `MAYA_FIXES_SUMMARY.md`
- Outliner implementation → `MAYA_OUTLINER_IMPLEMENTATION.md`

### Current File Count
**Examples**: 3 files
- `outliner_view.py` - Main example
- `ai_chat_integration.py` - AI integration
- `simple_panel.py` - Simple template

**Docs**: 10 files (down from 25+)
- Core documentation only
- No debug/temporary files
- Clear organization

## [SEARCH] Quick Start

### Installation
```bash
# Clone repository
git clone https://github.com/loonghao/auroraview.git
cd auroraview

# Build Rust library
cargo build --release

# Install Python package
pip install -e .
```

### Basic Usage
```python
from auroraview import WebView

# Create WebView
webview = WebView(
    title="My Tool",
    width=800,
    height=600,
    dev_tools=True  # Enable F12 DevTools
)

# Register event handler
@webview.on("my_event")
def handle_event(data):
    print(f"Received: {data}")

# Load HTML
webview.load_html("<h1>Hello World</h1>")

# Show window
webview.show()
```

### Maya Integration
```python
import maya.cmds as cmds
from auroraview import WebView

# Get Maya main window
from PySide2 import QtWidgets
maya_window = None
for widget in QtWidgets.QApplication.topLevelWidgets():
    if widget.objectName() == 'MayaWindow':
        maya_window = widget
        break

# Create embedded WebView
webview = WebView(
    title="Maya Tool",
    width=600,
    height=400,
    parent_hwnd=int(maya_window.winId())
)

# Event processing loop
def process_events():
    if webview._core.process_events():
        cmds.scriptJob(kill=timer_id)

timer_id = cmds.scriptJob(event=["idle", process_events])

# Keep reference
import __main__
__main__.my_webview = webview
```

##  Support

- **Issues**: [GitHub Issues](https://github.com/loonghao/auroraview/issues)
- **Discussions**: [GitHub Discussions](https://github.com/loonghao/auroraview/discussions)
- **Email**: hal.long@outlook.com

