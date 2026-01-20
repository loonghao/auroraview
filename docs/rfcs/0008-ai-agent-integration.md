# RFC 0008: AI Agent Integration with AGUI/A2UI Protocol

## Summary

Add AI Agent capabilities to AuroraView, enabling natural language control of the browser and WebView. Support multiple AI providers (OpenAI, Anthropic Claude, Google Gemini, Azure OpenAI, local models) with extensible Python bindings. Implement AGUI (Agent-User Interaction Protocol) and A2UI (Agent to UI) protocols for standardized AI-frontend communication.

## Motivation

### Current Limitations

1. **No AI Integration**: AuroraView lacks native AI capabilities for intelligent browsing
2. **Manual Control Only**: Users must manually navigate, search, and interact
3. **No Standard Protocol**: No unified way for AI agents to communicate with UI
4. **Limited Extensibility**: Users cannot easily add custom AI providers

### Goals

- **Multi-Provider Support**: OpenAI, Claude, Gemini, Azure, local models (Ollama, LM Studio)
- **AGUI/A2UI Compliance**: Follow emerging AI-UI interaction standards
- **Python Extensibility**: Allow users to implement custom providers
- **Seamless Integration**: AI controls real WebView, not simulated content
- **DCC Compatible**: Works in Maya, Houdini, Blender, and standalone

### Use Cases

1. **Natural Language Navigation**: "Open GitHub and search for AuroraView"
2. **Content Summarization**: "Summarize this page"
3. **Intelligent Actions**: "Bookmark all tabs related to Python"
4. **Code Generation**: "Create a login form with Tailwind CSS"
5. **DCC Workflow**: "Find Maya scripting documentation"
6. **Auto-call Application APIs**: AI automatically discovers and calls Python APIs bound to WebView

## Design

### 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      AI Agent Integration Architecture                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Provider Layer (Rust + Python)                    │   │
│  │                                                                     │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌──────────────┐  │   │
│  │  │   OpenAI    │ │  Anthropic  │ │   Gemini    │ │ Azure OpenAI │  │   │
│  │  │ GPT-4/4o    │ │  Claude 3   │ │  2.0 Flash  │ │    GPT-4     │  │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └──────────────┘  │   │
│  │                                                                     │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────────┐   │   │
│  │  │   Ollama    │ │  LM Studio  │ │   Custom Python Provider    │   │   │
│  │  │ Local LLMs  │ │ Local LLMs  │ │   (User Extensible)         │   │   │
│  │  └─────────────┘ └─────────────┘ └─────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Protocol Layer (AGUI + A2UI)                      │   │
│  │                                                                     │   │
│  │  ┌──────────────────────────┐  ┌──────────────────────────────┐    │   │
│  │  │   AGUI Events (16+)      │  │   A2UI UI Components         │    │   │
│  │  │ - TEXT_MESSAGE_START     │  │ - Declarative UI specs       │    │   │
│  │  │ - TEXT_MESSAGE_CONTENT   │  │ - Component catalog          │    │   │
│  │  │ - TEXT_MESSAGE_END       │  │ - Safe rendering             │    │   │
│  │  │ - TOOL_CALL_START        │  │ - SSE + JSON-RPC             │    │   │
│  │  │ - TOOL_CALL_END          │  │                              │    │   │
│  │  │ - STATE_SYNC             │  │                              │    │   │
│  │  │ - ...                    │  │                              │    │   │
│  │  └──────────────────────────┘  └──────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Action Layer (Browser Control)                    │   │
│  │                                                                     │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐          │   │
│  │  │ navigate  │ │  search   │ │ bookmark  │ │  execute  │          │   │
│  │  │  (url)    │ │  (query)  │ │  (add)    │ │   (js)    │          │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘          │   │
│  │                                                                     │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐          │   │
│  │  │  tab_*    │ │ summarize │ │ generate  │ │  custom   │          │   │
│  │  │(new/close)│ │  (page)   │ │   (ui)    │ │  (action) │          │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Presentation Layer (Frontend)                     │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │           Modern Chat UI (Gemini Browser Inspired)            │   │   │
│  │  │                                                               │   │   │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │   │   │
│  │  │  │  Chat Panel │  │  History    │  │  Discover/Actions   │  │   │   │
│  │  │  │  - Messages │  │  - Sessions │  │  - Capabilities     │  │   │   │
│  │  │  │  - Streaming│  │  - Search   │  │  - Quick Actions    │  │   │   │
│  │  │  │  - UI Cards │  │             │  │                     │  │   │   │
│  │  │  └─────────────┘  └─────────────┘  └─────────────────────┘  │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. Sidebar/Drawer Mode Architecture

AI Agent supports two presentation modes:

1. **Standalone Browser Mode** - AI as a complete browser interface
2. **Sidebar/Drawer Mode** - AI as a collapsible sidebar attached to any WebView

