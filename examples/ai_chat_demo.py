"""AI Chat Assistant Demo - Qt Window with DeepSeek WebView Integration.

This example demonstrates a hybrid application combining:
- Qt-based main window for tool/parameter controls
- WebView panel for AI chat interface (DeepSeek)
- Two-way communication between Qt and WebView

Features demonstrated:
- Qt + WebView hybrid layout
- API key configuration via environment
- Streaming chat responses
- Bidirectional Python ↔ WebView communication

Requirements:
    - PySide6>=6.5.0
    - openai>=1.0.0

Use cases:
- AI-assisted DCC tools
- Interactive parameter editors with AI suggestions
- Smart content generation pipelines

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import importlib
import os
import sys
from pathlib import Path
from threading import Event, Thread

# Add project root to path so demos can reuse Gallery utilities (dependency installer, etc.)
PROJECT_ROOT = Path(__file__).resolve().parents[1]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

# Check for Qt framework
try:
    from PySide6.QtCore import Qt, Signal, Slot, QThread
    from PySide6.QtWidgets import (
        QApplication,
        QMainWindow,
        QWidget,
        QVBoxLayout,
        QHBoxLayout,
        QSplitter,
        QLabel,
        QLineEdit,
        QPushButton,
        QTextEdit,
        QGroupBox,
        QFormLayout,
        QSpinBox,
        QDoubleSpinBox,
        QComboBox,
        QMessageBox,
    )
    from PySide6.QtGui import QFont, QPalette, QColor

    HAS_QT = True
except ImportError:
    HAS_QT = False
    print("PySide6 is required for this demo.")
    print("Please run: pip install PySide6>=6.5.0")

# OpenAI client (DeepSeek compatible)
# NOTE: do NOT hard-fail at import time; we support auto-install via docstring requirements.
OpenAI = None


# Chat HTML template for WebView
CHAT_HTML = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AI Chat Assistant</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e4e4e4;
            height: 100vh;
            display: flex;
            flex-direction: column;
        }
        
        .header {
            padding: 16px 20px;
            background: rgba(0, 0, 0, 0.2);
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
            display: flex;
            align-items: center;
            gap: 12px;
        }
        
        .header .icon {
            width: 32px;
            height: 32px;
            background: linear-gradient(135deg, #00d4ff 0%, #0099ff 100%);
            border-radius: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 18px;
        }
        
        .header .title {
            font-size: 16px;
            font-weight: 600;
            color: #00d4ff;
        }
        
        .header .status {
            margin-left: auto;
            font-size: 12px;
            color: #888;
        }
        
        .header .status.connected { color: #00cc66; }
        .header .status.error { color: #ff6b6b; }
        
        .messages {
            flex: 1;
            overflow-y: auto;
            padding: 20px;
            display: flex;
            flex-direction: column;
            gap: 16px;
        }
        
        .message {
            max-width: 85%;
            padding: 12px 16px;
            border-radius: 12px;
            line-height: 1.5;
            animation: fadeIn 0.3s ease;
        }
        
        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(10px); }
            to { opacity: 1; transform: translateY(0); }
        }
        
        .message.user {
            align-self: flex-end;
            background: linear-gradient(135deg, #0066cc 0%, #0099ff 100%);
            color: white;
        }
        
        .message.assistant {
            align-self: flex-start;
            background: rgba(255, 255, 255, 0.08);
            border: 1px solid rgba(255, 255, 255, 0.1);
        }
        
        .message.system {
            align-self: center;
            background: rgba(255, 193, 7, 0.15);
            color: #ffc107;
            font-size: 13px;
            border: 1px solid rgba(255, 193, 7, 0.3);
        }
        
        .message.error {
            align-self: center;
            background: rgba(255, 107, 107, 0.15);
            color: #ff6b6b;
            font-size: 13px;
            border: 1px solid rgba(255, 107, 107, 0.3);
        }
        
        .message pre {
            background: rgba(0, 0, 0, 0.3);
            padding: 12px;
            border-radius: 8px;
            overflow-x: auto;
            margin: 8px 0;
            font-family: 'Monaco', 'Menlo', monospace;
            font-size: 13px;
        }
        
        .typing {
            display: flex;
            gap: 4px;
            padding: 8px 12px;
        }
        
        .typing span {
            width: 8px;
            height: 8px;
            background: #00d4ff;
            border-radius: 50%;
            animation: typing 1.4s infinite ease-in-out;
        }
        
        .typing span:nth-child(2) { animation-delay: 0.2s; }
        .typing span:nth-child(3) { animation-delay: 0.4s; }
        
        @keyframes typing {
            0%, 80%, 100% { transform: scale(0.8); opacity: 0.5; }
            40% { transform: scale(1); opacity: 1; }
        }
        
        .input-area {
            padding: 16px 20px;
            background: rgba(0, 0, 0, 0.2);
            border-top: 1px solid rgba(255, 255, 255, 0.1);
            display: flex;
            gap: 12px;
        }
        
        .input-area input {
            flex: 1;
            padding: 12px 16px;
            border: 1px solid rgba(255, 255, 255, 0.2);
            border-radius: 24px;
            background: rgba(255, 255, 255, 0.05);
            color: #e4e4e4;
            font-size: 14px;
            outline: none;
            transition: all 0.3s;
        }
        
        .input-area input:focus {
            border-color: #00d4ff;
            background: rgba(255, 255, 255, 0.08);
        }
        
        .input-area input::placeholder { color: #666; }
        
        .input-area button {
            width: 44px;
            height: 44px;
            border: none;
            border-radius: 22px;
            background: linear-gradient(135deg, #00d4ff 0%, #0099ff 100%);
            color: white;
            font-size: 18px;
            cursor: pointer;
            transition: all 0.3s;
        }
        
        .input-area button:hover { transform: scale(1.05); }
        .input-area button:disabled { opacity: 0.5; cursor: not-allowed; }
    </style>
</head>
<body>
    <div class="header">
        <div class="icon">🤖</div>
        <span class="title">DeepSeek Assistant</span>
        <span class="status" id="status">Connecting...</span>
    </div>
    
    <div class="messages" id="messages">
        <div class="message system">
            Welcome! I'm your AI assistant powered by DeepSeek.
            Configure your API key in the left panel to start chatting.
        </div>
    </div>
    
    <div class="input-area">
        <input type="text" id="input" placeholder="Type your message..." 
               onkeypress="if(event.key==='Enter')sendMessage()">
        <button onclick="sendMessage()" id="sendBtn">→</button>
    </div>
    
    <script>
        const messagesEl = document.getElementById('messages');
        const inputEl = document.getElementById('input');
        const statusEl = document.getElementById('status');
        const sendBtn = document.getElementById('sendBtn');
        
        let isProcessing = false;
        
        // Initialize AuroraView connection
        if (window.auroraview) {
            window.auroraview.whenReady().then(() => {
                statusEl.textContent = 'Connected';
                statusEl.className = 'status connected';
            });
            
            // Listen for responses from Python
            let streamingEl = null;
            let streamingText = '';

            function formatMessageHtml(content) {
                // Handle code blocks
                content = content.replace(/```(\w*)\n([\s\S]*?)```/g,
                    '<pre><code>$2</code></pre>');
                content = content.replace(/`([^`]+)`/g, '<code>$1</code>');
                content = content.replace(/\n/g, '<br>');
                return content;
            }

            function ensureStreamingMessage() {
                if (streamingEl) return;
                streamingEl = document.createElement('div');
                streamingEl.className = 'message assistant';
                streamingEl.id = 'streaming';
                streamingEl.innerHTML = '';
                messagesEl.appendChild(streamingEl);
                messagesEl.scrollTop = messagesEl.scrollHeight;
                streamingText = '';
            }

            window.auroraview.on('chat:delta', (data) => {
                const delta = (data && data.delta) ? data.delta : '';
                if (!delta) return;
                removeTypingIndicator();
                ensureStreamingMessage();
                streamingText += delta;
                streamingEl.innerHTML = formatMessageHtml(streamingText);
                messagesEl.scrollTop = messagesEl.scrollHeight;
            });

            window.auroraview.on('chat:response', (data) => {
                removeTypingIndicator();
                const content = (data && data.content) ? data.content : '';
                if (streamingEl) {
                    streamingEl.innerHTML = formatMessageHtml(content);
                    streamingEl.removeAttribute('id');
                    streamingEl = null;
                    streamingText = '';
                } else {
                    addMessage(content, 'assistant');
                }
                isProcessing = false;
                updateUI();
            });
            
            window.auroraview.on('chat:error', (data) => {
                removeTypingIndicator();
                if (streamingEl) {
                    streamingEl.remove();
                    streamingEl = null;
                    streamingText = '';
                }
                addMessage(data.error, 'error');
                isProcessing = false;
                updateUI();
            });
            
            window.auroraview.on('chat:status', (data) => {
                statusEl.textContent = data.status;
                statusEl.className = 'status ' + (data.ok ? 'connected' : 'error');
            });
        }
        
        function addMessage(content, type) {
            const div = document.createElement('div');
            div.className = 'message ' + type;
            
            // Handle code blocks
            content = content.replace(/```(\\w*)\\n([\\s\\S]*?)```/g, 
                '<pre><code>$2</code></pre>');
            content = content.replace(/`([^`]+)`/g, '<code>$1</code>');
            content = content.replace(/\\n/g, '<br>');
            
            div.innerHTML = content;
            messagesEl.appendChild(div);
            messagesEl.scrollTop = messagesEl.scrollHeight;
            return div;
        }
        
        function addTypingIndicator() {
            const div = document.createElement('div');
            div.className = 'message assistant typing';
            div.id = 'typing';
            div.innerHTML = '<span></span><span></span><span></span>';
            messagesEl.appendChild(div);
            messagesEl.scrollTop = messagesEl.scrollHeight;
        }
        
        function removeTypingIndicator() {
            const typing = document.getElementById('typing');
            if (typing) typing.remove();
        }
        
        function updateUI() {
            inputEl.disabled = isProcessing;
            sendBtn.disabled = isProcessing;
        }
        
        function sendMessage() {
            const message = inputEl.value.trim();
            if (!message || isProcessing) return;
            
            addMessage(message, 'user');
            inputEl.value = '';
            isProcessing = true;
            updateUI();
            addTypingIndicator();
            
            // Send to Python backend - chat.send is fire-and-forget for now
            if (window.auroraview) {
                // Note: We don't await because chat.send is designed as fire-and-forget
                // The Python side will emit events back (chat:delta, chat:response, chat:error)
                window.auroraview.call('chat.send', { message }).catch(err => {
                    console.error('Failed to send message to Python:', err);
                    addMessage('Error: Failed to communicate with backend', 'error');
                    isProcessing = false;
                    updateUI();
                });
            }
        }
    </script>
</body>
</html>
"""


