# RFC 0008: AI Agent 集成与 AGUI/A2UI 协议

## 概述

为 AuroraView 添加 AI Agent 能力，实现自然语言控制浏览器和 WebView。支持多种 AI 供应商（OpenAI、Anthropic Claude、Google Gemini、Azure OpenAI、本地模型），并提供可扩展的 Python 绑定。实现 AGUI（Agent-User Interaction Protocol）和 A2UI（Agent to UI）协议，标准化 AI 与前端的通信。

## 动机

### 当前限制

1. **无 AI 集成**：AuroraView 缺乏原生 AI 能力进行智能浏览
2. **仅手动控制**：用户必须手动导航、搜索和交互
3. **无标准协议**：AI Agent 与 UI 通信缺乏统一方式
4. **扩展性有限**：用户无法轻松添加自定义 AI 供应商

### 目标

- **多供应商支持**：OpenAI、Claude、Gemini、Azure、本地模型（Ollama、LM Studio）
- **AGUI/A2UI 兼容**：遵循新兴的 AI-UI 交互标准
- **Python 可扩展**：允许用户实现自定义供应商
- **无缝集成**：AI 控制真实 WebView，而非模拟内容
- **DCC 兼容**：适用于 Maya、Houdini、Blender 和独立模式

### 使用场景

1. **自然语言导航**："打开 GitHub 并搜索 AuroraView"
2. **内容摘要**："总结这个页面"
3. **智能操作**："收藏所有与 Python 相关的标签页"
4. **代码生成**："用 Tailwind CSS 创建一个登录表单"
5. **DCC 工作流**："查找 Maya 脚本文档"
6. **自动调用应用 API**：AI 自动发现并调用 WebView 绑定的 Python API

## 设计

### 1. 架构概览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         AI Agent 集成架构                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    供应商层 (Rust + Python)                          │   │
│  │                                                                     │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌──────────────┐  │   │
│  │  │   OpenAI    │ │  Anthropic  │ │   Gemini    │ │ Azure OpenAI │  │   │
│  │  │ GPT-4/4o    │ │  Claude 3   │ │  2.0 Flash  │ │    GPT-4     │  │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └──────────────┘  │   │
│  │                                                                     │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────────┐   │   │
│  │  │   Ollama    │ │  LM Studio  │ │   自定义 Python 供应商       │   │   │
│  │  │  本地 LLM   │ │  本地 LLM   │ │   (用户可扩展)               │   │   │
│  │  └─────────────┘ └─────────────┘ └─────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    协议层 (AGUI + A2UI)                              │   │
│  │                                                                     │   │
│  │  ┌──────────────────────────┐  ┌──────────────────────────────┐    │   │
│  │  │   AGUI 事件 (16+)        │  │   A2UI UI 组件                │    │   │
│  │  │ - TEXT_MESSAGE_START     │  │ - 声明式 UI 规范              │    │   │
│  │  │ - TEXT_MESSAGE_CONTENT   │  │ - 组件目录                    │    │   │
│  │  │ - TEXT_MESSAGE_END       │  │ - 安全渲染                    │    │   │
│  │  │ - TOOL_CALL_START        │  │ - SSE + JSON-RPC              │    │   │
│  │  │ - TOOL_CALL_END          │  │                              │    │   │
│  │  │ - STATE_SYNC             │  │                              │    │   │
│  │  │ - ...                    │  │                              │    │   │
│  │  └──────────────────────────┘  └──────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    操作层 (浏览器控制)                                │   │
│  │                                                                     │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐          │   │
│  │  │ navigate  │ │  search   │ │ bookmark  │ │  execute  │          │   │
│  │  │  (导航)   │ │  (搜索)   │ │  (收藏)   │ │   (执行)  │          │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘          │   │
│  │                                                                     │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐          │   │
│  │  │  tab_*    │ │ summarize │ │ generate  │ │  custom   │          │   │
│  │  │(标签管理) │ │  (摘要)   │ │  (生成UI) │ │ (自定义)  │          │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    展示层 (前端)                                      │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │           现代聊天 UI (Gemini 浏览器风格)                     │   │   │
│  │  │                                                               │   │   │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │   │   │
│  │  │  │  聊天面板   │  │  历史记录   │  │  发现/快捷操作       │  │   │   │
│  │  │  │  - 消息    │  │  - 会话     │  │  - 能力展示          │  │   │   │
│  │  │  │  - 流式    │  │  - 搜索     │  │  - 快捷操作          │  │   │   │
│  │  │  │  - UI 卡片 │  │             │  │                     │  │   │   │
│  │  │  └─────────────┘  └─────────────┘  └─────────────────────┘  │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. 侧边栏/抽屉模式架构