```
┌───────────────────────────────────────────────────────────────────────────┐
│                          WebView + AI Sidebar                              │
├───────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌─────────────────────────────────────────────┐  ┌───────────────────┐  │
│  │                                             │  │   AI Agent Sidebar │  │
│  │                                             │  │   ┌─────────────┐ │  │
│  │              Main WebView Content           │  │   │ Chat Messages│ │  │
│  │                                             │  │   │             │ │  │
│  │     (Your app, tool panels, DCC plugins)    │  │   │  User: ...  │ │  │
│  │                                             │  │   │  AI: ...    │ │  │
│  │                                             │  │   │             │ │  │
│  │                                             │  │   └─────────────┘ │  │
│  │                                             │  │   ┌─────────────┐ │  │
│  │                                             │  │   │  Input Box   │ │  │
│  │                                             │  │   └─────────────┘ │  │
│  └─────────────────────────────────────────────┘  └───────────────────┘  │
│                                                        ↑                  │
│                                                 Collapsible/Resizable     │
└───────────────────────────────────────────────────────────────────────────┘
```

**Sidebar Features**:
- Collapsible/expandable (like VSCode sidebar)
- Resizable width
- Draggable to left/right side
- Keyboard shortcut support (Ctrl+Shift+A to toggle)
- Shares context with main WebView

### 3. Auto-Discovery of WebView APIs

AI Agent **automatically discovers and uses** all API methods bound via `bind_call()`:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     API Auto-Discovery Flow                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐                                                       │
│  │  WebView     │                                                       │
│  │              │    bind_call("api.echo", echo_func)                   │
│  │  bind_call() ├──────────────────────────────────────────┐            │
│  │  bind_api()  │    bind_api(my_api_object)               │            │
│  └──────────────┘                                          │            │
│                                                            ↓            │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    API Registry (Python)                          │  │
│  │                                                                  │  │
│  │  _bound_functions = {                                            │  │
│  │      "api.echo": <function echo>,                                │  │
│  │      "api.export_scene": <function export_scene>,                │  │
│  │      "api.create_cube": <function create_cube>,                  │  │
│  │      "browser.navigate": <function navigate>,                    │  │
│  │      ...                                                         │  │
│  │  }                                                               │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                          │                              │
│                                          │ get_bound_methods()          │
│                                          ↓                              │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    AI Agent Tool Registration                      │  │
│  │                                                                  │  │
│  │  Auto-converted to AI tool definitions:                           │  │
│  │  {                                                               │  │
│  │    "name": "api.echo",                                           │  │
│  │    "description": "Echo a message back",  // from docstring      │  │
│  │    "parameters": {                        // inferred from types │  │
│  │      "type": "object",                                           │  │
│  │      "properties": {"message": {"type": "string"}}               │  │
│  │    }                                                             │  │
│  │  }                                                               │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4. Crate Structure

```
crates/
├── auroraview-ai-agent/           # NEW: AI Agent Integration
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                 # Public API
│       ├── agent.rs               # AIAgent struct
│       ├── session.rs             # ChatSession management
│       ├── message.rs             # Message types
│       │
│       ├── providers/             # AI Provider implementations
│       │   ├── mod.rs
│       │   ├── provider.rs        # Provider trait
│       │   ├── openai.rs          # OpenAI (GPT-4, GPT-4o)
│       │   ├── anthropic.rs       # Anthropic (Claude 3)
│       │   ├── gemini.rs          # Google (Gemini 2.0)
│       │   ├── azure.rs           # Azure OpenAI
│       │   ├── ollama.rs          # Ollama (local models)
│       │   └── custom.rs          # Custom provider bridge
│       │
│       ├── protocol/              # AGUI/A2UI Protocol
│       │   ├── mod.rs
│       │   ├── agui.rs            # AGUI events
│       │   ├── a2ui.rs            # A2UI components
│       │   └── streaming.rs       # SSE streaming
│       │
│       ├── actions/               # Browser Actions (Function Calling)
│       │   ├── mod.rs
│       │   ├── navigation.rs      # navigate, go_back, go_forward
│       │   ├── tabs.rs            # new_tab, close_tab, switch_tab
│       │   ├── content.rs         # summarize, search, bookmark
│       │   ├── ui.rs              # generate_ui, show_toast
│       │   └── registry.rs        # Action registry
│       │
│       └── ui/                    # UI Components
│           ├── mod.rs
│           ├── components.rs      # Predefined UI components
│           └── renderer.rs        # Component renderer
```

### 5. Provider Trait (Rust)