class ChatWorker(QThread):
    """Worker thread for handling DeepSeek API calls."""

    delta_ready = Signal(str)
    response_ready = Signal(str)
    error_occurred = Signal(str)

    def __init__(
        self,
        api_key: str,
        base_url: str,
        model: str,
        message: str,
        *,
        temperature: float = 0.7,
        max_tokens: int = 2000,
    ):
        super().__init__()
        self.api_key = api_key
        self.base_url = base_url
        self.model = model
        self.message = message
        self.temperature = temperature
        self.max_tokens = max_tokens
        self.messages_history: list[dict] = []

    def set_history(self, history: list[dict]):
        """Set conversation history."""
        self.messages_history = history.copy()

    def run(self):
        """Execute the API call in background thread."""
        try:
            if OpenAI is None:
                raise RuntimeError("OpenAI client not available. Please install 'openai>=1.0.0'.")

            client = OpenAI(api_key=self.api_key, base_url=self.base_url)

            messages = self.messages_history + [{"role": "user", "content": self.message}]

            stream = client.chat.completions.create(
                model=self.model,
                messages=messages,
                stream=True,
                temperature=self.temperature,
                max_tokens=self.max_tokens,
            )

            chunks = []
            for event in stream:
                # openai>=1 returns ChatCompletionChunk events
                try:
                    delta = event.choices[0].delta.content
                except Exception:
                    delta = None

                if delta:
                    chunks.append(delta)
                    self.delta_ready.emit(delta)

            content = "".join(chunks).strip()
            self.response_ready.emit(content)
        except Exception as e:
            self.error_occurred.emit(str(e))


