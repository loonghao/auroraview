# RFC 0012: AI Agent Sidebar for DCC Integration

## Summary

Extend AuroraView's AI Agent capabilities with a specialized sidebar mode for DCC (Digital Content Creation) applications. Enable natural language control of Maya, Houdini, and Blender by automatically discovering and exposing bound Python APIs as AI-callable tools, leveraging the existing `bind_call`/`bind_api` pattern.

## Motivation

### Current Limitations

1. **Manual Scripting Required**: DCC artists must write MEL/Python/VEX scripts for automation
2. **Steep Learning Curve**: Each DCC has different APIs and conventions
3. **No AI Integration**: Existing AI agent frameworks lack DCC-specific support
4. **Disconnected Tools**: External AI tools can't directly call DCC application APIs

### Goals

- **Zero-Config Tool Discovery**: Automatically convert `bind_call`/`bind_api` methods to AI tools
- **DCC-Native Integration**: Work seamlessly within Maya/Houdini/Blender Qt windows
- **Multi-Provider Support**: OpenAI, Claude, Gemini, DeepSeek, Ollama
- **AG-UI Protocol**: Standardized streaming events for responsive UI
- **DOM Control**: AI can also manipulate the WebView UI when needed

### Use Cases

1. **Natural Language Scene Control**: "Export this scene as FBX with selected objects only"
2. **Selection Operations**: "Select all meshes with more than 10,000 polygons"
3. **Material Workflow**: "Apply a red metallic material to selected objects"
4. **Animation Tasks**: "Set keyframes on selected objects at frames 1, 12, and 24"
5. **Render Management**: "Render the current view with Arnold at 1920x1080"
6. **Pipeline Integration**: "Publish this asset to the asset library"

## Design

### 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    DCC Application (Maya/Houdini/Blender)                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        Qt Main Window                                  │  │
│  │  ┌─────────────────────────────────────────┬─────────────────────┐    │  │
│  │  │                                         │   AI Agent Sidebar   │    │  │
│  │  │          AuroraView WebView             │  ┌───────────────┐  │    │  │
│  │  │                                         │  │  Model Select │  │    │  │
│  │  │     (Custom Tool UI / Panels)           │  ├───────────────┤  │    │  │
│  │  │                                         │  │ Chat Messages │  │    │  │
│  │  │     ┌───────────────────────────┐       │  │  User: ...    │  │    │  │
│  │  │     │   bind_call("scene.export")│       │  │  AI: ...      │  │    │  │
│  │  │     │   bind_call("selection.*") │       │  │               │  │    │  │
│  │  │     │   bind_api(my_tools)       │       │  ├───────────────┤  │    │  │
│  │  │     └───────────────────────────┘       │  │ Tool Status   │  │    │  │
│  │  │                                         │  ├───────────────┤  │    │  │
│  │  │                                         │  │ [Input Field] │  │    │  │
│  │  └─────────────────────────────────────────┴──┴───────────────┴──┘    │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│                            Python Layer                                      │
│  ┌─────────────┐  ┌─────────────────┐  ┌──────────────────────────────────┐ │
│  │  QtWebView  │──│    AIAgent      │──│         ToolRegistry             │ │
│  │  bind_*()   │  │  as_sidebar()   │  │  (auto-discover from bind_call)  │ │
│  └─────────────┘  └─────────────────┘  └──────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│                          AI Provider Layer                                   │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌──────────┐  │
│  │ OpenAI  │ │ Claude  │ │ Gemini  │ │DeepSeek │ │  Groq   │ │  Ollama  │  │
│  │ GPT-4o  │ │Claude 4 │ │ 2.0Flash│ │   R1    │ │  Fast   │ │  Local   │  │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. API Auto-Discovery Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        API Auto-Discovery Flow                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    WebView API Registration                            │ │
│  │                                                                       │ │
│  │  @webview.bind_call("scene.export")                                   │ │
│  │  def export_scene(format: str = "fbx") -> dict:                       │ │
│  │      '''Export the current scene to specified format.'''              │ │
│  │      cmds.file(exportAll=True, type=format)                           │ │
│  │      return {"success": True}                                         │ │
│  │                                                                       │ │
│  │  @webview.bind_call("selection.get")                                  │ │
│  │  def get_selection() -> list[str]:                                    │ │
│  │      '''Get currently selected objects.'''                            │ │
│  │      return cmds.ls(selection=True)                                   │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                        │
│                                    ↓                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    ToolRegistry.discover_from_webview()               │ │
│  │                                                                       │ │
│  │  - Scans webview._bound_functions                                     │ │
│  │  - Extracts function signatures via inspect                           │ │
│  │  - Parses docstrings for descriptions                                 │ │
│  │  - Infers JSON Schema from type hints                                 │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                        │
│                                    ↓                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    Generated AI Tool Definitions                      │ │
│  │                                                                       │ │
│  │  {                                                                    │ │
│  │    "name": "scene.export",                                            │ │
│  │    "description": "Export the current scene to specified format.",    │ │
│  │    "parameters": {                                                    │ │
│  │      "type": "object",                                                │ │
│  │      "properties": {                                                  │ │
│  │        "format": {"type": "string", "default": "fbx"}                 │ │
│  │      }                                                                │ │
│  │    }                                                                  │ │
│  │  }                                                                    │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3. DCC Tool Categories

