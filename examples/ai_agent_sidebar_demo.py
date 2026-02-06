"""AI Agent Sidebar Demo - Showcase AI-Powered Tool Control.

This example demonstrates the AI Agent sidebar feature in AuroraView:
- Collapsible sidebar with chat interface
- Multi-model support (OpenAI, Claude, DeepSeek, Ollama)
- Automatic tool discovery from bind_call/bind_api
- AG-UI protocol for streaming responses
- DCC-style tool integration patterns

Features demonstrated:
- AI Agent sidebar integration
- Auto-discovery of Python APIs as AI tools
- Streaming chat responses with AG-UI protocol
- Tool execution and result display
- Model selection and configuration

Requirements:
    - PySide6>=6.5.0

Use cases:
- AI-assisted DCC workflows
- Natural language tool control
- Intelligent parameter suggestions
- Automated scene operations

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import sys

# Check for Qt framework
try:
    from PySide6.QtCore import Qt
    from PySide6.QtWidgets import (
        QApplication,
        QMainWindow,
        QWidget,
        QVBoxLayout,
        QHBoxLayout,
        QSplitter,
        QLabel,
        QTextEdit,
        QGroupBox,
    )
    from PySide6.QtGui import QPalette, QColor

    HAS_QT = True
except ImportError:
    HAS_QT = False
    print("PySide6 is required for this demo.")


# Demo tool functions that will be discovered by AI Agent
DEMO_TOOLS_CONTEXT = {
    "selected_objects": ["Cube1", "Sphere1", "Cylinder1"],
    "scene_name": "demo_scene.ma",
    "render_settings": {"width": 1920, "height": 1080, "format": "exr"},
}


def get_selection() -> list[str]:
    """Get currently selected objects in the scene.

    Returns:
        List of selected object names.
    """
    return DEMO_TOOLS_CONTEXT["selected_objects"]


def set_selection(objects: list[str]) -> dict:
    """Set the current selection to specified objects.

    Args:
        objects: List of object names to select.

    Returns:
        Success status and selected objects.
    """
    DEMO_TOOLS_CONTEXT["selected_objects"] = objects
    return {"success": True, "selected": objects}


def get_scene_info() -> dict:
    """Get information about the current scene.

    Returns:
        Scene information including name and object count.
    """
    return {
        "name": DEMO_TOOLS_CONTEXT["scene_name"],
        "objects": len(DEMO_TOOLS_CONTEXT["selected_objects"]),
        "render_settings": DEMO_TOOLS_CONTEXT["render_settings"],
    }


def export_scene(format: str = "fbx", path: str = "") -> dict:
    """Export the current scene to specified format.

    Args:
        format: Export format (fbx, obj, abc, usd).
        path: Output file path. Auto-generated if empty.

    Returns:
        Success status and export path.
    """
    if not path:
        path = f"/tmp/export.{format}"
    return {"success": True, "path": path, "format": format}


def create_object(obj_type: str = "cube", name: str = "") -> dict:
    """Create a new object in the scene.

    Args:
        obj_type: Type of object (cube, sphere, cylinder, plane).
        name: Object name. Auto-generated if empty.

    Returns:
        Created object information.
    """
    if not name:
        name = f"{obj_type.capitalize()}_{len(DEMO_TOOLS_CONTEXT['selected_objects']) + 1}"
    DEMO_TOOLS_CONTEXT["selected_objects"].append(name)
    return {"success": True, "name": name, "type": obj_type}


def render_preview(width: int = 1920, height: int = 1080) -> dict:
    """Render a preview of the current view.

    Args:
        width: Render width in pixels.
        height: Render height in pixels.

    Returns:
        Render status and output path.
    """
    return {
        "success": True,
        "width": width,
        "height": height,
        "path": "/tmp/preview.exr",
    }


# Gallery HTML with embedded AI sidebar demo
DEMO_HTML = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AI Agent Sidebar Demo</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: linear-gradient(135deg, #0f0f1a 0%, #1a1a2e 100%);
            color: #e4e4e4;
            height: 100vh;
            display: flex;
        }

        .main-content {
            flex: 1;
            padding: 32px;
            overflow-y: auto;
        }

        h1 {
            font-size: 28px;
            margin-bottom: 8px;
            background: linear-gradient(135deg, #00d4ff 0%, #a855f7 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }

        .subtitle {
            color: #888;
            margin-bottom: 32px;
        }

        .feature-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 20px;
            margin-bottom: 32px;
        }

        .feature-card {
            background: rgba(255, 255, 255, 0.05);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 12px;
            padding: 20px;
            transition: all 0.3s;
        }

        .feature-card:hover {
            background: rgba(255, 255, 255, 0.08);
            border-color: rgba(168, 85, 247, 0.3);
            transform: translateY(-2px);
        }

        .feature-card h3 {
            font-size: 16px;
            margin-bottom: 8px;
            color: #a855f7;
        }

        .feature-card p {
            font-size: 14px;
            color: #888;
            line-height: 1.5;
        }

        .tools-section {
            background: rgba(0, 0, 0, 0.3);
            border-radius: 12px;
            padding: 24px;
            margin-bottom: 24px;
        }

        .tools-section h2 {
            font-size: 18px;
            margin-bottom: 16px;
            color: #00d4ff;
        }

        .tool-list {
            display: flex;
            flex-wrap: wrap;
            gap: 12px;
        }

        .tool-badge {
            padding: 8px 16px;
            background: rgba(168, 85, 247, 0.2);
            border: 1px solid rgba(168, 85, 247, 0.3);
            border-radius: 20px;
            font-size: 13px;
            color: #a855f7;
            cursor: pointer;
            transition: all 0.2s;
        }

        .tool-badge:hover {
            background: rgba(168, 85, 247, 0.3);
        }

        .ai-toggle {
            position: fixed;
            bottom: 24px;
            right: 24px;
            width: 56px;
            height: 56px;
            border-radius: 28px;
            background: linear-gradient(135deg, #a855f7 0%, #00d4ff 100%);
            border: none;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 24px;
            box-shadow: 0 4px 20px rgba(168, 85, 247, 0.4);
            transition: all 0.3s;
        }

        .ai-toggle:hover {
            transform: scale(1.1);
        }

        .hint {
            position: fixed;
            bottom: 90px;
            right: 24px;
            background: rgba(0, 0, 0, 0.8);
            padding: 8px 16px;
            border-radius: 8px;
            font-size: 13px;
            color: #888;
        }
    </style>
</head>
<body>
    <div class="main-content">
        <h1>🤖 AI Agent Sidebar Demo</h1>
        <p class="subtitle">Natural language control for your tools and workflows</p>

        <div class="feature-grid">
            <div class="feature-card">
                <h3>🔍 Auto Tool Discovery</h3>
                <p>AI automatically discovers Python APIs bound via bind_call() and exposes them as callable tools.</p>
            </div>
            <div class="feature-card">
                <h3>💬 Streaming Responses</h3>
                <p>Real-time streaming via AG-UI protocol for responsive chat experience.</p>
            </div>
            <div class="feature-card">
                <h3>🎯 Multi-Provider</h3>
                <p>Supports OpenAI, Claude, Gemini, DeepSeek, Groq, and local Ollama models.</p>
            </div>
            <div class="feature-card">
                <h3>🎨 DCC Ready</h3>
                <p>Designed for Maya, Houdini, Blender, and other DCC applications.</p>
            </div>
        </div>

        <div class="tools-section">
            <h2>📦 Discovered Tools</h2>
            <div class="tool-list" id="tools">
                <span class="tool-badge" data-tool="selection.get">selection.get</span>
                <span class="tool-badge" data-tool="selection.set">selection.set</span>
                <span class="tool-badge" data-tool="scene.info">scene.info</span>
                <span class="tool-badge" data-tool="scene.export">scene.export</span>
                <span class="tool-badge" data-tool="object.create">object.create</span>
                <span class="tool-badge" data-tool="render.preview">render.preview</span>
            </div>
        </div>

        <div class="tools-section">
            <h2>💡 Try These Prompts</h2>
            <div class="tool-list">
                <span class="tool-badge">"What objects are selected?"</span>
                <span class="tool-badge">"Create a new sphere"</span>
                <span class="tool-badge">"Export scene as FBX"</span>
                <span class="tool-badge">"Render a preview at 4K"</span>
            </div>
        </div>
    </div>

    <button class="ai-toggle" onclick="toggleAI()" title="Toggle AI Assistant (Ctrl+Shift+A)">
        🤖
    </button>
    <div class="hint">Press Ctrl+Shift+A to toggle AI sidebar</div>

    <script>
        // Initialize tool discovery
        if (window.auroraview) {
            window.auroraview.whenReady().then(async () => {
                try {
                    const tools = await window.auroraview.call('ai.get_tools');
                    console.log('Discovered tools:', tools);
                } catch (e) {
                    console.log('AI tools not available:', e);
                }
            });
        }

        function toggleAI() {
            // Emit keyboard shortcut to toggle sidebar
            const event = new KeyboardEvent('keydown', {
                key: 'A',
                ctrlKey: true,
                shiftKey: true,
            });
            window.dispatchEvent(event);
        }

        // Tool badge clicks
        document.querySelectorAll('.tool-badge[data-tool]').forEach(badge => {
            badge.addEventListener('click', async () => {
                const tool = badge.dataset.tool;
                if (window.auroraview) {
                    try {
                        const result = await window.auroraview.call(tool);
                        console.log(`${tool} result:`, result);
                        alert(`${tool} returned:\\n${JSON.stringify(result, null, 2)}`);
                    } catch (e) {
                        console.error(`${tool} error:`, e);
                    }
                }
            });
        });
    </script>
</body>
</html>
"""