```rust
// crates/auroraview-ai-agent/src/providers/provider.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// AI Provider trait - implement this for new providers
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Provider name (e.g., "openai", "anthropic", "gemini")
    fn name(&self) -> &str;
    
    /// Send message and get response (non-streaming)
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError>;
    
    /// Send message and stream response
    async fn complete_stream(
        &self,
        request: CompletionRequest,
        on_event: Box<dyn Fn(StreamEvent) + Send>,
    ) -> Result<(), AIError>;
    
    /// Check if provider supports function calling
    fn supports_function_calling(&self) -> bool;
    
    /// Check if provider supports vision (image input)
    fn supports_vision(&self) -> bool;
    
    /// Get available models
    fn available_models(&self) -> Vec<ModelInfo>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: MessageContent,
    pub name: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String, // JSON string
}

/// Stream events for real-time UI updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "text_start")]
    TextStart { id: String },
    #[serde(rename = "text_delta")]
    TextDelta { id: String, delta: String },
    #[serde(rename = "text_end")]
    TextEnd { id: String },
    #[serde(rename = "tool_call_start")]
    ToolCallStart { id: String, name: String },
    #[serde(rename = "tool_call_delta")]
    ToolCallDelta { id: String, delta: String },
    #[serde(rename = "tool_call_end")]
    ToolCallEnd { id: String },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "done")]
    Done,
}
```

### 6. AGUI Protocol Events

```rust
// crates/auroraview-ai-agent/src/protocol/agui.rs

use serde::{Deserialize, Serialize};

/// AGUI Event Types (following AG-UI Protocol standard)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AGUIEvent {
    // === Text Message Events ===
    #[serde(rename = "TEXT_MESSAGE_START")]
    TextMessageStart {
        message_id: String,
        role: String,
    },
    
    #[serde(rename = "TEXT_MESSAGE_CONTENT")]
    TextMessageContent {
        message_id: String,
        delta: String,
    },
    
    #[serde(rename = "TEXT_MESSAGE_END")]
    TextMessageEnd {
        message_id: String,
    },
    
    // === Tool Call Events ===
    #[serde(rename = "TOOL_CALL_START")]
    ToolCallStart {
        message_id: String,
        tool_call_id: String,
        tool_name: String,
    },
    
    #[serde(rename = "TOOL_CALL_ARGS")]
    ToolCallArgs {
        tool_call_id: String,
        delta: String,
    },
    
    #[serde(rename = "TOOL_CALL_END")]
    ToolCallEnd {
        tool_call_id: String,
    },
    
    // === State Sync Events ===
    #[serde(rename = "STATE_SNAPSHOT")]
    StateSnapshot {
        state: serde_json::Value,
    },
    
    #[serde(rename = "STATE_DELTA")]
    StateDelta {
        path: String,
        operation: StateOperation,
        value: serde_json::Value,
    },
    
    // === UI Generation Events (A2UI) ===
    #[serde(rename = "UI_COMPONENT")]
    UIComponent {
        id: String,
        component: UIComponentSpec,
    },
    
    // === Lifecycle Events ===
    #[serde(rename = "RUN_START")]
    RunStart {
        run_id: String,
        thread_id: Option<String>,
    },
    
    #[serde(rename = "RUN_END")]
    RunEnd {
        run_id: String,
    },
    
    #[serde(rename = "RUN_ERROR")]
    RunError {
        run_id: String,
        error: ErrorInfo,
    },
    
    // === Human-in-the-Loop Events ===
    #[serde(rename = "INTERRUPT_REQUEST")]
    InterruptRequest {
        id: String,
        prompt: String,
        options: Vec<InterruptOption>,
    },
    
    #[serde(rename = "INTERRUPT_RESPONSE")]
    InterruptResponse {
        id: String,
        choice: String,
        data: Option<serde_json::Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StateOperation {
    Set,
    Delete,
    Append,
    Insert,
}

/// A2UI Component Specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIComponentSpec {
    #[serde(rename = "type")]
    pub component_type: String,
    pub props: serde_json::Value,
    pub children: Option<Vec<UIComponentSpec>>,
}
```

### 7. Browser Actions (Function Calling)