AI Agent 支持两种展示模式：

1. **独立浏览器模式** - AI 作为完整的浏览器界面
2. **侧边栏/抽屉模式** - AI 作为可折叠的侧边栏附加到任意 WebView

```
┌───────────────────────────────────────────────────────────────────────────┐
│                          WebView + AI 侧边栏                               │
├───────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌─────────────────────────────────────────────┐  ┌───────────────────┐  │
│  │                                             │  │   AI Agent 侧边栏  │  │
│  │                                             │  │   ┌─────────────┐ │  │
│  │              主 WebView 内容                 │  │   │   聊天消息   │ │  │
│  │                                             │  │   │             │ │  │
│  │     （你的应用、工具面板、DCC 插件等）        │  │   │  用户: ...  │ │  │
│  │                                             │  │   │  AI: ...    │ │  │
│  │                                             │  │   │             │ │  │
│  │                                             │  │   └─────────────┘ │  │
│  │                                             │  │   ┌─────────────┐ │  │
│  │                                             │  │   │   输入框     │ │  │
│  │                                             │  │   └─────────────┘ │  │
│  └─────────────────────────────────────────────┘  └───────────────────┘  │
│                                                        ↑                  │
│                                                   可折叠/可调整大小        │
└───────────────────────────────────────────────────────────────────────────┘
```

**侧边栏特性**：
- 可折叠/展开（类似 VSCode 侧边栏）
- 可调整宽度
- 可拖拽到左/右侧
- 支持键盘快捷键（Ctrl+Shift+A 切换）
- 与主 WebView 共享上下文

### 3. 自动发现 WebView API

AI Agent **自动发现并使用** WebView 通过 `bind_call()` 绑定的所有 API 方法：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     API 自动发现流程                                     │
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
│  │                    API 注册表 (Python)                            │  │
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
│  │                    AI Agent 工具注册                               │  │
│  │                                                                  │  │
│  │  自动转换为 AI 工具定义：                                          │  │
│  │  {                                                               │  │
│  │    "name": "api.echo",                                           │  │
│  │    "description": "Echo a message back",  // 从 docstring 提取   │  │
│  │    "parameters": {                        // 从类型注解推断       │  │
│  │      "type": "object",                                           │  │
│  │      "properties": {"message": {"type": "string"}}               │  │
│  │    }                                                             │  │
│  │  }                                                               │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4. Crate 结构

```
crates/
├── auroraview-ai-agent/           # 新建：AI Agent 集成
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                 # 公共 API
│       ├── agent.rs               # AIAgent 结构体
│       ├── session.rs             # 聊天会话管理
│       ├── message.rs             # 消息类型
│       │
│       ├── providers/             # AI 供应商实现
│       │   ├── mod.rs
│       │   ├── provider.rs        # Provider trait
│       │   ├── openai.rs          # OpenAI (GPT-4, GPT-4o)
│       │   ├── anthropic.rs       # Anthropic (Claude 3)
│       │   ├── gemini.rs          # Google (Gemini 2.0)
│       │   ├── azure.rs           # Azure OpenAI
│       │   ├── ollama.rs          # Ollama (本地模型)
│       │   └── custom.rs          # 自定义供应商桥接
│       │
│       ├── protocol/              # AGUI/A2UI 协议
│       │   ├── mod.rs
│       │   ├── agui.rs            # AGUI 事件
│       │   ├── a2ui.rs            # A2UI 组件
│       │   └── streaming.rs       # SSE 流式传输
│       │
│       ├── actions/               # 浏览器操作 (Function Calling)
│       │   ├── mod.rs
│       │   ├── navigation.rs      # navigate, go_back, go_forward
│       │   ├── tabs.rs            # new_tab, close_tab, switch_tab
│       │   ├── content.rs         # summarize, search, bookmark
│       │   ├── ui.rs              # generate_ui, show_toast
│       │   └── registry.rs        # 操作注册表
│       │
│       └── ui/                    # UI 组件
│           ├── mod.rs
│           ├── components.rs      # 预定义 UI 组件
│           └── renderer.rs        # 组件渲染器
```

### 5. Provider Trait (Rust)

