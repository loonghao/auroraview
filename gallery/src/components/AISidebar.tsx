/**
 * AI Agent Sidebar Component
 * 
 * A collapsible sidebar for interacting with the AI assistant.
 * Features:
 * - Chat interface with streaming responses
 * - Model selection
 * - Tool discovery display
 * - Keyboard shortcut toggle (Ctrl+Shift+A)
 */

import { useState, useRef, useEffect, type KeyboardEvent } from 'react';
import {
  Bot,
  X,
  Send,
  Loader2,
  Trash2,
  ChevronDown,
  Wrench,
  Sparkles,
  Brain,
  AlertCircle,
} from 'lucide-react';
import { useAIAgent } from '../hooks/useAIAgent';
import type { AIMessage, AIModel } from '../types/ai';

// Provider icons/colors
const PROVIDER_COLORS: Record<string, string> = {
  openai: '#10a37f',
  anthropic: '#d97706',
  gemini: '#4285f4',
  deepseek: '#0ea5e9',
  ollama: '#9333ea',
  groq: '#ef4444',
};

interface AISidebarProps {
  className?: string;
}

export function AISidebar({ className = '' }: AISidebarProps) {
  const {
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
    toggleSidebar,
    setIsOpen,
    sendMessage,
    changeModel,
    clearSession,
    refreshTools,
  } = useAIAgent();

  const [inputValue, setInputValue] = useState('');
  const [showModelSelect, setShowModelSelect] = useState(false);
  const [showTools, setShowTools] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingContent]);

  // Keyboard shortcut: Ctrl+Shift+A
  useEffect(() => {
    const handleKeyDown = (e: globalThis.KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === 'A') {
        e.preventDefault();
        toggleSidebar();
      }
      // Escape to close
      if (e.key === 'Escape' && isOpen) {
        setIsOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [toggleSidebar, setIsOpen, isOpen]);

  // Focus input when opened
  useEffect(() => {
    if (isOpen) {
      inputRef.current?.focus();
    }
  }, [isOpen]);

  const handleSubmit = () => {
    if (inputValue.trim() && !isLoading) {
      sendMessage(inputValue.trim());
      setInputValue('');
    }
  };

  const handleKeyPress = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const getCurrentModelInfo = (): AIModel | undefined => {
    return availableModels.find(m => m.id === currentModel);
  };

  const modelInfo = getCurrentModelInfo();

  // Group models by provider
  const modelsByProvider = availableModels.reduce((acc, model) => {
    if (!acc[model.provider]) {
      acc[model.provider] = [];
    }
    acc[model.provider].push(model);
    return acc;
  }, {} as Record<string, AIModel[]>);

  return (
    <>
      {/* Toggle Button */}
      <button
        onClick={toggleSidebar}
        className={`fixed bottom-4 right-4 z-50 p-3 rounded-full shadow-lg transition-all duration-200 ${
          isOpen
            ? 'bg-gray-700 text-white'
            : 'bg-gradient-to-r from-purple-600 to-blue-600 text-white hover:shadow-xl hover:scale-105'
        } ${!isReady ? 'opacity-50' : ''}`}
        title={isReady ? "Toggle AI Assistant (Ctrl+Shift+A)" : "AI Assistant (Loading...)"}
      >
        {isOpen ? <X size={24} /> : <Bot size={24} />}
      </button>

      {/* Sidebar */}
      <div
        className={`fixed top-0 right-0 h-full bg-gray-900 border-l border-gray-700 shadow-2xl transition-transform duration-300 ease-in-out z-40 flex flex-col ${
          isOpen ? 'translate-x-0' : 'translate-x-full'
        } ${className}`}
        style={{ width: '380px' }}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-gray-700 bg-gray-800/50">
          <div className="flex items-center gap-2">
            <div className="p-1.5 rounded-lg bg-gradient-to-r from-purple-600 to-blue-600">
              <Sparkles size={18} className="text-white" />
            </div>
            <span className="font-semibold text-white">AI Assistant</span>
          </div>
          <div className="flex items-center gap-1">
            <button
              onClick={() => setShowTools(!showTools)}
              className="p-2 rounded-lg hover:bg-gray-700 text-gray-400 hover:text-white transition-colors"
              title="View Tools"
            >
              <Wrench size={18} />
            </button>
            <button
              onClick={clearSession}
              className="p-2 rounded-lg hover:bg-gray-700 text-gray-400 hover:text-white transition-colors"
              title="Clear Chat"
            >
              <Trash2 size={18} />
            </button>
            <button
              onClick={() => setIsOpen(false)}
              className="p-2 rounded-lg hover:bg-gray-700 text-gray-400 hover:text-white transition-colors"
            >
              <X size={18} />
            </button>
          </div>
        </div>

        {/* Model Selector */}
        <div className="px-4 py-2 border-b border-gray-700 bg-gray-800/30">
          <button
            onClick={() => setShowModelSelect(!showModelSelect)}
            className="w-full flex items-center justify-between p-2 rounded-lg hover:bg-gray-700/50 transition-colors"
          >
            <div className="flex items-center gap-2">
              <div
                className="w-2 h-2 rounded-full"
                style={{
                  backgroundColor: modelInfo
                    ? PROVIDER_COLORS[modelInfo.provider] || '#666'
                    : '#666',
                }}
              />
              <span className="text-sm text-gray-300">
                {modelInfo?.name || currentModel}
              </span>
              {modelInfo?.supports_vision && (
                <span className="text-xs px-1.5 py-0.5 rounded bg-blue-600/20 text-blue-400">
                  Vision
                </span>
              )}
            </div>
            <ChevronDown
              size={16}
              className={`text-gray-500 transition-transform ${
                showModelSelect ? 'rotate-180' : ''
              }`}
            />
          </button>

          {/* Model Dropdown */}
          {showModelSelect && (
            <div className="mt-2 max-h-64 overflow-y-auto rounded-lg border border-gray-700 bg-gray-800">
              {Object.entries(modelsByProvider).map(([provider, models]) => (
                <div key={provider}>
                  <div className="px-3 py-1.5 text-xs font-medium text-gray-500 uppercase tracking-wider bg-gray-900/50">
                    {provider}
                    {!apiKeys[provider as keyof typeof apiKeys] && (
                      <span className="ml-2 text-yellow-500">(No API Key)</span>
                    )}
                  </div>
                  {models.map(model => (
                    <button
                      key={model.id}
                      onClick={() => {
                        changeModel(model.id);
                        setShowModelSelect(false);
                      }}
                      disabled={!model.available}
                      className={`w-full px-3 py-2 text-left text-sm hover:bg-gray-700/50 transition-colors ${
                        model.id === currentModel
                          ? 'bg-gray-700/50 text-white'
                          : model.available
                          ? 'text-gray-300'
                          : 'text-gray-600 cursor-not-allowed'
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <span>{model.name}</span>
                        {model.supports_vision && (
                          <span className="text-xs text-blue-400">👁</span>
                        )}
                      </div>
                      {model.description && (
                        <p className="text-xs text-gray-500 mt-0.5">
                          {model.description}
                        </p>
                      )}
                    </button>
                  ))}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Tools Panel (Collapsible) */}
        {showTools && (
          <div className="px-4 py-2 border-b border-gray-700 bg-gray-800/30 max-h-48 overflow-y-auto">
            <div className="flex items-center justify-between mb-2">
              <span className="text-xs font-medium text-gray-500 uppercase tracking-wider">
                Available Tools ({tools.length})
              </span>
              <button
                onClick={refreshTools}
                className="text-xs text-blue-400 hover:text-blue-300"
              >
                Refresh
              </button>
            </div>
            {tools.length === 0 ? (
              <p className="text-xs text-gray-500 italic">No tools discovered</p>
            ) : (
              <div className="space-y-1">
                {tools.map(tool => (
                  <div
                    key={tool.name}
                    className="p-2 rounded-lg bg-gray-800/50 border border-gray-700/50"
                  >
                    <p className="text-sm text-gray-300 font-mono">
                      {tool.name}
                    </p>
                    {tool.description && (
                      <p className="text-xs text-gray-500 mt-0.5">
                        {tool.description}
                      </p>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Messages */}
        <div className="flex-1 overflow-y-auto px-4 py-3 space-y-4">
          {!isReady && (
            <div className="flex flex-col items-center justify-center h-full text-center">
              <Loader2 size={48} className="text-gray-600 mb-4 animate-spin" />
              <p className="text-gray-500 mb-2">Connecting to AI Agent...</p>
              <p className="text-xs text-gray-600 max-w-xs">
                Please wait while the bridge initializes.
              </p>
            </div>
          )}
          {isReady && messages.length === 0 && !isStreaming && (
            <div className="flex flex-col items-center justify-center h-full text-center">
              <Brain size={48} className="text-gray-600 mb-4" />
              <p className="text-gray-500 mb-2">How can I help you?</p>
              <p className="text-xs text-gray-600 max-w-xs">
                I can help you explore examples, explain code, and assist with
                AuroraView development.
              </p>
            </div>
          )}

          {messages.map(message => (
            <MessageBubble key={message.id} message={message} />
          ))}

          {/* Streaming message */}
          {isStreaming && streamingContent && (
            <MessageBubble
              message={{
                id: 'streaming',
                role: 'assistant',
                content: streamingContent,
              }}
              isStreaming
            />
          )}

          {/* Loading indicator */}
          {isLoading && !isStreaming && (
            <div className="flex items-center gap-2 text-gray-500">
              <Loader2 size={16} className="animate-spin" />
              <span className="text-sm">Thinking...</span>
            </div>
          )}

          {/* Error message */}
          {error && (
            <div className="flex items-start gap-2 p-3 rounded-lg bg-red-900/20 border border-red-800/50 text-red-400">
              <AlertCircle size={18} className="flex-shrink-0 mt-0.5" />
              <div>
                <p className="text-sm font-medium">Error</p>
                <p className="text-xs mt-1 opacity-80">{error}</p>
              </div>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        {/* Input */}
        <div className="px-4 py-3 border-t border-gray-700 bg-gray-800/50">
          <div className="flex items-end gap-2">
            <textarea
              ref={inputRef}
              value={inputValue}
              onChange={e => setInputValue(e.target.value)}
              onKeyDown={handleKeyPress}
              placeholder={isReady ? "Ask me anything..." : "Connecting..."}
              className="flex-1 px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white placeholder-gray-500 resize-none focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent text-sm"
              rows={2}
              disabled={isLoading || !isReady}
            />
            <button
              onClick={handleSubmit}
              disabled={!inputValue.trim() || isLoading || !isReady}
              className="p-2 rounded-lg bg-gradient-to-r from-purple-600 to-blue-600 text-white disabled:opacity-50 disabled:cursor-not-allowed hover:opacity-90 transition-opacity"
            >
              {isLoading ? (
                <Loader2 size={20} className="animate-spin" />
              ) : (
                <Send size={20} />
              )}
            </button>
          </div>
          <p className="text-xs text-gray-600 mt-2 text-center">
            Press Enter to send, Shift+Enter for new line
          </p>
        </div>
      </div>

      {/* Backdrop */}
      {isOpen && (
        <div
          className="fixed inset-0 bg-black/20 backdrop-blur-sm z-30"
          onClick={() => setIsOpen(false)}
        />
      )}
    </>
  );
}

// Message Bubble Component
interface MessageBubbleProps {
  message: AIMessage;
  isStreaming?: boolean;
}

function MessageBubble({ message, isStreaming = false }: MessageBubbleProps) {
  const isUser = message.role === 'user';

  return (
    <div
      className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}
    >
      <div
        className={`max-w-[85%] rounded-lg px-3 py-2 ${
          isUser
            ? 'bg-gradient-to-r from-purple-600 to-blue-600 text-white'
            : 'bg-gray-800 text-gray-200 border border-gray-700'
        }`}
      >
        <p className="text-sm whitespace-pre-wrap">{message.content}</p>
        {isStreaming && (
          <span className="inline-block w-1.5 h-4 ml-1 bg-current animate-pulse" />
        )}
      </div>
    </div>
  );
}

export default AISidebar;