```rust
// crates/auroraview-ai-agent/src/actions/mod.rs

use serde::{Deserialize, Serialize};

/// Action registry for function calling
pub struct ActionRegistry {
    actions: HashMap<String, Box<dyn Action>>,
}

/// Action trait - implement for new browser actions
pub trait Action: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    fn execute(&self, args: serde_json::Value, ctx: &ActionContext) -> Result<ActionResult, ActionError>;
}

/// Built-in browser actions
pub fn register_builtin_actions(registry: &mut ActionRegistry) {
    // Navigation
    registry.register(NavigateAction);
    registry.register(GoBackAction);
    registry.register(GoForwardAction);
    registry.register(ReloadAction);
    registry.register(SearchAction);
    
    // Tabs
    registry.register(NewTabAction);
    registry.register(CloseTabAction);
    registry.register(SwitchTabAction);
    registry.register(ListTabsAction);
    
    // Content
    registry.register(SummarizePageAction);
    registry.register(ExtractTextAction);
    registry.register(TakeScreenshotAction);
    
    // Bookmarks
    registry.register(AddBookmarkAction);
    registry.register(ListBookmarksAction);
    registry.register(SearchBookmarksAction);
    
    // UI
    registry.register(ShowToastAction);
    registry.register(GenerateUIAction);
    registry.register(SetThemeAction);
    
    // Extensions
    registry.register(ToggleExtensionAction);
    registry.register(ListExtensionsAction);
}

// Example action implementation
pub struct NavigateAction;

impl Action for NavigateAction {
    fn name(&self) -> &str { "navigate" }
    
    fn description(&self) -> &str {
        "Navigate to a URL. Supports both direct URLs and search queries."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to navigate to, or search query"
                },
                "new_tab": {
                    "type": "boolean",
                    "description": "Open in new tab (default: false)"
                }
            },
            "required": ["url"]
        })
    }
    
    fn execute(&self, args: serde_json::Value, ctx: &ActionContext) -> Result<ActionResult, ActionError> {
        let url = args["url"].as_str().ok_or(ActionError::MissingParam("url"))?;
        let new_tab = args["new_tab"].as_bool().unwrap_or(false);
        
        // Determine if URL or search query
        let final_url = if url.contains('.') && !url.contains(' ') {
            if url.starts_with("http") { url.to_string() } 
            else { format!("https://{}", url) }
        } else {
            format!("https://www.google.com/search?q={}", urlencoding::encode(url))
        };
        
        if new_tab {
            ctx.browser.new_tab(&final_url)?;
        } else {
            ctx.browser.navigate(&final_url)?;
        }
        
        Ok(ActionResult::success(json!({
            "navigated_to": final_url,
            "new_tab": new_tab
        })))
    }
}
```

### 8. Python Bindings