```rust
// crates/auroraview-ai-agent/src/providers/provider.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// AI Provider trait - 为新供应商实现此 trait
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// 供应商名称 (例如 "openai", "anthropic", "gemini")
    fn name(&self) -> &str;
    
    /// 发送消息并获取响应（非流式）
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError>;
    
    /// 发送消息并流式获取响应
    async fn complete_stream(
        &self,
        request: CompletionRequest,
        on_event: Box<dyn Fn(StreamEvent) + Send>,
    ) -> Result<(), AIError>;
    
    /// 检查供应商是否支持函数调用
    fn supports_function_calling(&self) -> bool;
    
    /// 检查供应商是否支持视觉（图像输入）
    fn supports_vision(&self) -> bool;
    
    /// 获取可用模型
    fn available_models(&self) -> Vec<ModelInfo>;
}

/// 流式事件，用于实时 UI 更新
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

### 6. AGUI 协议事件

```rust
// crates/auroraview-ai-agent/src/protocol/agui.rs

/// AGUI 事件类型（遵循 AG-UI 协议标准）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AGUIEvent {
    // === 文本消息事件 ===
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
    
    // === 工具调用事件 ===
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
    
    // === 状态同步事件 ===
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
    
    // === UI 生成事件 (A2UI) ===
    #[serde(rename = "UI_COMPONENT")]
    UIComponent {
        id: String,
        component: UIComponentSpec,
    },
    
    // === 生命周期事件 ===
    #[serde(rename = "RUN_START")]
    RunStart {
        run_id: String,
        thread_id: Option<String>,
    },
    
    #[serde(rename = "RUN_END")]
    RunEnd {
        run_id: String,
    },
    
    // === 人机协作事件 ===
    #[serde(rename = "INTERRUPT_REQUEST")]
    InterruptRequest {
        id: String,
        prompt: String,
        options: Vec<InterruptOption>,
    },
}
```

### 7. Python 绑定

```python
# python/auroraview/ai/agent.py

from typing import Optional, Callable, Any, Dict, List, Union
from enum import Enum
from abc import ABC, abstractmethod

class AIProviderType(Enum):
    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    GEMINI = "gemini"
    AZURE = "azure"
    OLLAMA = "ollama"
    CUSTOM = "custom"

class AIProvider(ABC):
    """自定义 AI 供应商基类。
    
    用户可以继承此类添加自定义 AI 后端支持。
    """
    
    @abstractmethod
    def name(self) -> str:
        """供应商名称。"""
        pass
    
    @abstractmethod
    async def complete(
        self,
        messages: List[Dict],
        tools: Optional[List[Dict]] = None,
        **kwargs
    ) -> Dict:
        """发送补全请求。"""
        pass

