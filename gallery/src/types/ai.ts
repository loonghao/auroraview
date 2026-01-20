/**
 * AI Agent API Type Definitions
 * 
 * Types for the AI Agent sidebar and API integration.
 */

// =============================================================================
// Configuration Types
// =============================================================================

export interface AIConfig {
  model: string;
  temperature: number;
  max_tokens: number;
  provider: string;
  stream: boolean;
}

export interface AIModel {
  id: string;
  name: string;
  provider: string;
  description: string;
  context_window: number;
  supports_vision: boolean;
  supports_tools: boolean;
  available: boolean;
}

export interface AITool {
  name: string;
  description: string;
  parameters?: Record<string, unknown>;
}

// =============================================================================
// Message Types
// =============================================================================

export interface AIMessage {
  id: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  timestamp?: number;
  thinking?: string;
  tool_calls?: AIToolCall[];
}

export interface AIToolCall {
  id: string;
  name: string;
  arguments: string;
  result?: string;
}

export interface AISession {
  id: string;
  messages: AIMessage[];
  system_prompt?: string;
}

// =============================================================================
// API Response Types
// =============================================================================

export interface AIChatResponse {
  status: 'ok' | 'error' | 'streaming';
  response?: string;
  message?: string;
}

export interface AIApiKeysStatus {
  openai: boolean;
  anthropic: boolean;
  gemini: boolean;
  deepseek: boolean;
  groq: boolean;
  ollama: boolean;
}

// =============================================================================
// AG-UI Event Types
// =============================================================================

export type AGUIEventType = 
  | 'run_started'
  | 'run_finished'
  | 'run_error'
  | 'text_message_start'
  | 'text_message_content'
  | 'text_message_end'
  | 'thinking_text_message_start'
  | 'thinking_text_message_content'
  | 'thinking_text_message_end'
  | 'tool_call_start'
  | 'tool_call_args'
  | 'tool_call_end'
  | 'tool_call_result';

export interface AGUIEvent {
  type: string;
  timestamp: number;
  run_id?: string;
  thread_id?: string;
  message_id?: string;
  tool_call_id?: string;
  delta?: string;
  content?: string;
  role?: string;
  tool_name?: string;
  arguments?: string;
  message?: string;
  code?: string;
}

// =============================================================================
// Sidebar State
// =============================================================================

export interface AISidebarState {
  isOpen: boolean;
  isLoading: boolean;
  messages: AIMessage[];
  currentModel: string;
  availableModels: AIModel[];
  tools: AITool[];
  apiKeys: AIApiKeysStatus;
  error?: string;
}

export const DEFAULT_SIDEBAR_STATE: AISidebarState = {
  isOpen: false,
  isLoading: false,
  messages: [],
  currentModel: 'gpt-4o',
  availableModels: [],
  tools: [],
  apiKeys: {
    openai: false,
    anthropic: false,
    gemini: false,
    deepseek: false,
    groq: false,
    ollama: true,
  },
};