```python
# python/auroraview/ai/agent.py

from typing import Optional, Callable, Any, Dict, List, Union
from dataclasses import dataclass, field
from enum import Enum
from abc import ABC, abstractmethod

class AIProviderType(Enum):
    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    GEMINI = "gemini"
    AZURE = "azure"
    OLLAMA = "ollama"
    CUSTOM = "custom"

@dataclass
class AIConfig:
    """AI Agent configuration."""
    provider: AIProviderType = AIProviderType.OPENAI
    model: str = "gpt-4o"
    api_key: Optional[str] = None
    base_url: Optional[str] = None
    temperature: float = 0.7
    max_tokens: int = 4096
    system_prompt: Optional[str] = None
    
class AIProvider(ABC):
    """Base class for custom AI providers.
    
    Users can extend this to add support for custom AI backends.
    """
    
    @abstractmethod
    def name(self) -> str:
        """Provider name."""
        pass
    
    @abstractmethod
    async def complete(
        self,
        messages: List[Dict],
        tools: Optional[List[Dict]] = None,
        **kwargs
    ) -> Dict:
        """Send completion request."""
        pass
    
    @abstractmethod
    async def complete_stream(
        self,
        messages: List[Dict],
        tools: Optional[List[Dict]] = None,
        on_event: Optional[Callable[[Dict], None]] = None,
        **kwargs
    ):
        """Send streaming completion request."""
        pass

class AIAgent:
    """AI Agent for natural language control of browser and WebView.
    
    Features:
    - Auto-discovers all API methods bound to WebView
    - Supports sidebar/drawer mode attached to any WebView
    - Multi-provider AI support
    
    Example - Standalone Browser Mode:
        from auroraview import Browser
        from auroraview.ai import AIAgent, AIConfig
        
        browser = Browser(title="AI Browser")
        agent = AIAgent(browser=browser, config=AIConfig(provider="gemini"))
        
        await agent.chat("Open GitHub and search for Python")
    
    Example - Sidebar Mode (attached to existing WebView):
        from auroraview import WebView
        from auroraview.ai import AIAgent
        
        # Create your application WebView
        webview = WebView(title="My App")
        
        # Bind some API methods
        @webview.bind_call("api.export_scene")
        def export_scene(format: str = "fbx"):
            '''Export current scene'''
            return {"status": "exported"}
        
        # Create AI Agent as sidebar
        agent = AIAgent.as_sidebar(webview, config=AIConfig(provider="gemini"))
        
        # AI can now automatically use export_scene method
        webview.show()
    """
    
    def __init__(
        self,
        browser: "Browser" = None,
        webview: "WebView" = None,
        config: Optional["AIConfig"] = None,
        provider: Optional[AIProvider] = None,
        auto_discover_apis: bool = True,
    ):
        self.browser = browser
        self.webview = webview or (browser._webview if browser else None)
        self.config = config or AIConfig()
        self._provider = provider
        self._session = ChatSession()
        self._actions = ActionRegistry()
        self._init_builtin_actions()
        
        # Auto-discover APIs bound to WebView
        if auto_discover_apis and self.webview:
            self._discover_webview_apis()
    
    @classmethod
    def as_sidebar(
        cls,
        webview: "WebView",
        config: Optional["AIConfig"] = None,
        **kwargs
    ) -> "AIAgent":
        """Create an AI Agent as a sidebar attached to a WebView.
        
        Args:
            webview: The WebView instance to attach sidebar to
            config: AI configuration
            **kwargs: Additional configuration parameters
        
        Returns:
            AIAgent instance configured in sidebar mode
        
        Example:
            webview = WebView(title="My Tool")
            agent = AIAgent.as_sidebar(webview, config=AIConfig(provider="gemini"))
        """
        agent = cls(webview=webview, config=config, **kwargs)
        agent._enable_sidebar_mode()
        return agent
    
    def _enable_sidebar_mode(self):
        """Enable sidebar mode, inject sidebar UI into WebView."""
        # Inject sidebar CSS and JS
        self.webview.eval_js(self._get_sidebar_injection_script())
        
        # Bind sidebar control methods
        @self.webview.bind_call("ai.toggle_sidebar")
        def toggle_sidebar():
            self.webview.eval_js("window.__auroraview_ai_sidebar.toggle()")
            return {"toggled": True}
        
        @self.webview.bind_call("ai.chat")
        def sidebar_chat(message: str):
            return self.chat_sync(message)
    
    def _discover_webview_apis(self):
        """Auto-discover and register all API methods bound to WebView."""
        if not hasattr(self.webview, 'get_bound_methods'):
            return
        
        bound_methods = self.webview.get_bound_methods()
        
        for method_name in bound_methods:
            # Skip internal methods
            if method_name.startswith('_') or method_name.startswith('ai.'):
                continue
            
            # Get function reference to extract docstring and type hints
            func = self.webview._bound_functions.get(method_name)
            if func is None:
                continue
            
            # Get description from docstring
            description = func.__doc__ or f"Call {method_name}"
            
            # Infer parameter schema from type annotations (if available)
            parameters = self._infer_parameters_schema(func)
            
            # Register as AI tool
            self._actions.register_from_webview(
                name=method_name,
                description=description,
                parameters=parameters,
                webview=self.webview,
            )
    
    def _infer_parameters_schema(self, func: Callable) -> Dict:
        """Infer JSON Schema parameter definition from function signature."""
        import inspect
        sig = inspect.signature(func)
        
        properties = {}
        required = []
        
        for name, param in sig.parameters.items():
            if name in ('self', 'cls'):
                continue
            
            prop = {"type": "string"}  # Default type
            
            # Try to infer type from annotation
            if param.annotation != inspect.Parameter.empty:
                python_type = param.annotation
                if python_type == int:
                    prop["type"] = "integer"
                elif python_type == float:
                    prop["type"] = "number"
                elif python_type == bool:
                    prop["type"] = "boolean"
                elif python_type == str:
                    prop["type"] = "string"
                elif python_type == list:
                    prop["type"] = "array"
                elif python_type == dict:
                    prop["type"] = "object"
            
            properties[name] = prop
            
            # Parameters without default values are required
            if param.default == inspect.Parameter.empty:
                required.append(name)
        
        return {
            "type": "object",
            "properties": properties,
            "required": required,
        }
    
    def _init_builtin_actions(self):
        """Register built-in browser actions."""
        self._actions.register("navigate", self._action_navigate)
        self._actions.register("new_tab", self._action_new_tab)
        self._actions.register("close_tab", self._action_close_tab)
        self._actions.register("list_tabs", self._action_list_tabs)
        self._actions.register("summarize", self._action_summarize)
        self._actions.register("search", self._action_search)
        self._actions.register("bookmark", self._action_bookmark)
        self._actions.register("set_theme", self._action_set_theme)
        # ... more actions
    
    async def chat(
        self,
        message: str,
        stream: bool = True,
        on_event: Optional[Callable[[Dict], None]] = None,
    ) -> str:
        """Send a message to the AI agent.
        
        Args:
            message: User message
            stream: Whether to stream the response
            on_event: Callback for AGUI events (streaming mode)
            
        Returns:
            AI response text
        """
        # Add user message to session
        self._session.add_message(role="user", content=message)
        
        # Prepare tools (function calling)
        tools = self._actions.get_tools_schema()
        
        # Get response from provider
        if stream:
            response = await self._complete_stream(
                messages=self._session.messages,
                tools=tools,
                on_event=on_event,
            )
        else:
            response = await self._complete(
                messages=self._session.messages,
                tools=tools,
            )
        
        # Handle tool calls
        while response.get("tool_calls"):
            for tool_call in response["tool_calls"]:
                result = await self._execute_action(
                    name=tool_call["name"],
                    args=tool_call["arguments"],
                )
                self._session.add_tool_result(
                    tool_call_id=tool_call["id"],
                    result=result,
                )
            
            # Get next response
            response = await self._complete(
                messages=self._session.messages,
                tools=tools,
            )
        
        # Add assistant response to session
        assistant_message = response.get("content", "")
        self._session.add_message(role="assistant", content=assistant_message)
        
        return assistant_message
    
    def register_action(
        self,
        name: str,
        handler: Callable,
        description: str = "",
        parameters: Optional[Dict] = None,
    ):
        """Register a custom action for the AI to call."""
        self._actions.register(name, handler, description, parameters)
    
    def action(self, name: str):
        """Decorator to register a custom action."""
        def decorator(func):
            self.register_action(name, func, func.__doc__ or "")
            return func
        return decorator
```