```python
from enum import Enum

class DCCToolCategory(Enum):
    """Categories for DCC-specific tools."""
    SCENE = "scene"           # Scene operations (open, save, export, import)
    SELECTION = "selection"   # Selection management (get, set, filter)
    TRANSFORM = "transform"   # Object transforms (move, rotate, scale)
    MATERIAL = "material"     # Material/shader operations
    ANIMATION = "animation"   # Animation/keyframe operations
    RENDER = "render"         # Rendering operations
    MODELING = "modeling"     # Modeling operations
    RIGGING = "rigging"       # Rigging operations
    CUSTOM = "custom"         # User-defined tools

class DCCType(Enum):
    """Supported DCC application types."""
    MAYA = "maya"
    HOUDINI = "houdini"
    BLENDER = "blender"
    NUKE = "nuke"
    UNREAL = "unreal"
    STANDALONE = "standalone"
```

### 4. DCC Tool Decorator

```python
from functools import wraps
from typing import Callable, Optional, List
from dataclasses import dataclass

@dataclass
class DCCToolMetadata:
    """Metadata for a DCC tool."""
    category: DCCToolCategory
    requires_selection: bool = False
    confirm: bool = False  # Show confirmation dialog
    dcc_types: Optional[List[DCCType]] = None  # Supported DCCs
    description: str = ""
    tags: List[str] = None

def dcc_tool(
    category: DCCToolCategory = DCCToolCategory.CUSTOM,
    requires_selection: bool = False,
    confirm: bool = False,
    dcc_types: Optional[List[DCCType]] = None,
    description: str = "",
    tags: Optional[List[str]] = None,
) -> Callable:
    """Decorator to mark a function as a DCC tool with metadata."""
    def decorator(func: Callable) -> Callable:
        @wraps(func)
        def wrapper(*args, **kwargs):
            return func(*args, **kwargs)

        # Attach metadata to function
        wrapper._dcc_tool_metadata = DCCToolMetadata(
            category=category,
            requires_selection=requires_selection,
            confirm=confirm,
            dcc_types=dcc_types,
            description=description or func.__doc__ or "",
            tags=tags or [],
        )
        return wrapper
    return decorator
```

### 5. Context-Aware System Prompts

```python
DCC_SYSTEM_PROMPTS = {
    "maya": """You are an AI assistant for Autodesk Maya.
You can help with scene management, modeling, animation, and rendering.
Available tools are automatically discovered from the application's API.

When using tools:
- Always confirm destructive operations (delete, export, save)
- Check selection before operations that require it
- Report results clearly to the user

Current capabilities are exposed via the tool list below.""",

    "houdini": """You are an AI assistant for SideFX Houdini.
You can help with procedural modeling, simulations, and VFX workflows.
Available tools are automatically discovered from the application's API.

When using tools:
- Prefer node-based workflows where applicable
- Explain HDA parameters when asked
- Support both SOPs and LOPs contexts""",

    "blender": """You are an AI assistant for Blender.
You can help with modeling, sculpting, animation, and rendering.
Available tools are automatically discovered from the application's API.

When using tools:
- Use the correct context (Object/Edit mode)
- Support both Cycles and EEVEE renderers
- Consider Geometry Nodes for procedural workflows""",
}
```

### 6. AG-UI Protocol Events for DCC

The sidebar uses AG-UI protocol for real-time updates, extended with DCC-specific events:

