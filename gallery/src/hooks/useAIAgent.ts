/**
 * AI Agent Hook for Gallery
 * 
 * Provides React state management and API calls for the AI Agent sidebar.
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { useAuroraView as useAuroraViewBase } from '@auroraview/sdk/react';
import type {
  AIConfig,
  AIModel,
  AITool,
  AIMessage,
  AIChatResponse,
  AIApiKeysStatus,
  AGUIEvent,
} from '../types/ai';

// Generate unique message ID
function generateMessageId(): string {
  return `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

export function useAIAgent() {
  const { client, isReady } = useAuroraViewBase();
  
  // State
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [messages, setMessages] = useState<AIMessage[]>([]);
  const [currentModel, setCurrentModel] = useState('gpt-4o');
  const [availableModels, setAvailableModels] = useState<AIModel[]>([]);
  const [tools, setTools] = useState<AITool[]>([]);
  const [apiKeys, setApiKeys] = useState<AIApiKeysStatus>({
    openai: false,
    anthropic: false,
    gemini: false,
    deepseek: false,
    groq: false,
    ollama: true,
  });
  const [error, setError] = useState<string | undefined>();
  const [streamingContent, setStreamingContent] = useState<string>('');
  const [isStreaming, setIsStreaming] = useState(false);
  
  // Ref for current streaming message
  const streamingMessageIdRef = useRef<string | null>(null);

  // Load initial data when ready
  useEffect(() => {
    if (!isReady || !client) return;

    const loadInitialData = async () => {
      try {
        // Load models
        const models = await client.call<AIModel[]>('ai.get_models');
        setAvailableModels(models);

        // Load API key status
        const keys = await client.call<AIApiKeysStatus>('ai.get_api_keys');
        setApiKeys(keys);

        // Load tools
        const toolsList = await client.call<AITool[]>('ai.get_tools');
        setTools(toolsList);

        // Load config to get current model
        const config = await client.call<AIConfig>('ai.get_config');
        setCurrentModel(config.model);
      } catch (err) {
        console.error('[AI Agent] Failed to load initial data:', err);
      }
    };

    loadInitialData();
  }, [isReady, client]);

  // Ref to track streaming content for use in event handlers
  const streamingContentRef = useRef<string>('');
  
  // Update ref when streamingContent changes
  useEffect(() => {
    streamingContentRef.current = streamingContent;
  }, [streamingContent]);

  // Subscribe to AG-UI events
  useEffect(() => {
    if (!isReady || !client) return;

    const unsubscribers: (() => void)[] = [];

    // Text streaming events
    const handleTextStart = (event: AGUIEvent) => {
      console.log('[AI Agent] Text stream started:', event.message_id);
      setIsStreaming(true);
      setIsLoading(false); // Stop showing "Thinking..."
      const msgId = event.message_id || generateMessageId();
      streamingMessageIdRef.current = msgId;
      streamingContentRef.current = '';
      setStreamingContent('');
      
      // Add placeholder assistant message
      const assistantMessage: AIMessage = {
        id: msgId,
        role: 'assistant',
        content: '',
        timestamp: Date.now(),
      };
      setMessages(prev => [...prev, assistantMessage]);
    };

    const handleTextDelta = (event: AGUIEvent) => {
      if (event.delta) {
        streamingContentRef.current += event.delta;
        setStreamingContent(streamingContentRef.current);
        
        // Update the message content in real-time
        if (streamingMessageIdRef.current) {
          const currentContent = streamingContentRef.current;
          setMessages(prev => prev.map(m =>
            m.id === streamingMessageIdRef.current
              ? { ...m, content: currentContent }
              : m
          ));
        }
      }
    };

    const handleTextEnd = (_event: AGUIEvent) => {
      console.log('[AI Agent] Text stream ended');
      setIsStreaming(false);
      setIsLoading(false);
      
      // Final update with complete content
      if (streamingMessageIdRef.current) {
        const finalContent = streamingContentRef.current;
        setMessages(prev => prev.map(m =>
          m.id === streamingMessageIdRef.current
            ? { ...m, content: finalContent }
            : m
        ));
      }
      
      setStreamingContent('');
      streamingContentRef.current = '';
      streamingMessageIdRef.current = null;
    };

    // Thinking events (for reasoning models)
    const handleThinkingDelta = (event: AGUIEvent) => {
      console.log('[AI Agent] Thinking:', event.delta);
    };

    // Error events
    const handleRunError = (event: AGUIEvent) => {
      console.error('[AI Agent] Run error:', event.message);
      setError(event.message);
      setIsLoading(false);
      setIsStreaming(false);
    };

    // Subscribe to events
    console.log('[AI Agent] Subscribing to AG-UI events...');
    try {
      unsubscribers.push(client.on('agui:text_message_start', handleTextStart));
      unsubscribers.push(client.on('agui:text_message_content', handleTextDelta));
      unsubscribers.push(client.on('agui:text_message_end', handleTextEnd));
      unsubscribers.push(client.on('agui:thinking_text_message_content', handleThinkingDelta));
      unsubscribers.push(client.on('agui:run_error', handleRunError));
      console.log('[AI Agent] Subscribed to', unsubscribers.length, 'events');
    } catch (err) {
      console.error('[AI Agent] Failed to subscribe to events:', err);
    }

    return () => {
      console.log('[AI Agent] Unsubscribing from AG-UI events...');
      unsubscribers.forEach(unsub => unsub());
    };
  }, [isReady, client]); // Remove streamingContent from dependencies!

  // Toggle sidebar
  const toggleSidebar = useCallback(() => {
    setIsOpen(prev => !prev);
  }, []);

  // Send message
  const sendMessage = useCallback(async (content: string) => {
    if (!client || !content.trim()) return;

    // Add user message
    const userMessage: AIMessage = {
      id: generateMessageId(),
      role: 'user',
      content: content.trim(),
      timestamp: Date.now(),
    };
    setMessages(prev => [...prev, userMessage]);
    setError(undefined);
    setIsLoading(true);

    try {
      // Use streaming endpoint
      const response = await client.call<AIChatResponse>('ai.chat_stream', {
        message: content.trim(),
      });

      if (response.status === 'error') {
        setError(response.message);
        setIsLoading(false);
      }
      // For streaming, the response comes via events
      // For non-streaming, add the response directly
      if (response.status === 'ok' && response.response) {
        const assistantMessage: AIMessage = {
          id: generateMessageId(),
          role: 'assistant',
          content: response.response,
          timestamp: Date.now(),
        };
        setMessages(prev => [...prev, assistantMessage]);
        setIsLoading(false);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      setIsLoading(false);
    }
  }, [client]);

  // Send message (non-streaming)
  const sendMessageSync = useCallback(async (content: string) => {
    if (!client || !content.trim()) return;

    // Add user message
    const userMessage: AIMessage = {
      id: generateMessageId(),
      role: 'user',
      content: content.trim(),
      timestamp: Date.now(),
    };
    setMessages(prev => [...prev, userMessage]);
    setError(undefined);
    setIsLoading(true);

    try {
      const response = await client.call<AIChatResponse>('ai.chat', {
        message: content.trim(),
      });

      if (response.status === 'ok' && response.response) {
        const assistantMessage: AIMessage = {
          id: generateMessageId(),
          role: 'assistant',
          content: response.response,
          timestamp: Date.now(),
        };
        setMessages(prev => [...prev, assistantMessage]);
      } else if (response.status === 'error') {
        setError(response.message);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setIsLoading(false);
    }
  }, [client]);

  // Change model
  const changeModel = useCallback(async (modelId: string) => {
    if (!client) return;

    try {
      await client.call('ai.set_config', { model: modelId });
      setCurrentModel(modelId);
    } catch (err) {
      console.error('[AI Agent] Failed to change model:', err);
    }
  }, [client]);

  // Clear session
  const clearSession = useCallback(async () => {
    if (!client) return;

    try {
      await client.call('ai.clear_session');
      setMessages([]);
      setError(undefined);
    } catch (err) {
      console.error('[AI Agent] Failed to clear session:', err);
    }
  }, [client]);

  // Refresh tools
  const refreshTools = useCallback(async () => {
    if (!client) return;

    try {
      await client.call('ai.discover_tools');
      const toolsList = await client.call<AITool[]>('ai.get_tools');
      setTools(toolsList);
    } catch (err) {
      console.error('[AI Agent] Failed to refresh tools:', err);
    }
  }, [client]);

  return {
    // State
    isOpen,
    isLoading,
    messages,
    currentModel,
    availableModels,
    tools,
    apiKeys,
    error,
    isStreaming,
    streamingContent,
    isReady,

    // Actions
    toggleSidebar,
    setIsOpen,
    sendMessage,
    sendMessageSync,
    changeModel,
    clearSession,
    refreshTools,
  };
}