class AIAgent:
    """AI Agent，用于自然语言控制浏览器和 WebView。
    
    特性:
    - 自动发现 WebView 绑定的所有 API 方法
    - 支持侧边栏/抽屉模式附加到任意 WebView
    - 多 AI 供应商支持
    
    示例 - 独立浏览器模式:
        from auroraview import Browser
        from auroraview.ai import AIAgent, AIConfig
        
        browser = Browser(title="AI 浏览器")
        agent = AIAgent(browser=browser, config=AIConfig(provider="gemini"))
        
        await agent.chat("打开 GitHub 并搜索 Python")
    
    示例 - 侧边栏模式（附加到现有 WebView）:
        from auroraview import WebView
        from auroraview.ai import AIAgent
        
        # 创建你的应用 WebView
        webview = WebView(title="我的应用")
        
        # 绑定一些 API 方法
        @webview.bind_call("api.export_scene")
        def export_scene(format: str = "fbx"):
            '''导出当前场景'''
            return {"status": "exported"}
        
        # 创建 AI Agent 作为侧边栏
        agent = AIAgent.as_sidebar(webview, config=AIConfig(provider="gemini"))
        
        # AI 现在可以自动使用 export_scene 方法
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
        
        # 自动发现 WebView 绑定的 API
        if auto_discover_apis and self.webview:
            self._discover_webview_apis()
    
    @classmethod
    def as_sidebar(
        cls,
        webview: "WebView",
        config: Optional["AIConfig"] = None,
        **kwargs
    ) -> "AIAgent":
        """创建作为侧边栏附加到 WebView 的 AI Agent。
        
        Args:
            webview: 要附加侧边栏的 WebView 实例
            config: AI 配置
            **kwargs: 其他配置参数
        
        Returns:
            配置为侧边栏模式的 AIAgent 实例
        
        Example:
            webview = WebView(title="我的工具")
            agent = AIAgent.as_sidebar(webview, config=AIConfig(provider="gemini"))
        """
        agent = cls(webview=webview, config=config, **kwargs)
        agent._enable_sidebar_mode()
        return agent
    
    def _enable_sidebar_mode(self):
        """启用侧边栏模式，注入侧边栏 UI 到 WebView。"""
        # 注入侧边栏 CSS 和 JS
        self.webview.eval_js(self._get_sidebar_injection_script())
        
        # 绑定侧边栏控制方法
        @self.webview.bind_call("ai.toggle_sidebar")
        def toggle_sidebar():
            self.webview.eval_js("window.__auroraview_ai_sidebar.toggle()")
            return {"toggled": True}
        
        @self.webview.bind_call("ai.chat")
        def sidebar_chat(message: str):
            return self.chat_sync(message)
    
    def _discover_webview_apis(self):
        """自动发现并注册 WebView 绑定的所有 API 方法。"""
        if not hasattr(self.webview, 'get_bound_methods'):
            return
        
        bound_methods = self.webview.get_bound_methods()
        
        for method_name in bound_methods:
            # 跳过内部方法
            if method_name.startswith('_') or method_name.startswith('ai.'):
                continue
            
            # 获取函数引用以提取 docstring 和类型注解
            func = self.webview._bound_functions.get(method_name)
            if func is None:
                continue
            
            # 从 docstring 获取描述
            description = func.__doc__ or f"调用 {method_name}"
            
            # 从类型注解推断参数 schema（如果可用）
            parameters = self._infer_parameters_schema(func)
            
            # 注册为 AI 工具
            self._actions.register_from_webview(
                name=method_name,
                description=description,
                parameters=parameters,
                webview=self.webview,
            )
    
    def _infer_parameters_schema(self, func: Callable) -> Dict:
        """从函数签名推断 JSON Schema 参数定义。"""
        import inspect
        sig = inspect.signature(func)
        
        properties = {}
        required = []
        
        for name, param in sig.parameters.items():
            if name in ('self', 'cls'):
                continue
            
            prop = {"type": "string"}  # 默认类型
            
            # 尝试从类型注解推断类型
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
            
            # 无默认值的参数为必需
            if param.default == inspect.Parameter.empty:
                required.append(name)
        
        return {
            "type": "object",
            "properties": properties,
            "required": required,
        }
    
    def register_action(
        self,
        name: str,
        handler: Callable,
        description: str = "",
        parameters: Optional[Dict] = None,
    ):
        """注册自定义操作供 AI 调用。"""
        self._actions.register(name, handler, description, parameters)
    
    def action(self, name: str):
        """装饰器：注册自定义操作。"""
        def decorator(func):
            self.register_action(name, func, func.__doc__ or "")
            return func
        return decorator
```

### 8. 实施计划

#### 第一阶段：核心 AI Agent Crate（第 1-2 周）✅ 已完成

1. [x] 创建 `crates/auroraview-ai-agent/`
   - 已实现：`agent.rs`, `error.rs`, `lib.rs`, `message.rs`, `session.rs`
   - `actions/`: `mod.rs`, `registry.rs`
   - `protocol/`: `a2ui.rs`, `agui.rs`, `mod.rs`
   - `providers/`: `mod.rs`, `types.rs`, `wrapper.rs`
   - `ui/`: `mod.rs`
   - `assets/`: `ai_browser.html`
2. [x] 实现 Provider trait 和类型 (`providers/types.rs`)
3. [x] Provider wrapper 支持多供应商 (`providers/wrapper.rs`)
4. [ ] 实现 OpenAI 供应商 (Python 层已实现)
5. [ ] 实现 Anthropic 供应商 (Python 层已实现)
6. [ ] 实现 Gemini 供应商 (Python 层已实现)
7. [ ] 实现 Ollama 供应商（本地）
8. [ ] 添加自定义供应商桥接

#### 第二阶段：AGUI/A2UI 协议（第 2-3 周）✅ 已完成

1. [x] 实现 AGUI 事件类型 (`protocol/agui.rs`)
2. [x] 实现 A2UI 组件规范 (`protocol/a2ui.rs`)
3. [x] 添加 SSE 流式支持
4. [ ] 实现状态同步
5. [ ] 添加人机协作中断

#### 第三阶段：浏览器操作（第 3-4 周）✅ 已完成

1. [x] 实现操作注册表 (`actions/registry.rs`)
2. [x] 添加导航操作 (`actions/mod.rs`)
3. [x] 添加标签管理操作
4. [x] 添加内容操作（摘要、提取）
5. [x] 添加书签操作
6. [x] 添加 UI 操作（toast、主题）

#### 第四阶段：现代 UI（第 4-5 周）- 进行中

1. [x] 创建 AI 浏览器 HTML (`assets/ai_browser.html`)
2. [ ] 更新 browser_controller.html，添加 AI 面板
3. [ ] 实现聊天界面
4. [ ] 实现 AGUI 组件渲染器
5. [ ] 添加会话管理
6. [ ] 应用 Gemini 风格设计

#### 第五阶段：Python 绑定（第 5-6 周）✅ 已完成

1. [x] 实现 Python AIAgent 类 (`python/auroraview/ai/agent.py`)
2. [x] 实现 Python AIProvider 基类和配置 (`config.py`)
3. [x] 实现 AGUI 协议 Python 版 (`protocol.py`)
4. [x] 实现工具注册表 (`tools.py`)
5. [x] 添加操作装饰器 API
6. [x] 创建便捷函数

#### 第六阶段：文档和 Gallery（第 6-7 周）- 待完成

1. [ ] 编写 API 文档
2. [ ] 创建 Gallery 演示
3. [ ] 编写教程指南
4. [ ] 添加集成示例

## API 示例

### 基础用法

```python
from auroraview import Browser
from auroraview.ai import AIAgent, AIConfig, AIProviderType

# 创建带 AI agent 的浏览器
browser = Browser(title="AI 浏览器")

agent = AIAgent(
    browser=browser,
    config=AIConfig(
        provider=AIProviderType.GEMINI,
        model="gemini-2.0-flash-exp",
        api_key=os.environ["GEMINI_API_KEY"],
    )
)

# 自然语言控制
await agent.chat("打开 GitHub")
await agent.chat("搜索 Python web 框架")
await agent.chat("收藏第一个结果")
await agent.chat("总结这个页面")
```

### 侧边栏模式（推荐）

将 AI Agent 作为侧边栏附加到你的任意 WebView 应用：

```python
from auroraview import WebView
from auroraview.ai import AIAgent, AIConfig

# 创建你的应用 WebView
webview = WebView(title="我的 DCC 工具面板")

# 绑定你的应用 API
@webview.bind_call("api.export_scene")
def export_scene(format: str = "fbx", path: str = None):
    """导出当前场景到指定格式。
    
    Args:
        format: 导出格式 (fbx, obj, usd)
        path: 导出路径（可选）
    """
    import maya.cmds as cmds
    export_path = path or f"/tmp/scene.{format}"
    cmds.file(export_path, exportAll=True, type=format)
    return {"exported": export_path, "format": format}

@webview.bind_call("api.create_primitive")
def create_primitive(shape: str = "cube", size: float = 1.0, name: str = None):
    """在场景中创建基本几何体。
    
    Args:
        shape: 几何体类型 (cube, sphere, cylinder, plane)
        size: 大小
        name: 对象名称
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
    """列出场景中的对象。"""
    import maya.cmds as cmds
    
    if type_filter:
        objects = cmds.ls(type=type_filter)
    else:
        objects = cmds.ls(dag=True, visible=True)
    
    return {"objects": objects, "count": len(objects)}