| Event | Description | Payload |
|-------|-------------|---------|
| `agui:run_started` | Chat run started | `{run_id, thread_id}` |
| `agui:text_message_start` | Text message started | `{message_id}` |
| `agui:text_message_content` | Streaming text chunk | `{message_id, delta}` |
| `agui:text_message_end` | Text message ended | `{message_id}` |
| `agui:thinking_text_message_content` | Reasoning text (DeepSeek R1) | `{delta}` |
| `agui:tool_call_start` | Tool execution started | `{tool_call_id, name}` |
| `agui:tool_call_args` | Tool arguments | `{tool_call_id, args}` |
| `agui:tool_call_end` | Tool execution completed | `{tool_call_id}` |
| `agui:tool_call_result` | Tool result | `{tool_call_id, result}` |
| `agui:run_finished` | Chat run completed | `{run_id}` |
| `agui:run_error` | Error occurred | `{run_id, error}` |
| `agui:dcc_selection_changed` | DCC selection changed | `{objects}` |
| `agui:dcc_scene_modified` | DCC scene modified | `{operation}` |

### 7. TypeScript Sidebar Component

```typescript
// packages/auroraview-sdk/src/components/AgentSidebar.tsx
import { useState, useCallback, useEffect } from 'react';
import { useAuroraView, useAuroraEvent } from '@auroraview/sdk/react';

interface Message {
  id: string;
  role: 'user' | 'assistant' | 'tool';
  content: string;
  toolCall?: { name: string; args: object; result?: object };
  timestamp: number;
}

interface AgentSidebarProps {
  position?: 'left' | 'right';
  width?: number;
  defaultOpen?: boolean;
}

export function AgentSidebar({
  position = 'right',
  width = 380,
  defaultOpen = false
}: AgentSidebarProps) {
  const av = useAuroraView();
  const [messages, setMessages] = useState<Message[]>([]);
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamingContent, setStreamingContent] = useState('');

  // Handle keyboard shortcut: Ctrl+Shift+A
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === 'A') {
        e.preventDefault();
        setIsOpen(prev => !prev);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Listen for AG-UI streaming events
  useAuroraEvent('agui:text_message_start', () => {
    setIsStreaming(true);
    setStreamingContent('');
  });

  useAuroraEvent('agui:text_message_content', (data) => {
    setStreamingContent(prev => prev + data.delta);
  });

  useAuroraEvent('agui:text_message_end', (data) => {
    setMessages(prev => [...prev, {
      id: data.message_id,
      role: 'assistant',
      content: streamingContent,
      timestamp: Date.now(),
    }]);
    setIsStreaming(false);
    setStreamingContent('');
  });

  useAuroraEvent('agui:tool_call_start', (data) => {
    setMessages(prev => [...prev, {
      id: data.tool_call_id,
      role: 'tool',
      content: `Executing: ${data.name}`,
      toolCall: { name: data.name, args: {} },
      timestamp: Date.now(),
    }]);
  });

  const sendMessage = useCallback(async (text: string) => {
    setMessages(prev => [...prev, {
      id: `msg_${Date.now()}`,
      role: 'user',
      content: text,
      timestamp: Date.now(),
    }]);
    await av.call('ai.chat_stream', { message: text });
  }, [av]);

  return (
    <div className={`agent-sidebar ${position} ${isOpen ? 'open' : ''}`}
         style={{ width: isOpen ? width : 0 }}>
      <SidebarHeader onClose={() => setIsOpen(false)} />
      <MessageList messages={messages} />
      {isStreaming && <StreamingMessage content={streamingContent} />}
      <InputArea onSend={sendMessage} disabled={isStreaming} />
    </div>
  );
}
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1-2) ✅ Complete

1. [x] Create `python/auroraview/ai/` module structure
   - `agent.py` - AIAgent class with sidebar mode
   - `config.py` - AIConfig with provider settings
   - `tool.py` - Tool and ToolRegistry classes
   - `providers/` - Provider implementations
2. [x] Implement ToolRegistry with auto-discovery
3. [x] Create AG-UI event emitter in Python
4. [x] TypeScript SDK components (`AgentSidebar.tsx`)
5. [x] Gallery AISidebar integration (`useAIAgent` hook)

### Phase 2: DCC Tool Integration (Week 2-3) 🔄 In Progress

1. [x] Create `dcc_tools.py` with decorators and categories
2. [ ] Implement DCC-specific system prompts
3. [ ] Add selection context provider
4. [ ] Add scene state context provider
5. [ ] Create tool confirmation dialogs
6. [ ] Error recovery and retry logic

### Phase 3: Enhanced UI (Week 3-4) 📋 Planned

1. [ ] Tool execution visualization
2. [ ] Thinking/reasoning display (for DeepSeek R1)
3. [ ] Conversation history persistence
4. [ ] Multi-turn tool chaining
5. [ ] Visual element selection (click to reference)

### Phase 4: Advanced Features (Week 4-5) 📋 Planned

1. [ ] Voice input support
2. [ ] Screenshot-based element location
3. [ ] Cross-page navigation
4. [ ] Form auto-fill
5. [ ] Plugin marketplace integration

## API Examples

### Basic Maya Integration

```python
from auroraview import QtWebView
from auroraview.ai import AIAgent, AIConfig
from auroraview.ai.dcc_tools import dcc_tool, DCCToolCategory

