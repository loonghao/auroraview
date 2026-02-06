/**
 * AI Agent Sidebar Component
 *
 * A React component that provides an AI-powered sidebar for natural language
 * interaction with AuroraView applications.
 *
 * Features:
 * - Streaming responses with AG-UI protocol
 * - Tool execution visualization
 * - Thinking/reasoning display (for models like DeepSeek R1)
 * - Keyboard shortcuts
 * - Responsive design
 *
 * @example
 * ```tsx
 * import { AgentSidebar } from '@auroraview/sdk/components';
 *
 * function App() {
 *   return (
 *     <div className="app">
 *       <MainContent />
 *       <AgentSidebar position="right" width={400} />
 *     </div>
 *   );
 * }
 * ```
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import { useAuroraView, useAuroraEvent } from '../adapters/react';

// ============================================
// Types
// ============================================

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  timestamp: Date;
  toolCalls?: ToolCallInfo[];
  thinking?: string;
  isStreaming?: boolean;
}

export interface ToolCallInfo {
  id: string;
  name: string;
  args?: string;
  result?: string;
  status: 'pending' | 'executing' | 'completed' | 'error';
}

export interface AgentSidebarProps {
  /** Sidebar position */
  position?: 'left' | 'right';
  /** Sidebar width in pixels */
  width?: number;
  /** Initially open */
  defaultOpen?: boolean;
  /** Keyboard shortcut to toggle (e.g., "Ctrl+Shift+A") */
  shortcut?: string;
  /** Header title */
  title?: string;
  /** Placeholder text for input */
  placeholder?: string;
  /** Custom class name */
  className?: string;
  /** Callback when sidebar opens/closes */
  onToggle?: (isOpen: boolean) => void;
}

// ============================================
// Component
// ============================================