# 创建 AI Agent 作为侧边栏（自动发现上面绑定的所有 API）
agent = AIAgent.as_sidebar(
    webview,
    config=AIConfig(
        provider="gemini",
        model="gemini-2.0-flash-exp",
    )
)

# 启动应用
webview.show()

# 现在用户可以在侧边栏中与 AI 对话：
# - "创建一个名为 hero_cube 的立方体，大小为 2"
# - "列出场景中所有的 mesh 对象"
# - "将场景导出为 FBX 格式"
# AI 会自动调用对应的 api.* 方法！
```

### 自动 API 发现详解

AI Agent 会自动将 WebView 绑定的方法转换为 AI 工具：

```python
from auroraview import WebView
from auroraview.ai import AIAgent

webview = WebView()

# 方式1：使用 bind_call 装饰器
@webview.bind_call("tools.calculate")
def calculate(expression: str) -> dict:
    """计算数学表达式。
    
    Args:
        expression: 数学表达式，如 "2 + 2 * 3"
    
    Returns:
        包含计算结果的字典
    """
    result = eval(expression)  # 仅示例，生产环境请使用安全的计算方式
    return {"expression": expression, "result": result}

# 方式2：使用 bind_api 绑定整个类
class SceneAPI:
    def get_selection(self) -> dict:
        """获取当前选中的对象。"""
        return {"selected": ["cube1", "sphere1"]}
    
    def set_attribute(self, object: str, attr: str, value: float) -> dict:
        """设置对象属性值。
        
        Args:
            object: 对象名称
            attr: 属性名称（如 translateX, rotateY）
            value: 新的属性值
        """
        return {"object": object, "attr": attr, "value": value, "status": "set"}

