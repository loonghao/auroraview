# RFC 0009: AI Chat UI Integration with CopilotKit

## Summary

Integrate CopilotKit as the primary AI chat UI solution for AuroraView, replacing the custom implementation. This provides a production-ready, well-tested chat interface while AuroraView focuses on its core strengths: Python-JS bridge, WebView embedding, and DCC integration.

## Motivation

### Current Problems

1. **Encoding Issues**: UTF-8 streaming content causes garbled text due to JavaScript string escaping issues in the Python→JS event bridge
2. **Event Timing**: Race conditions between event subscription and triggering
3. **Maintenance Burden**: Custom chat UI requires ongoing maintenance for features like:
   - Markdown rendering
   - Code highlighting
   - Tool call visualization
   - Thinking/reasoning display
   - Message history management

### Why CopilotKit

1. **Native AG-UI Support**: CopilotKit implements the AG-UI protocol that we already support in our Python backend
2. **Production Ready**: Battle-tested in production environments
3. **Rich Features**: Built-in support for:
   - Streaming text with proper UTF-8 handling
   - Tool call visualization
   - Thinking/reasoning display
   - Message history
   - Customizable UI components
4. **Active Development**: Regular updates and community support

## Design

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend (React)                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┐│
│  │              CopilotKit (@copilotkit/react-*)               ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  ││
│  │  │CopilotChat  │  │CopilotSidebar│ │ Custom Components  │  ││
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘  ││
│  └─────────────────────────────────────────────────────────────┘│
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │           AuroraView SDK (@auroraview/sdk)                  ││
│  │  ┌─────────────────────────────────────────────────────────┐││
│  │  │     useAuroraViewAI() - Bridge to Python AG-UI          │││
│  │  └─────────────────────────────────────────────────────────┘││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ WebView Bridge (JSON-RPC)
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Python Backend                               │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                 auroraview.ai.AIAgent                       ││
│  │  • AG-UI Protocol Implementation                            ││
│  │  • Multi-provider Support (OpenAI, Anthropic, DeepSeek)     ││
│  │  • Tool Registration & Execution                            ││
│  │  • Session Management                                       ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### SDK Integration

Add a new export to `@auroraview/sdk` for AI integration:

```typescript
// packages/auroraview-sdk/src/adapters/copilotkit.ts

import { CopilotRuntimeClient, Message } from '@copilotkit/runtime-client-gql';

/**
 * AuroraView CopilotKit Runtime Adapter
 * 
 * Bridges CopilotKit's runtime to AuroraView's Python AG-UI backend.
 */
export class AuroraViewCopilotRuntime {
  private client: AuroraViewClient;
  
  constructor(client: AuroraViewClient) {
    this.client = client;
  }
  
  /**
   * Process a CopilotKit request through AuroraView's Python backend
   */
  async process(request: CopilotRuntimeRequest): Promise<ReadableStream> {
    // Convert CopilotKit format to AG-UI format
    const aguiRequest = this.toAGUIRequest(request);
    
    // Start streaming via AuroraView bridge
    const response = await this.client.call('ai.chat_stream', aguiRequest);
    
    // Return a ReadableStream that emits CopilotKit-compatible events
    return this.createCopilotStream();
  }
  
  private createCopilotStream(): ReadableStream {
    // Subscribe to AG-UI events and convert to CopilotKit format
    // ...
  }
}

/**
 * Hook to use CopilotKit with AuroraView backend
 */
export function useAuroraViewCopilot() {
  const { client, isReady } = useAuroraView();
  
  const runtime = useMemo(() => {
    if (!client) return null;
    return new AuroraViewCopilotRuntime(client);
  }, [client]);
  
  return { runtime, isReady };
}
```

### Usage Example

```tsx
// App.tsx
import { CopilotKit, CopilotSidebar } from '@copilotkit/react-ui';
import { useAuroraViewCopilot } from '@auroraview/sdk/copilotkit';

function App() {
  const { runtime, isReady } = useAuroraViewCopilot();
  
  if (!isReady) return <Loading />;
  
  return (
    <CopilotKit runtime={runtime}>
      <CopilotSidebar>
        <YourApp />
      </CopilotSidebar>
    </CopilotKit>
  );
}
```

### Alternative: Direct HTTP Endpoint

For simpler integration, expose the AG-UI backend as an HTTP endpoint:

```python
# Python backend exposes AG-UI HTTP endpoint
from auroraview.ai import AIAgent

@webview.bind_call("ai.get_runtime_url")
def get_runtime_url():
    # Return the AG-UI compatible endpoint URL
    return "http://localhost:8765/api/copilotkit"

# Start AG-UI HTTP server
AIAgent.start_http_server(port=8765)
```

```tsx
// Frontend connects directly to AG-UI endpoint
import { CopilotKit, CopilotSidebar } from '@copilotkit/react-ui';

function App() {
  return (
    <CopilotKit runtimeUrl="http://localhost:8765/api/copilotkit">
      <CopilotSidebar>
        <YourApp />
      </CopilotSidebar>
    </CopilotKit>
  );
}
```

## Implementation Plan

### Phase 1: Fix Immediate Encoding Issue (Short-term) ✅ Done
- [x] Fix JSON escaping in `emit_event.js` template to handle UTF-8 properly
  - Implemented in `crates/auroraview-core/templates/emit_event.js` using Askama's `|safe` filter
  - JSON data is passed directly without manual escaping
- [x] Use `JSON.stringify` instead of manual escaping
  - Added `to_js_literal()` function in `crates/auroraview-core/src/json.rs`
  - All Python→JS event emission uses proper JSON serialization with `ensure_ascii=False`

### Phase 2: CopilotKit Integration (Medium-term) - Future
- [ ] Add `@copilotkit/react-ui` as optional peer dependency
- [ ] Create `@auroraview/sdk/copilotkit` adapter
- [ ] Implement AG-UI to CopilotKit event translation
- [ ] Update Gallery to use CopilotKit components

### Phase 3: HTTP Endpoint (Long-term) - Future
- [ ] Implement AG-UI HTTP server in Python
- [ ] Support Server-Sent Events (SSE) for streaming
- [ ] Add authentication/CORS configuration
- [ ] Document standalone deployment

## Dependencies

```json
{
  "peerDependencies": {
    "@copilotkit/react-core": ">=1.0.0",
    "@copilotkit/react-ui": ">=1.0.0"
  },
  "peerDependenciesMeta": {
    "@copilotkit/react-core": { "optional": true },
    "@copilotkit/react-ui": { "optional": true }
  }
}
```

## Alternatives Considered

### 1. Vercel AI SDK
- Pros: Lightweight, good `useChat` hook
- Cons: No built-in UI components, requires custom implementation

### 2. Continue Custom Implementation
- Pros: Full control
- Cons: Significant maintenance burden, encoding issues

### 3. Langchain Chat UI
- Pros: Good Python integration
- Cons: Less mature React components

## Conclusion

CopilotKit provides the best balance of features, AG-UI compatibility, and maintenance burden. By integrating it as an optional adapter in our SDK, we give developers the choice to use production-ready AI chat UI while keeping AuroraView's core focus on WebView embedding and Python-JS bridging.