class AIChatWindow(QMainWindow):
    """Main window with Qt controls and WebView chat panel."""

    def __init__(self):
        super().__init__()
        self.setWindowTitle("AI Chat Assistant - Qt + WebView Demo")
        self.setMinimumSize(1200, 700)

        # Chat state
        self.messages_history: list[dict] = []
        self.worker: ChatWorker | None = None
        self.webview = None

        # Dependency state
        self._missing_requirements: list[str] = []
        self._install_cancel_event: Optional[Event] = None
        self._install_thread: Optional[Thread] = None

        self._setup_ui()
        self._setup_webview()
        self._apply_dark_theme()

        # Soft-check dependencies (do not exit; allow installing from UI)
        self._refresh_dependency_status()

    def _setup_ui(self):
        """Setup the main UI layout."""
        central = QWidget()
        self.setCentralWidget(central)

        # Main horizontal splitter
        splitter = QSplitter(Qt.Horizontal)

        # Left panel - Qt controls
        left_panel = self._create_left_panel()
        splitter.addWidget(left_panel)

        # Right panel - WebView placeholder (will be replaced)
        self.webview_container = QWidget()
        self.webview_layout = QVBoxLayout(self.webview_container)
        self.webview_layout.setContentsMargins(0, 0, 0, 0)

        placeholder = QLabel("Loading AI Chat...")
        placeholder.setAlignment(Qt.AlignCenter)
        placeholder.setStyleSheet("color: #666; font-size: 16px;")
        self.webview_layout.addWidget(placeholder)

        splitter.addWidget(self.webview_container)
        splitter.setSizes([350, 850])

        layout = QHBoxLayout(central)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.addWidget(splitter)

    def _create_left_panel(self) -> QWidget:
        """Create the left control panel."""
        panel = QWidget()
        panel.setMaximumWidth(400)
        layout = QVBoxLayout(panel)
        layout.setSpacing(16)

        # API Configuration group
        api_group = QGroupBox("DeepSeek API Configuration")
        api_layout = QFormLayout(api_group)

        self.api_key_input = QLineEdit()
        self.api_key_input.setEchoMode(QLineEdit.Password)
        self.api_key_input.setPlaceholderText("sk-...")
        self.api_key_input.setText(os.environ.get("DEEPSEEK_API_KEY", ""))
        api_layout.addRow("API Key:", self.api_key_input)

        self.base_url_input = QLineEdit()
        self.base_url_input.setText("https://api.deepseek.com/v1")
        api_layout.addRow("Base URL:", self.base_url_input)

        self.model_combo = QComboBox()
        self.model_combo.addItems(["deepseek-chat", "deepseek-coder", "deepseek-reasoner"])
        api_layout.addRow("Model:", self.model_combo)

        layout.addWidget(api_group)

        # Parameters group
        params_group = QGroupBox("Generation Parameters")
        params_layout = QFormLayout(params_group)

        self.temp_spin = QDoubleSpinBox()
        self.temp_spin.setRange(0.0, 2.0)
        self.temp_spin.setValue(0.7)
        self.temp_spin.setSingleStep(0.1)
        params_layout.addRow("Temperature:", self.temp_spin)

        self.max_tokens_spin = QSpinBox()
        self.max_tokens_spin.setRange(100, 8000)
        self.max_tokens_spin.setValue(2000)
        self.max_tokens_spin.setSingleStep(100)
        params_layout.addRow("Max Tokens:", self.max_tokens_spin)

        layout.addWidget(params_group)

        # Actions group
        actions_group = QGroupBox("Actions")
        actions_layout = QVBoxLayout(actions_group)

        self.connect_btn = QPushButton("🔌 Test Connection")
        self.connect_btn.clicked.connect(self._test_connection)
        actions_layout.addWidget(self.connect_btn)

        self.install_deps_btn = QPushButton("⬇️ Install Missing Dependencies")
        self.install_deps_btn.clicked.connect(self._install_missing_dependencies)
        actions_layout.addWidget(self.install_deps_btn)

        self.clear_btn = QPushButton("🗑️ Clear Chat History")
        self.clear_btn.clicked.connect(self._clear_history)
        actions_layout.addWidget(self.clear_btn)

        self.deps_log = QTextEdit()
        self.deps_log.setReadOnly(True)
        self.deps_log.setPlaceholderText("Dependency install logs will appear here...")
        self.deps_log.setMaximumHeight(160)
        actions_layout.addWidget(self.deps_log)

        layout.addWidget(actions_group)

        # Status
        self.status_label = QLabel("Status: Not connected")
        self.status_label.setStyleSheet("color: #888; padding: 8px;")
        layout.addWidget(self.status_label)

        layout.addStretch()
        return panel

    def _setup_webview(self):
        """Setup the AuroraView WebView panel."""
        try:
            from auroraview import AuroraView

            # Create WebView
            self.webview = AuroraView(
                html=CHAT_HTML,
                title="AI Chat",
                width=850,
                height=700,
                parent=self.webview_container,
                embed_mode="child",
            )

            # Bind chat API
            @self.webview.bind_call("chat.send")
            def handle_chat_send(message: str = ""):
                self._handle_user_message(message)

            # Replace placeholder with WebView
            for i in reversed(range(self.webview_layout.count())):
                w = self.webview_layout.itemAt(i).widget()
                if w:
                    w.setParent(None)

            self.webview_layout.addWidget(self.webview)
            self.webview.show()

            # Update status
            self._update_status("WebView loaded", ok=True)

        except ImportError as e:
            self._update_status(f"AuroraView not available: {e}", ok=False)

    def _apply_dark_theme(self):
        """Apply dark theme to Qt widgets."""
        palette = QPalette()
        palette.setColor(QPalette.Window, QColor(26, 26, 46))
        palette.setColor(QPalette.WindowText, QColor(228, 228, 228))
        palette.setColor(QPalette.Base, QColor(22, 33, 62))
        palette.setColor(QPalette.AlternateBase, QColor(26, 26, 46))
        palette.setColor(QPalette.ToolTipBase, QColor(228, 228, 228))
        palette.setColor(QPalette.ToolTipText, QColor(228, 228, 228))
        palette.setColor(QPalette.Text, QColor(228, 228, 228))
        palette.setColor(QPalette.Button, QColor(22, 33, 62))
        palette.setColor(QPalette.ButtonText, QColor(228, 228, 228))
        palette.setColor(QPalette.BrightText, QColor(0, 212, 255))
        palette.setColor(QPalette.Link, QColor(0, 153, 255))
        palette.setColor(QPalette.Highlight, QColor(0, 153, 255))
        palette.setColor(QPalette.HighlightedText, QColor(255, 255, 255))
        self.setPalette(palette)

    def _update_status(self, message: str, ok: bool = True):
        """Update status label and WebView status."""
        color = "#00cc66" if ok else "#ff6b6b"
        self.status_label.setText(f"Status: {message}")
        self.status_label.setStyleSheet(f"color: {color}; padding: 8px;")

        if self.webview:
            self.webview.emit("chat:status", {"status": message, "ok": ok})

    def _handle_user_message(self, message: str):
        """Handle incoming user message from WebView."""
        api_key = self.api_key_input.text().strip()
        if not api_key:
            if self.webview:
                self.webview.emit("chat:error", {"error": "Please configure API key first"})
            return

        # Refresh dependency status before checking
        self._refresh_dependency_status()
        
        # Check if installation is in progress
        if hasattr(self, '_install_thread') and self._install_thread and self._install_thread.is_alive():
            if self.webview:
                self.webview.emit(
                    "chat:error",
                    {
                        "error": "Dependency installation is in progress. Please wait for it to complete.",
                    },
                )
            self._update_status("Installation in progress...", ok=False)
            return
        
        if OpenAI is None:
            missing_deps = ", ".join(self._missing_requirements) if self._missing_requirements else "openai>=1.0.0"
            if self.webview:
                self.webview.emit(
                    "chat:error",
                    {
                        "error": f"Missing dependency: {missing_deps}. Please click 'Install Missing Dependencies' button on the left panel.",
                    },
                )
            self._update_status(f"Missing dependency: {missing_deps}", ok=False)
            
            # Highlight the install button
            self.install_deps_btn.setStyleSheet("background-color: #ff6b6b; color: white;")
            # Reset style after 3 seconds
            from PySide6.QtCore import QTimer
            QTimer.singleShot(3000, lambda: self.install_deps_btn.setStyleSheet(""))
            return

        # Create worker thread for API call
        self.worker = ChatWorker(
            api_key=api_key,
            base_url=self.base_url_input.text().strip(),
            model=self.model_combo.currentText(),
            message=message,
            temperature=float(self.temp_spin.value()),
            max_tokens=int(self.max_tokens_spin.value()),
        )
        self.worker.set_history(self.messages_history)
        self.worker.delta_ready.connect(self._on_delta)
        self.worker.response_ready.connect(self._on_response)
        self.worker.error_occurred.connect(self._on_error)
        self.worker.start()

        # Add to history
        self.messages_history.append({"role": "user", "content": message})

    @Slot(str)
    def _on_delta(self, delta: str):
        """Handle streaming delta chunks."""
        if self.webview:
            self.webview.emit("chat:delta", {"delta": delta})

    @Slot(str)
    def _on_response(self, content: str):
        """Handle API response."""
        self.messages_history.append({"role": "assistant", "content": content})
        if self.webview:
            self.webview.emit("chat:response", {"content": content})

    @Slot(str)
    def _on_error(self, error: str):
        """Handle API error."""
        if self.webview:
            self.webview.emit("chat:error", {"error": error})
        self._update_status(f"Error: {error}", ok=False)

    def _test_connection(self):
        """Test API connection."""
        api_key = self.api_key_input.text().strip()
        if not api_key:
            QMessageBox.warning(self, "Error", "Please enter an API key")
            return

        if OpenAI is None:
            QMessageBox.warning(
                self,
                "Missing dependency",
                "The OpenAI client is not installed. Click 'Install Missing Dependencies' first.",
            )
            return

        self._update_status("Testing connection...")
        try:
            client = OpenAI(
                api_key=api_key,
                base_url=self.base_url_input.text().strip(),
            )
            # Simple test call
            _ = client.chat.completions.create(
                model=self.model_combo.currentText(),
                messages=[{"role": "user", "content": "Hi"}],
                max_tokens=10,
                temperature=float(self.temp_spin.value()),
                stream=False,
            )
            self._update_status("Connection successful!", ok=True)
            QMessageBox.information(self, "Success", "API connection successful!")
        except Exception as e:
            self._update_status(f"Connection failed: {e}", ok=False)
            QMessageBox.critical(self, "Error", f"Connection failed:\n{e}")

    def _refresh_dependency_status(self) -> None:
        """Refresh missing dependency list from the demo docstring Requirements."""
        try:
            from gallery.backend.dependency_installer import (
                get_missing_requirements,
                parse_requirements_from_docstring,
            )
        except Exception as e:
            # If gallery isn't available, just leave as-is.
            print(f"Warning: Could not check dependencies: {e}")
            return

        docstring = __doc__ or ""
        reqs = parse_requirements_from_docstring(docstring)
        missing = get_missing_requirements(reqs)
        self._missing_requirements = missing

        # Update button state based on missing requirements
        if missing:
            self.install_deps_btn.setEnabled(True)
            self.install_deps_btn.setText("⬇️ Install Missing Dependencies")
            if len(missing) == 1:
                self._update_status(f"Missing dependency: {missing[0]}", ok=False)
            else:
                self._update_status(f"Missing dependencies: {len(missing)} packages", ok=False)
        else:
            self.install_deps_btn.setEnabled(False)
            self.install_deps_btn.setText("✓ Dependencies Ready")
            self._update_status("All dependencies satisfied", ok=True)

    def _install_missing_dependencies(self):
        """Install missing dependencies defined in the demo docstring asynchronously."""
        try:
            from gallery.backend.dependency_installer import install_requirements
            from threading import Event
        except Exception as exc:
            QMessageBox.critical(self, "Error", f"Dependency installer not available: {exc}")
            return

        # Always refresh before installing
        self._refresh_dependency_status()
        if not self._missing_requirements:
            QMessageBox.information(self, "OK", "No missing dependencies.")
            return

        self.install_deps_btn.setEnabled(False)
        self.install_deps_btn.setText("⏳ Installing...")
        self.deps_log.clear()
        self._update_status("Installing dependencies...", ok=True)

        # Create cancel event for the installation
        self._install_cancel_event = Event()

        def on_progress(p: dict):
            # Ensure UI updates happen in main thread
            from PySide6.QtCore import QMetaObject, Qt, Q_ARG
            
            msg = p.get("message") or p.get("line") or str(p)
            # Use signal-slot mechanism to update UI safely
            QMetaObject.invokeMethod(
                self,
                "_append_dep_log",
                Qt.ConnectionType.QueuedConnection,
                Q_ARG(str, msg)
            )

        def install_worker():
            """Run installation in background thread to avoid blocking UI."""
            try:
                result = install_requirements(
                    self._missing_requirements, 
                    on_progress=on_progress,
                    cancel_event=self._install_cancel_event
                )
                
                # Schedule UI update in main thread
                QMetaObject.invokeMethod(
                    self,
                    "_finalize_installation",
                    Qt.ConnectionType.QueuedConnection,
                    Q_ARG(dict, result)
                )
            except Exception as e:
                # Handle unexpected errors
                error_msg = f"Installation error: {str(e)}"
                QMetaObject.invokeMethod(
                    self,
                    "_append_dep_log",
                    Qt.ConnectionType.QueuedConnection,
                    Q_ARG(str, f"\n❌ {error_msg}")
                )
                QMetaObject.invokeMethod(
                    self,
                    "_finalize_installation",
                    Qt.ConnectionType.QueuedConnection,
                    Q_ARG(dict, {"success": False, "output": error_msg, "cancelled": False})
                )

        # Start installation in background thread
        from threading import Thread
        self._install_thread = Thread(target=install_worker, daemon=True)
        self._install_thread.start()

    @Slot(dict)
    def _finalize_installation(self, result: dict):
        """Finalize installation process in main thread."""
        # Finalize installation
        self.install_deps_btn.setText("⬇️ Install Missing Dependencies")
        
        if result.get("success"):
            self.deps_log.append("\n✅ All dependencies installed successfully.")
            # Import openai after install
            try:
                global OpenAI
                OpenAI = importlib.import_module("openai").OpenAI
            except Exception as e:
                self.deps_log.append(f"\n⚠️ Warning: Could not import openai after install: {e}")

            self._refresh_dependency_status()
            QMessageBox.information(self, "Success", "Dependencies installed successfully!")
            self._update_status("Dependencies ready", ok=True)
        else:
            err = result.get("output") or "Installation failed"
            self.deps_log.append("\n❌ " + err)
            if result.get("cancelled"):
                self.deps_log.append("\n⏹️ Installation was cancelled.")
                self._update_status("Installation cancelled", ok=False)
            else:
                self._update_status("Installation failed", ok=False)
            
            # Re-enable button for retry
            self.install_deps_btn.setEnabled(True)
            QMessageBox.critical(self, "Error", "Some dependencies failed to install. See log below.")
        
        # Clean up
        self._install_cancel_event = None
        self._install_thread = None

    @Slot(str)
    def _append_dep_log(self, msg: str):
        """Append message to dependency log (thread-safe UI update)."""
        self.deps_log.append(msg)
        # Auto-scroll to bottom
        cursor = self.deps_log.textCursor()
        cursor.movePosition(cursor.End)
        self.deps_log.setTextCursor(cursor)

    def _clear_history(self):
        """Clear chat history."""
        self.messages_history.clear()
        if self.webview:
            self.webview.eval_js("""
                document.getElementById('messages').innerHTML =
                    '<div class="message system">Chat history cleared.</div>';
            """)
        self._update_status("History cleared", ok=True)