### 9. UI Components (AGUI/A2UI)

```typescript
// packages/auroraview-sdk/src/inject/plugins/ai-agent.ts

import { UIComponentSpec } from '../types';

/**
 * AGUI UI Component Types
 */
export interface AGUIComponents {
  // Text content
  markdown: { content: string };
  
  // Navigation feedback
  navigation_card: { url: string; title?: string };
  
  // Tab management
  tab_list: { tabs: Array<{ id: string; title: string; url: string; active: boolean }> };
  
  // Actions
  extension_toggle: { id: string; name: string; enabled: boolean };
  success_toast: { message: string };
  error_toast: { message: string };
  
  // Rich content
  code_snippet: { html: string; title: string; description?: string };
  media_player: { title: string; artist: string; type: 'audio' | 'video'; url: string };
  rich_link: { type: string; title: string; subtitle: string; url: string };
  file_viewer: { filename: string; code: string; language: string };
  
  // Interactive
  action_buttons: { buttons: Array<{ id: string; label: string; primary?: boolean }> };
  form_input: { fields: Array<{ name: string; type: string; label: string }> };
}

/**
 * Render AGUI UI Component
 */
export function renderAGUIComponent(spec: UIComponentSpec): HTMLElement {
  switch (spec.type) {
    case 'markdown':
      return renderMarkdown(spec.props as AGUIComponents['markdown']);
    case 'navigation_card':
      return renderNavigationCard(spec.props as AGUIComponents['navigation_card']);
    case 'tab_list':
      return renderTabList(spec.props as AGUIComponents['tab_list']);
    case 'code_snippet':
      return renderCodeSnippet(spec.props as AGUIComponents['code_snippet']);
    // ... more components
    default:
      return renderUnknown(spec);
  }
}
```

### 10. Modern Browser UI (Gemini-Inspired)

The browser UI will be updated to include:

1. **Side Panel** - Collapsible AI assistant panel
2. **Chat Interface** - Real-time streaming messages
3. **History View** - Chat session history
4. **Discover View** - Quick action cards
5. **AGUI Components** - Rich UI cards for responses

Design principles (from UI/UX Pro Max):
- Clean, minimal interface
- Glass morphism effects
- Dark/Light mode support
- Smooth animations
- Accessible components

### 11. Implementation Plan

#### Phase 1: Core AI Agent Crate (Week 1-2) ✅ Done

1. [x] Create `crates/auroraview-ai-agent/`
   - Implemented: `agent.rs`, `error.rs`, `lib.rs`, `message.rs`, `session.rs`
   - `actions/`: `mod.rs`, `registry.rs`
   - `protocol/`: `a2ui.rs`, `agui.rs`, `mod.rs`
   - `providers/`: `mod.rs`, `types.rs`, `wrapper.rs`
   - `ui/`: `mod.rs`
   - `assets/`: `ai_browser.html`
2. [x] Implement Provider trait and types (`providers/types.rs`)
3. [x] Provider wrapper supporting multiple providers (`providers/wrapper.rs`)
4. [ ] Implement OpenAI provider (Python layer done)
5. [ ] Implement Anthropic provider (Python layer done)
6. [ ] Implement Gemini provider (Python layer done)
7. [ ] Implement Ollama provider (local)
8. [ ] Add custom provider bridge

#### Phase 2: AGUI/A2UI Protocol (Week 2-3) ✅ Done

1. [x] Implement AGUI event types (`protocol/agui.rs`)
2. [x] Implement A2UI component specs (`protocol/a2ui.rs`)
3. [x] Add SSE streaming support
4. [ ] Implement state synchronization
5. [ ] Add human-in-the-loop interrupts

#### Phase 3: Browser Actions (Week 3-4) ✅ Done

1. [x] Implement action registry (`actions/registry.rs`)
2. [x] Add navigation actions (`actions/mod.rs`)
3. [x] Add tab management actions
4. [x] Add content actions (summarize, extract)
5. [x] Add bookmark actions
6. [x] Add UI actions (toast, theme)

#### Phase 4: Modern UI (Week 4-5) ✅ Done

1. [x] Create AI browser HTML (`assets/ai_browser.html`)
   - 完整的 Gemini 风格浏览器界面
   - 包含 Tab 管理、URL 栏、Chat Panel
   - 支持 Light/Dark 主题