webview.bind_api(SceneAPI(), namespace="scene")

# 创建 Agent - 自动发现所有绑定的方法
agent = AIAgent.as_sidebar(webview, auto_discover_apis=True)

# AI 现在可以使用以下工具：
# - tools.calculate: "计算数学表达式"
# - scene.get_selection: "获取当前选中的对象"
# - scene.set_attribute: "设置对象属性值"

webview.show()
```

### 自定义供应商

```python
from auroraview.ai import AIAgent, AIProvider

class MyLLMProvider(AIProvider):
    def name(self) -> str:
        return "my-llm"
    
    async def complete(self, messages, tools=None, **kwargs):
        # 调用你的自定义 LLM API
        response = await my_llm_api.chat(messages)
        return {"content": response.text}

agent = AIAgent(browser=browser, provider=MyLLMProvider())
```

### 自定义操作（DCC 场景）

```python
# 在 Maya 中
from auroraview.ai import AIAgent

agent = AIAgent(browser=browser)

@agent.action("export_scene")
def export_scene(format: str = "fbx", path: str = None):
    """导出当前 Maya 场景。"""
    import maya.cmds as cmds
    
    export_path = path or f"/tmp/scene.{format}"
    cmds.file(export_path, exportAll=True, type=format)
    return {"exported": export_path}

@agent.action("create_cube")
def create_cube(size: float = 1.0, name: str = "cube"):
    """在 Maya 场景中创建立方体。"""
    import maya.cmds as cmds
    
    cube = cmds.polyCube(w=size, h=size, d=size, name=name)
    return {"created": cube[0]}

# 现在 AI 可以使用这些操作
await agent.chat("创建一个名为 'myBox' 大小为 2 的立方体")
await agent.chat("将场景导出为 FBX")
```

### 侧边栏 UI 配置

```python
from auroraview import WebView
from auroraview.ai import AIAgent, AIConfig, SidebarConfig

webview = WebView(title="我的应用")

# 自定义侧边栏配置
sidebar_config = SidebarConfig(
    position="right",           # "left" 或 "right"
    width=380,                  # 初始宽度（像素）
    min_width=280,              # 最小宽度
    max_width=600,              # 最大宽度
    collapsed=False,            # 初始是否折叠
    resizable=True,             # 是否可调整大小
    keyboard_shortcut="Ctrl+Shift+A",  # 切换快捷键
    theme="auto",               # "light", "dark", 或 "auto"
)

agent = AIAgent.as_sidebar(
    webview,
    config=AIConfig(provider="gemini"),
    sidebar_config=sidebar_config,
)

webview.show()
```

### AGUI 事件处理

```python
def on_event(event: dict):
    if event["type"] == "TEXT_MESSAGE_CONTENT":
        print(event["delta"], end="", flush=True)
    elif event["type"] == "TOOL_CALL_START":
        print(f"\n[调用工具: {event['tool_name']}]")
    elif event["type"] == "UI_COMPONENT":
        render_component(event["component"])

await agent.chat(
    "搜索 Python 教程并总结前 3 个结果",
    on_event=on_event
)
```

## 参考资料

- [AG-UI 协议](https://github.com/ag-ui-protocol/ag-ui)
- [A2UI 协议](https://developers.googleblog.com/introducing-a2ui-an-open-project-for-agent-driven-interfaces/)
- [Gemini 浏览器](https://github.com/example/gemini-browser)
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling)
- [Anthropic Tool Use](https://docs.anthropic.com/claude/docs/tool-use)
- [RFC 0007: WebView Browser 统一架构](./0007-webview-browser-unified-architecture.md)