def main():
    """Run the AI Chat Demo."""
    if not HAS_QT:
        print("\n" + "=" * 60)
        print("ERROR: PySide6 is required for this demo")
        print("Please install it with: pip install PySide6>=6.5.0")
        print("=" * 60)
        sys.exit(1)


    print("\n" + "=" * 60)
    print("AI Chat Assistant Demo")
    print("=" * 60)
    print("\nThis demo shows Qt + WebView hybrid application with:")
    print("  - Left panel: Qt controls for API configuration")
    print("  - Right panel: WebView chat interface (DeepSeek)")
    print("\nTo use the chat:")
    print("  1. Enter your DeepSeek API key in the left panel")
    print("  2. Click 'Test Connection' to verify")
    print("  3. Type messages in the chat panel")
    print("\nEnvironment variable: Set DEEPSEEK_API_KEY to pre-fill the key")
    print("=" * 60 + "\n")

    app = QApplication(sys.argv)
    app.setStyle("Fusion")

    window = AIChatWindow()

    # Try importing OpenAI after UI created; if missing, user can install via UI.
    try:
        global OpenAI
        OpenAI = importlib.import_module("openai").OpenAI
    except Exception:
        OpenAI = None

    window.show()

    sys.exit(app.exec())


if __name__ == "__main__":
    main()