2. [x] Implement chat interface
   - Gallery: `AISidebar.tsx` 组件
   - React hooks: `useAIAgent.ts`
   - 完整的消息列表、输入框、发送按钮
3. [x] Implement AGUI component renderer
   - AGUI 事件订阅: `agui:text_message_start/content/end`
   - 流式响应渲染支持
   - 错误处理和状态管理
4. [x] Add session management
   - 会话清除功能 (`clearSession`)
   - 消息历史管理
5. [x] Apply Gemini-inspired styling
   - 渐变 Logo、圆角设计、动画效果
   - 响应式布局
6. [x] Model selector with multi-provider support
7. [x] Tool discovery display
8. [ ] Update browser_controller.html with AI panel (Optional - 独立组件)

#### Phase 5: Python Bindings (Week 5-6) ✅ Done

1. [x] Implement Python AIAgent class (`python/auroraview/ai/agent.py`)
2. [x] Implement Python AIProvider base class and config (`config.py`)
3. [x] Implement AGUI protocol Python version (`protocol.py`)
4. [x] Implement tool registry (`tools.py`)
5. [x] Add action decorator API
6. [x] Create convenience functions

#### Phase 6: Documentation & Gallery (Week 6-7) ✅ Done

1. [x] Create Gallery demos
   - `AISidebar.tsx` - 完整的 AI 聊天侧边栏组件
   - `useAIAgent.ts` - React hook 封装
   - 类型定义: `types/ai.ts`
2. [x] Write API documentation (包含在 RFC 中)
   - Python API 使用示例
   - Sidebar mode 配置
   - Auto-discovery 机制说明
3. [x] Add integration examples
   - DCC 场景示例 (Maya API 绑定)
   - Custom provider 示例
   - AGUI 事件处理示例
4. [ ] Write standalone tutorial guides (可选 - 未来改进)

## API Examples

### Basic Usage

```python
from auroraview import Browser
from auroraview.ai import AIAgent, AIConfig, AIProviderType

# Create browser with AI agent
browser = Browser(title="AI Browser")

agent = AIAgent(
    browser=browser,
    config=AIConfig(
        provider=AIProviderType.GEMINI,
        model="gemini-2.0-flash-exp",
        api_key=os.environ["GEMINI_API_KEY"],
    )
)

# Natural language control
await agent.chat("Open GitHub")
await agent.chat("Search for Python web frameworks")
await agent.chat("Bookmark the top result")
await agent.chat("Summarize this page")
```

### Sidebar Mode (Recommended)

Attach AI Agent as a sidebar to any WebView application:

```python
from auroraview import WebView
from auroraview.ai import AIAgent, AIConfig

# Create your application WebView
webview = WebView(title="My DCC Tool Panel")

# Bind your application APIs
@webview.bind_call("api.export_scene")
def export_scene(format: str = "fbx", path: str = None):
    """Export current scene to specified format.
    
    Args:
        format: Export format (fbx, obj, usd)
        path: Export path (optional)
    """
    import maya.cmds as cmds
    export_path = path or f"/tmp/scene.{format}"
    cmds.file(export_path, exportAll=True, type=format)
    return {"exported": export_path, "format": format}

@webview.bind_call("api.create_primitive")
def create_primitive(shape: str = "cube", size: float = 1.0, name: str = None):
    """Create a primitive geometry in the scene.
    
    Args:
        shape: Geometry type (cube, sphere, cylinder, plane)
        size: Size
        name: Object name
    """
    import maya.cmds as cmds
    
    if shape == "cube":
        result = cmds.polyCube(w=size, h=size, d=size, name=name or "cube")
    elif shape == "sphere":
        result = cmds.polySphere(r=size/2, name=name or "sphere")
    elif shape == "cylinder":
        result = cmds.polyCylinder(r=size/2, h=size, name=name or "cylinder")
    else:
        result = cmds.polyPlane(w=size, h=size, name=name or "plane")
    
    return {"created": result[0], "shape": shape, "size": size}

@webview.bind_call("api.list_objects")
def list_objects(type_filter: str = None):
    """List objects in the scene."""
    import maya.cmds as cmds
    
    if type_filter:
        objects = cmds.ls(type=type_filter)
    else:
        objects = cmds.ls(dag=True, visible=True)
    
    return {"objects": objects, "count": len(objects)}

# Create AI Agent as sidebar (auto-discovers all bound APIs above)
agent = AIAgent.as_sidebar(
    webview,
    config=AIConfig(
        provider="gemini",
        model="gemini-2.0-flash-exp",
    )
)

# Start application
webview.show()

# Now users can chat with AI in the sidebar:
# - "Create a cube named 'hero_cube' with size 2"
# - "List all mesh objects in the scene"
# - "Export the scene as FBX"
# AI will automatically call the corresponding api.* methods!
```

