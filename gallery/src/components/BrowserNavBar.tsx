/**
 * BrowserNavBar - Browser-style navigation bar with URL input
 */

import { useState, useCallback, useRef, type KeyboardEvent } from 'react';
import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

interface BrowserNavBarProps {
  url: string;
  title?: string;
  canGoBack: boolean;
  canGoForward: boolean;
  isLoading: boolean;
  isSecure?: boolean;
  onBack: () => void;
  onForward: () => void;
  onRefresh: () => void;
  onStop: () => void;
  onHome: () => void;
  onUrlSubmit: (url: string) => void;
  onBookmark?: () => void;
  onHistory?: () => void;
  onDownloads?: () => void;
  onSettings?: () => void;
  isBookmarked?: boolean;
  className?: string;
}

export function BrowserNavBar({
  url,
  title: _title,
  canGoBack,
  canGoForward,
  isLoading,
  isSecure,
  onBack,
  onForward,
  onRefresh,
  onStop,
  onHome,
  onUrlSubmit,
  onBookmark,
  onHistory,
  onDownloads,
  onSettings,
  isBookmarked = false,
  className,
}: BrowserNavBarProps) {
  const [inputValue, setInputValue] = useState(url);
  const [isFocused, setIsFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // Sync input with url prop when not focused
  const handleFocus = useCallback(() => {
    setIsFocused(true);
    setInputValue(url);
    // Select all text on focus
    setTimeout(() => inputRef.current?.select(), 0);
  }, [url]);

  const handleBlur = useCallback(() => {
    setIsFocused(false);
    setInputValue(url);
  }, [url]);

  const handleKeyDown = useCallback((e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      const value = inputValue.trim();
      if (value) {
        // Add protocol if missing
        let finalUrl = value;
        if (!value.startsWith('http://') && !value.startsWith('https://') && !value.startsWith('file://')) {
          // Check if it looks like a domain
          if (value.includes('.') && !value.includes(' ')) {
            finalUrl = 'https://' + value;
          } else {
            // Treat as search query
            finalUrl = `https://www.google.com/search?q=${encodeURIComponent(value)}`;
          }
        }
        onUrlSubmit(finalUrl);
        inputRef.current?.blur();
      }
    } else if (e.key === 'Escape') {
      setInputValue(url);
      inputRef.current?.blur();
    }
  }, [inputValue, url, onUrlSubmit]);

  // Display URL without protocol for cleaner look
  const displayUrl = isFocused ? inputValue : (url.replace(/^https?:\/\//, ''));

  return (
    <div className={cn(
      "flex items-center gap-1.5 px-2 py-1.5 bg-background border-b border-border",
      className
    )}>
      {/* Navigation buttons */}
      <div className="flex items-center gap-0.5">
        <button
          onClick={onBack}
          disabled={!canGoBack}
          className={cn(
            "w-8 h-8 rounded-full flex items-center justify-center transition-colors",
            canGoBack 
              ? "hover:bg-muted text-foreground" 
              : "text-muted-foreground/50 cursor-not-allowed"
          )}
          title="Go back"
        >
          <Icons.ArrowLeft className="w-4 h-4" />
        </button>
        <button
          onClick={onForward}
          disabled={!canGoForward}
          className={cn(
            "w-8 h-8 rounded-full flex items-center justify-center transition-colors",
            canGoForward 
              ? "hover:bg-muted text-foreground" 
              : "text-muted-foreground/50 cursor-not-allowed"
          )}
          title="Go forward"
        >
          <Icons.ArrowRight className="w-4 h-4" />
        </button>
        <button
          onClick={isLoading ? onStop : onRefresh}
          className="w-8 h-8 rounded-full flex items-center justify-center hover:bg-muted transition-colors"
          title={isLoading ? "Stop" : "Refresh"}
        >
          {isLoading ? (
            <Icons.X className="w-4 h-4" />
          ) : (
            <Icons.RotateCw className="w-4 h-4" />
          )}
        </button>
        <button
          onClick={onHome}
          className="w-8 h-8 rounded-full flex items-center justify-center hover:bg-muted transition-colors"
          title="Home"
        >
          <Icons.Home className="w-4 h-4" />
        </button>
      </div>

      {/* URL Bar */}
      <div className={cn(
        "flex-1 flex items-center gap-2 px-3 py-1.5 rounded-full border transition-all",
        isFocused 
          ? "border-primary bg-background shadow-sm" 
          : "border-border bg-muted/50 hover:bg-muted"
      )}>
        {/* Security indicator */}
        {!isFocused && (
          <span className="flex-shrink-0">
            {isSecure ? (
              <Icons.Lock className="w-3.5 h-3.5 text-green-600" />
            ) : url.startsWith('http://') ? (
              <Icons.AlertTriangle className="w-3.5 h-3.5 text-yellow-600" />
            ) : (
              <Icons.Globe className="w-3.5 h-3.5 text-muted-foreground" />
            )}
          </span>
        )}
        
        <input
          ref={inputRef}
          type="text"
          value={isFocused ? inputValue : displayUrl}
          onChange={(e) => setInputValue(e.target.value)}
          onFocus={handleFocus}
          onBlur={handleBlur}
          onKeyDown={handleKeyDown}
          className="flex-1 bg-transparent outline-none text-sm"
          placeholder="Search or enter URL"
        />

        {/* Bookmark button inside URL bar */}
        {onBookmark && !isFocused && (
          <button
            onClick={onBookmark}
            className="flex-shrink-0 p-1 hover:bg-background/50 rounded transition-colors"
            title={isBookmarked ? "Remove bookmark" : "Add bookmark"}
          >
            {isBookmarked ? (
              <Icons.Star className="w-4 h-4 text-yellow-500 fill-yellow-500" />
            ) : (
              <Icons.Star className="w-4 h-4 text-muted-foreground" />
            )}
          </button>
        )}
      </div>

      {/* Right side actions */}
      <div className="flex items-center gap-0.5">
        {onHistory && (
          <button
            onClick={onHistory}
            className="w-8 h-8 rounded-full flex items-center justify-center hover:bg-muted transition-colors"
            title="History"
          >
            <Icons.History className="w-4 h-4" />
          </button>
        )}
        {onDownloads && (
          <button
            onClick={onDownloads}
            className="w-8 h-8 rounded-full flex items-center justify-center hover:bg-muted transition-colors"
            title="Downloads"
          >
            <Icons.Download className="w-4 h-4" />
          </button>
        )}
        {onSettings && (
          <button
            onClick={onSettings}
            className="w-8 h-8 rounded-full flex items-center justify-center hover:bg-muted transition-colors"
            title="Settings"
          >
            <Icons.Settings className="w-4 h-4" />
          </button>
        )}
      </div>
    </div>
  );
}