export function AgentSidebar({
  position = 'right',
  width = 400,
  defaultOpen = false,
  shortcut = 'Ctrl+Shift+A',
  title = 'AI Assistant',
  placeholder = 'Ask me anything...',
  className = '',
  onToggle,
}: AgentSidebarProps) {
  const { client } = useAuroraView();
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [currentThinking, setCurrentThinking] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Generate unique ID
  const generateId = () => `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

  // Scroll to bottom
  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, []);

  // Toggle sidebar
  const toggleSidebar = useCallback(() => {
    setIsOpen((prev) => {
      const newState = !prev;
      onToggle?.(newState);
      return newState;
    });
  }, [onToggle]);

  // Send message
  const sendMessage = useCallback(async () => {
    const text = inputValue.trim();
    if (!text || isLoading) return;

    // Add user message
    const userMessage: Message = {
      id: generateId(),
      role: 'user',
      content: text,
      timestamp: new Date(),
    };
    setMessages((prev) => [...prev, userMessage]);
    setInputValue('');
    setIsLoading(true);

    // Add placeholder for assistant response
    const assistantMessage: Message = {
      id: generateId(),
      role: 'assistant',
      content: '',
      timestamp: new Date(),
      isStreaming: true,
    };
    setMessages((prev) => [...prev, assistantMessage]);

    try {
      await client.call('ai.chat', { message: text });
    } catch (error) {
      console.error('AI chat error:', error);
      setMessages((prev) => {
        const updated = [...prev];
        const lastMsg = updated[updated.length - 1];
        if (lastMsg.role === 'assistant') {
          lastMsg.content = `Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
          lastMsg.isStreaming = false;
        }
        return updated;
      });
    } finally {
      setIsLoading(false);
    }
  }, [av, inputValue, isLoading]);

  // Handle keyboard shortcut
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const parts = shortcut.toLowerCase().split('+');
      const key = parts.pop();
      const ctrl = parts.includes('ctrl');
      const shift = parts.includes('shift');
      const alt = parts.includes('alt');

      if (
        e.key.toLowerCase() === key &&
        e.ctrlKey === ctrl &&
        e.shiftKey === shift &&
        e.altKey === alt
      ) {
        e.preventDefault();
        toggleSidebar();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [shortcut, toggleSidebar]);

  // ============================================
  // AG-UI Event Handlers
  // ============================================

  // Handle streaming text
  useAuroraEvent('agui:text_delta', (data: { message_id: string; delta: string }) => {
    setMessages((prev) => {
      const updated = [...prev];
      const lastMsg = updated[updated.length - 1];
      if (lastMsg?.role === 'assistant' && lastMsg.isStreaming) {
        lastMsg.content += data.delta;
      }
      return updated;
    });
    scrollToBottom();
  });

  // Handle thinking/reasoning (DeepSeek R1, O1)
  useAuroraEvent('agui:thinking_delta', (data: { delta: string }) => {
    setCurrentThinking((prev) => prev + data.delta);
  });

  // Handle tool call start
  useAuroraEvent('agui:tool_call_start', (data: { tool_call_id: string; name: string }) => {
    setMessages((prev) => {
      const updated = [...prev];
      const lastMsg = updated[updated.length - 1];
      if (lastMsg?.role === 'assistant') {
        lastMsg.toolCalls = lastMsg.toolCalls || [];
        lastMsg.toolCalls.push({
          id: data.tool_call_id,
          name: data.name,
          status: 'executing',
        });
      }
      return updated;
    });
  });

  // Handle tool call result
  useAuroraEvent('agui:tool_call_result', (data: { tool_call_id: string; result: string }) => {
    setMessages((prev) => {
      const updated = [...prev];
      const lastMsg = updated[updated.length - 1];
      if (lastMsg?.role === 'assistant' && lastMsg.toolCalls) {
        const toolCall = lastMsg.toolCalls.find((tc) => tc.id === data.tool_call_id);
        if (toolCall) {
          toolCall.result = data.result;
          toolCall.status = 'completed';
        }
      }
      return updated;
    });
  });

  // Handle run finished
  useAuroraEvent('agui:run_finished', () => {
    setMessages((prev) => {
      const updated = [...prev];
      const lastMsg = updated[updated.length - 1];
      if (lastMsg?.role === 'assistant') {
        lastMsg.isStreaming = false;
        if (currentThinking) {
          lastMsg.thinking = currentThinking;
        }
      }
      return updated;
    });
    setCurrentThinking('');
    setIsLoading(false);
  });

  // Handle run error
  useAuroraEvent('agui:run_error', (data: { error: string }) => {
    setMessages((prev) => {
      const updated = [...prev];
      const lastMsg = updated[updated.length - 1];
      if (lastMsg?.role === 'assistant') {
        lastMsg.content += `\n\nError: ${data.error}`;
        lastMsg.isStreaming = false;
      }
      return updated;
    });
    setIsLoading(false);
  });

  // Auto-scroll on new messages
  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  // ============================================
  // Render
  // ============================================

  return (
    <div
      className={`agent-sidebar ${position} ${isOpen ? 'open' : ''} ${className}`}
      style={{ width: isOpen ? width : 0 }}
    >
      {/* Header */}
      <div className="agent-sidebar-header">
        <span className="agent-sidebar-title">{title}</span>
        <button type="button" className="agent-sidebar-close" onClick={toggleSidebar}>
          ×
        </button>
      </div>

      {/* Messages */}
      <div className="agent-sidebar-messages">
        {messages.map((msg) => (
          <MessageBubble key={msg.id} message={msg} />
        ))}
        {currentThinking && (
          <div className="agent-thinking">
            <span className="thinking-label">Thinking...</span>
            <span className="thinking-content">{currentThinking}</span>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="agent-sidebar-input-container">
        <textarea
          ref={inputRef}
          className="agent-sidebar-input"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              sendMessage();
            }
          }}
          placeholder={placeholder}
          disabled={isLoading}
          rows={2}
        />
        <button
          type="button"
          className="agent-sidebar-send"
          onClick={sendMessage}
          disabled={isLoading || !inputValue.trim()}
        >
          {isLoading ? '...' : '→'}
        </button>
      </div>
    </div>
  );
}

// ============================================
// Sub-components
// ============================================

function MessageBubble({ message }: { message: Message }) {
  return (
    <div className={`agent-message ${message.role}`}>
      <div className="message-content">{message.content}</div>
      {message.toolCalls && message.toolCalls.length > 0 && (
        <div className="tool-calls">
          {message.toolCalls.map((tc) => (
            <ToolCallBadge key={tc.id} toolCall={tc} />
          ))}
        </div>
      )}
      {message.thinking && (
        <details className="message-thinking">
          <summary>View reasoning</summary>
          <pre>{message.thinking}</pre>
        </details>
      )}
    </div>
  );
}

function ToolCallBadge({ toolCall }: { toolCall: ToolCallInfo }) {
  const statusIcon = {
    pending: '⏳',
    executing: '⚙️',
    completed: '✅',
    error: '❌',
  }[toolCall.status];

  return (
    <div className={`tool-call-badge ${toolCall.status}`}>
      <span className="tool-icon">{statusIcon}</span>
      <span className="tool-name">{toolCall.name}</span>
      {toolCall.result && (
        <details className="tool-result">
          <summary>Result</summary>
          <pre>{toolCall.result}</pre>
        </details>
      )}
    </div>
  );
}

export default AgentSidebar;