# Get Maya main window
import maya.OpenMayaUI as omui
from shiboken2 import wrapInstance
from PySide2.QtWidgets import QMainWindow

ptr = omui.MQtUtil.mainWindow()
maya_window = wrapInstance(int(ptr), QMainWindow)

# Create WebView
webview = QtWebView(parent=maya_window, url="http://localhost:5173")

# Bind DCC APIs with decorators
@webview.bind_call("scene.export")
@dcc_tool(category=DCCToolCategory.SCENE, confirm=True)
def export_scene(format: str = "fbx", path: str = "") -> dict:
    """Export the current Maya scene to the specified format.

    Args:
        format: Export format (fbx, obj, abc)
        path: Output file path (auto-generated if empty)

    Returns:
        dict with success status and file path
    """
    import maya.cmds as cmds
    if not path:
        path = cmds.file(q=True, sn=True).replace(".ma", f".{format}")
    cmds.file(path, exportAll=True, type=format.upper(), force=True)
    return {"success": True, "path": path}

@webview.bind_call("selection.get")
@dcc_tool(category=DCCToolCategory.SELECTION)
def get_selection() -> list:
    """Get currently selected objects in the scene."""
    import maya.cmds as cmds
    return cmds.ls(selection=True) or []

@webview.bind_call("selection.set")
@dcc_tool(category=DCCToolCategory.SELECTION, requires_selection=False)
def set_selection(objects: list) -> dict:
    """Set the current selection to the specified objects.

    Args:
        objects: List of object names to select
    """
    import maya.cmds as cmds
    cmds.select(objects, replace=True)
    return {"success": True, "selected": objects}

# Create AI Agent with auto-discovery
agent = AIAgent.as_sidebar(
    webview,
    config=AIConfig.openai(model="gpt-4o"),
    auto_discover_apis=True,
)

webview.show()
```

### Gallery AI Demo

```tsx
// gallery/src/pages/AIDemo.tsx
import { AISidebar } from '../components/AISidebar';
import { useAIAgent } from '../hooks/useAIAgent';

export function AIDemo() {
  const { tools, isReady } = useAIAgent();

  return (
    <div className="ai-demo-page">
      <div className="main-content">
        <h1>AI Agent Demo</h1>
        <p>Available tools: {tools.length}</p>
        <ul>
          {tools.map(tool => (
            <li key={tool.name}>
              <strong>{tool.name}</strong>: {tool.description}
            </li>
          ))}
        </ul>
      </div>
      <AISidebar />
    </div>
  );
}
```

## Comparison with Existing Solutions

| Feature | AuroraView AI | Browser-Use | Stagehand | Midscene |
|---------|--------------|-------------|-----------|----------|
| DCC Integration | ✅ Native | ❌ | ❌ | ❌ |
| API Auto-Discovery | ✅ bind_call | ❌ | ❌ | ❌ |
| Embedded Sidebar | ✅ | ❌ | ❌ | ❌ |
| DOM Control | ✅ | ✅ | ✅ | ✅ |
| Multi-Provider | ✅ 6+ | ✅ | ✅ | ✅ |
| AG-UI Protocol | ✅ | ❌ | ❌ | ❌ |
| Python Native | ✅ | ✅ | ❌ (TS) | ❌ (TS) |
| Qt Integration | ✅ | ❌ | ❌ | ❌ |
| Tool Confirmation | ✅ | ❌ | ❌ | ❌ |

## References

- [RFC 0008: AI Agent Integration with AGUI/A2UI Protocol](./0008-ai-agent-integration.md)
- [RFC 0009: AI Chat UI Integration](./0009-ai-chat-ui-integration.md)
- [AG-UI Protocol Specification](https://docs.ag-ui.com/)
- [Browser-Use](https://github.com/browser-use/browser-use)
- [Stagehand by Browserbase](https://www.browserbase.com/blog/stagehand-v3)
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling)
- [Anthropic Tool Use](https://docs.anthropic.com/claude/docs/tool-use)
```

