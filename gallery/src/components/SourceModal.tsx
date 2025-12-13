import { useEffect, useRef, useState } from 'react';
import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';
import hljs from 'highlight.js/lib/core';
import python from 'highlight.js/lib/languages/python';

// Register Python language
hljs.registerLanguage('python', python);

interface SourceModalProps {
  isOpen: boolean;
  title: string;
  source: string;
  onClose: () => void;
  onCopy: () => void;
  onRun: () => void;
}

export function SourceModal({ isOpen, title, source, onClose, onCopy, onRun }: SourceModalProps) {
  const codeRef = useRef<HTMLElement>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (isOpen && codeRef.current && source) {
      // Reset any previous highlighting
      codeRef.current.removeAttribute('data-highlighted');
      codeRef.current.className = 'language-python';
      codeRef.current.textContent = source;
      hljs.highlightElement(codeRef.current);
    }
  }, [isOpen, source]);

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    if (isOpen) {
      document.addEventListener('keydown', handleEscape);
      return () => document.removeEventListener('keydown', handleEscape);
    }
  }, [isOpen, onClose]);

  const handleCopy = () => {
    onCopy();
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (!isOpen) return null;

  // Split source into lines for line numbers
  const lines = source.split('\n');

  return (
    <div
      className={cn(
        "fixed inset-0 bg-black/60 z-50 flex items-center justify-center backdrop-blur-sm",
        isOpen ? "flex" : "hidden"
      )}
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div className="bg-[#1e1e1e] border border-[#3c3c3c] rounded-xl w-[90%] max-w-5xl max-h-[85vh] flex flex-col shadow-2xl">
        {/* Header - IDE style title bar */}
        <div className="flex items-center justify-between px-4 py-3 bg-[#252526] border-b border-[#3c3c3c] rounded-t-xl">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-1.5">
              <div className="w-3 h-3 rounded-full bg-[#ff5f57]" />
              <div className="w-3 h-3 rounded-full bg-[#febc2e]" />
              <div className="w-3 h-3 rounded-full bg-[#28c840]" />
            </div>
            <div className="flex items-center gap-2 text-[#cccccc]">
              <Icons.FileCode className="w-4 h-4 text-[#519aba]" />
              <span className="text-sm font-medium">{title}</span>
            </div>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 rounded-lg flex items-center justify-center text-[#858585] hover:bg-[#3c3c3c] hover:text-[#cccccc] transition-all"
          >
            <Icons.X className="w-4 h-4" />
          </button>
        </div>

        {/* Code content with line numbers */}
        <div className="flex-1 overflow-auto bg-[#1e1e1e]">
          <div className="flex min-h-full">
            {/* Line numbers */}
            <div className="flex-shrink-0 py-4 px-3 text-right select-none bg-[#1e1e1e] border-r border-[#3c3c3c]">
              {lines.map((_, i) => (
                <div key={i} className="text-xs leading-6 text-[#858585] font-mono">
                  {i + 1}
                </div>
              ))}
            </div>
            {/* Code */}
            <pre className="flex-1 m-0 p-4 overflow-x-auto">
              <code ref={codeRef} className="language-python text-sm leading-6 font-mono">
                {source}
              </code>
            </pre>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-4 py-3 bg-[#252526] border-t border-[#3c3c3c] rounded-b-xl">
          <div className="text-xs text-[#858585]">
            {lines.length} lines • Python
          </div>
          <div className="flex gap-2">
            <button
              onClick={handleCopy}
              className={cn(
                "px-4 py-2 rounded-lg text-sm font-medium transition-all",
                "flex items-center gap-2 bg-[#3c3c3c] border border-[#4c4c4c] text-[#cccccc]",
                "hover:bg-[#4c4c4c]"
              )}
            >
              {copied ? (
                <>
                  <Icons.Check className="w-4 h-4 text-green-400" /> Copied!
                </>
              ) : (
                <>
                  <Icons.Copy className="w-4 h-4" /> Copy
                </>
              )}
            </button>
            <button
              onClick={onRun}
              className={cn(
                "px-4 py-2 rounded-lg text-sm font-medium transition-all",
                "flex items-center gap-2 bg-[#0e639c] text-white",
                "hover:bg-[#1177bb]"
              )}
            >
              <Icons.Play className="w-4 h-4 fill-current" /> Run Demo
            </button>
          </div>
        </div>
      </div>

      {/* VS Code Dark+ theme styles */}
      <style>{`
        .hljs {
          background: transparent !important;
          color: #d4d4d4 !important;
        }
        .hljs-keyword {
          color: #569cd6 !important;
        }
        .hljs-built_in {
          color: #4ec9b0 !important;
        }
        .hljs-type {
          color: #4ec9b0 !important;
        }
        .hljs-literal {
          color: #569cd6 !important;
        }
        .hljs-number {
          color: #b5cea8 !important;
        }
        .hljs-string {
          color: #ce9178 !important;
        }
        .hljs-comment {
          color: #6a9955 !important;
          font-style: italic;
        }
        .hljs-doctag {
          color: #608b4e !important;
        }
        .hljs-function {
          color: #dcdcaa !important;
        }
        .hljs-title {
          color: #dcdcaa !important;
        }
        .hljs-title.function_ {
          color: #dcdcaa !important;
        }
        .hljs-params {
          color: #9cdcfe !important;
        }
        .hljs-variable {
          color: #9cdcfe !important;
        }
        .hljs-class .hljs-title {
          color: #4ec9b0 !important;
        }
        .hljs-attr {
          color: #9cdcfe !important;
        }
        .hljs-meta {
          color: #c586c0 !important;
        }
        .hljs-decorator {
          color: #dcdcaa !important;
        }
        .hljs-symbol {
          color: #d4d4d4 !important;
        }
        .hljs-operator {
          color: #d4d4d4 !important;
        }
        .hljs-punctuation {
          color: #d4d4d4 !important;
        }
      `}</style>
    </div>
  );
}