### Auto API Discovery Explained

AI Agent automatically converts WebView-bound methods to AI tools:

```python
from auroraview import WebView
from auroraview.ai import AIAgent

webview = WebView()

# Method 1: Using bind_call decorator
@webview.bind_call("tools.calculate")
def calculate(expression: str) -> dict:
    """Calculate a mathematical expression.
    
    Args:
        expression: Mathematical expression, e.g., "2 + 2 * 3"
    
    Returns:
        Dictionary containing the calculation result
    """
    result = eval(expression)  # Example only, use safe evaluation in production
    return {"expression": expression, "result": result}

# Method 2: Using bind_api to bind an entire class
class SceneAPI:
    def get_selection(self) -> dict:
        """Get currently selected objects."""
        return {"selected": ["cube1", "sphere1"]}
    
    def set_attribute(self, object: str, attr: str, value: float) -> dict:
        """Set an object attribute value.
        
        Args:
            object: Object name
            attr: Attribute name (e.g., translateX, rotateY)
            value: New attribute value
        """
        return {"object": object, "attr": attr, "value": value, "status": "set"}

webview.bind_api(SceneAPI(), namespace="scene")

# Create Agent - auto-discovers all bound methods
agent = AIAgent.as_sidebar(webview, auto_discover_apis=True)

# AI can now use these tools:
# - tools.calculate: "Calculate a mathematical expression"
# - scene.get_selection: "Get currently selected objects"
# - scene.set_attribute: "Set an object attribute value"

webview.show()
```

### Sidebar UI Configuration

```python
from auroraview import WebView
from auroraview.ai import AIAgent, AIConfig, SidebarConfig

webview = WebView(title="My Application")

# Customize sidebar configuration
sidebar_config = SidebarConfig(
    position="right",           # "left" or "right"
    width=380,                  # Initial width in pixels
    min_width=280,              # Minimum width
    max_width=600,              # Maximum width
    collapsed=False,            # Initially collapsed
    resizable=True,             # Can resize
    keyboard_shortcut="Ctrl+Shift+A",  # Toggle shortcut
    theme="auto",               # "light", "dark", or "auto"
)

agent = AIAgent.as_sidebar(
    webview,
    config=AIConfig(provider="gemini"),
    sidebar_config=sidebar_config,
)

webview.show()
```

### Custom Provider

```python
from auroraview.ai import AIAgent, AIProvider

class MyLLMProvider(AIProvider):
    def name(self) -> str:
        return "my-llm"
    
    async def complete(self, messages, tools=None, **kwargs):
        # Call your custom LLM API
        response = await my_llm_api.chat(messages)
        return {"content": response.text}
    
    async def complete_stream(self, messages, tools=None, on_event=None, **kwargs):
        async for chunk in my_llm_api.chat_stream(messages):
            if on_event:
                on_event({"type": "text_delta", "delta": chunk.text})

agent = AIAgent(browser=browser, provider=MyLLMProvider())
```

### Custom Actions (DCC Scenario)

In addition to auto-discovered APIs, you can manually register additional actions:

```python
# In Maya
from auroraview.ai import AIAgent

agent = AIAgent(browser=browser)

@agent.action("export_scene")
def export_scene(format: str = "fbx", path: str = None):
    """Export the current Maya scene."""
    import maya.cmds as cmds
    
    export_path = path or f"/tmp/scene.{format}"
    cmds.file(export_path, exportAll=True, type=format)
    return {"exported": export_path}

@agent.action("create_cube")
def create_cube(size: float = 1.0, name: str = "cube"):
    """Create a cube in the Maya scene."""
    import maya.cmds as cmds
    
    cube = cmds.polyCube(w=size, h=size, d=size, name=name)
    return {"created": cube[0]}

# Now AI can use these actions
await agent.chat("Create a cube named 'myBox' with size 2")
await agent.chat("Export the scene as FBX")
```

### AGUI Event Handling

```python
def on_event(event: dict):
    if event["type"] == "TEXT_MESSAGE_CONTENT":
        print(event["delta"], end="", flush=True)
    elif event["type"] == "TOOL_CALL_START":
        print(f"\n[Calling: {event['tool_name']}]")
    elif event["type"] == "UI_COMPONENT":
        render_component(event["component"])

await agent.chat(
    "Search for Python tutorials and summarize the top 3",
    on_event=on_event
)
```

## References

- [AG-UI Protocol](https://github.com/ag-ui-protocol/ag-ui)
- [A2UI Protocol](https://developers.googleblog.com/introducing-a2ui-an-open-project-for-agent-driven-interfaces/)
- [Gemini Browser](https://github.com/example/gemini-browser)
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling)
- [Anthropic Tool Use](https://docs.anthropic.com/claude/docs/tool-use)
- [RFC 0007: WebView Browser Unified Architecture](./0007-webview-browser-unified-architecture.md)