class AIAgentDemoWindow(QMainWindow):
    """Demo window showcasing AI Agent sidebar capabilities."""

    def __init__(self):
        super().__init__()
        self.setWindowTitle("AI Agent Sidebar Demo")
        self.setMinimumSize(1200, 700)
        self.webview = None
        self._setup_ui()
        self._setup_webview()
        self._apply_dark_theme()

    def _setup_ui(self):
        """Setup the main UI layout."""
        central = QWidget()
        self.setCentralWidget(central)

        splitter = QSplitter(Qt.Horizontal)

        # Left panel - Info
        left_panel = self._create_info_panel()
        splitter.addWidget(left_panel)

        # Right panel - WebView
        self.webview_container = QWidget()
        self.webview_layout = QVBoxLayout(self.webview_container)
        self.webview_layout.setContentsMargins(0, 0, 0, 0)

        placeholder = QLabel("Loading AI Agent Demo...")
        placeholder.setAlignment(Qt.AlignCenter)
        placeholder.setStyleSheet("color: #666; font-size: 16px;")
        self.webview_layout.addWidget(placeholder)

        splitter.addWidget(self.webview_container)
        splitter.setSizes([300, 900])

        layout = QHBoxLayout(central)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.addWidget(splitter)

    def _create_info_panel(self) -> QWidget:
        """Create the info panel."""
        panel = QWidget()
        panel.setMaximumWidth(350)
        layout = QVBoxLayout(panel)
        layout.setSpacing(16)

        # Info group
        info_group = QGroupBox("AI Agent Features")
        info_layout = QVBoxLayout(info_group)

        info_text = QTextEdit()
        info_text.setReadOnly(True)
        info_text.setHtml("""
            <h3 style="color: #a855f7;">🤖 AI Agent Sidebar</h3>
            <p>This demo showcases the AI-powered sidebar that can:</p>
            <ul>
                <li><b>Auto-discover</b> Python APIs as AI tools</li>
                <li><b>Stream</b> responses via AG-UI protocol</li>
                <li><b>Execute</b> tools and display results</li>
                <li><b>Support</b> multiple AI providers</li>
            </ul>
            <h4 style="color: #00d4ff;">Try it:</h4>
            <p>Press <b>Ctrl+Shift+A</b> to toggle the AI sidebar, or click the 🤖 button.</p>
        """)
        info_layout.addWidget(info_text)
        layout.addWidget(info_group)

        # API Keys info
        api_group = QGroupBox("API Configuration")
        api_layout = QVBoxLayout(api_group)

        api_info = QLabel(
            "Set environment variables:\n"
            "• OPENAI_API_KEY\n"
            "• ANTHROPIC_API_KEY\n"
            "• DEEPSEEK_API_KEY\n"
            "• GEMINI_API_KEY\n\n"
            "Or use local Ollama models."
        )
        api_info.setWordWrap(True)
        api_info.setStyleSheet("color: #888; padding: 8px;")
        api_layout.addWidget(api_info)
        layout.addWidget(api_group)

        layout.addStretch()
        return panel

    def _setup_webview(self):
        """Setup the AuroraView WebView."""
        try:
            from auroraview import AuroraView

            self.webview = AuroraView(
                html=DEMO_HTML,
                title="AI Agent Demo",
                width=900,
                height=700,
                parent=self.webview_container,
                embed_mode="child",
            )

            # Bind demo tools
            self.webview.bind_call("selection.get")(get_selection)
            self.webview.bind_call("selection.set")(set_selection)
            self.webview.bind_call("scene.info")(get_scene_info)
            self.webview.bind_call("scene.export")(export_scene)
            self.webview.bind_call("object.create")(create_object)
            self.webview.bind_call("render.preview")(render_preview)

            # Replace placeholder
            for i in reversed(range(self.webview_layout.count())):
                w = self.webview_layout.itemAt(i).widget()
                if w:
                    w.setParent(None)

            self.webview_layout.addWidget(self.webview)
            self.webview.show()

        except ImportError as e:
            print(f"AuroraView not available: {e}")

    def _apply_dark_theme(self):
        """Apply dark theme."""
        palette = QPalette()
        palette.setColor(QPalette.Window, QColor(15, 15, 26))
        palette.setColor(QPalette.WindowText, QColor(228, 228, 228))
        palette.setColor(QPalette.Base, QColor(26, 26, 46))
        palette.setColor(QPalette.AlternateBase, QColor(15, 15, 26))
        palette.setColor(QPalette.Text, QColor(228, 228, 228))
        palette.setColor(QPalette.Button, QColor(26, 26, 46))
        palette.setColor(QPalette.ButtonText, QColor(228, 228, 228))
        palette.setColor(QPalette.Highlight, QColor(168, 85, 247))
        palette.setColor(QPalette.HighlightedText, QColor(255, 255, 255))
        self.setPalette(palette)


def main():
    """Run the AI Agent Sidebar Demo."""
    if not HAS_QT:
        print("\n" + "=" * 60)
        print("ERROR: PySide6 is required for this demo")
        print("Please install it with: pip install PySide6>=6.5.0")
        print("=" * 60)
        sys.exit(1)

    print("\n" + "=" * 60)
    print("AI Agent Sidebar Demo")
    print("=" * 60)
    print("\nThis demo showcases AI Agent sidebar capabilities:")
    print("  - Auto-discovery of Python APIs as AI tools")
    print("  - Streaming chat responses with AG-UI protocol")
    print("  - Multi-provider support (OpenAI, Claude, DeepSeek, Ollama)")
    print("\nPress Ctrl+Shift+A to toggle the AI sidebar")
    print("=" * 60 + "\n")

    app = QApplication(sys.argv)
    app.setStyle("Fusion")

    window = AIAgentDemoWindow()
    window.show()

    sys.exit(app.exec())


if __name__ == "__main__":
    main()
