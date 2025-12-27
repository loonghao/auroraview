import { useState, useEffect, useRef, useCallback, type ReactNode } from 'react';
import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

interface SplitLayoutProps {
  children: ReactNode;
  sidePanel?: {
    title: string;
    icon?: ReactNode;
    content: ReactNode;
    onClose: () => void;
    onReload?: () => void;
  } | null;
  defaultSidePanelWidth?: number;
  minSidePanelWidth?: number;
  maxSidePanelWidth?: number;
}

/**
 * Split Layout Component
 * 
 * Implements a Chrome-like split view with main content and resizable side panel.
 * The side panel slides in from the right and can be resized by dragging.
 */
export function SplitLayout({
  children,
  sidePanel,
  defaultSidePanelWidth = 400,
  minSidePanelWidth = 280,
  maxSidePanelWidth = 800,
}: SplitLayoutProps) {
  const [sidePanelWidth, setSidePanelWidth] = useState(defaultSidePanelWidth);
  const [isResizing, setIsResizing] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // Handle resize start
  const handleResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  }, []);

  // Handle resize
  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (!containerRef.current) return;
      const containerRect = containerRef.current.getBoundingClientRect();
      const newWidth = containerRect.right - e.clientX;
      setSidePanelWidth(Math.max(minSidePanelWidth, Math.min(maxSidePanelWidth, newWidth)));
    };

    const handleMouseUp = () => {
      setIsResizing(false);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = 'ew-resize';
    document.body.style.userSelect = 'none';

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };
  }, [isResizing, minSidePanelWidth, maxSidePanelWidth]);

  const isOpen = !!sidePanel;

  return (
    <div ref={containerRef} className="flex h-full w-full overflow-hidden">
      {/* Main Content Area */}
      <div
        className={cn(
          "flex-1 min-w-0 transition-all duration-300 ease-in-out overflow-auto",
          isOpen && "mr-0"
        )}
        style={{
          marginRight: isOpen ? 0 : 0,
        }}
      >
        {children}
      </div>

      {/* Side Panel */}
      {isOpen && (
        <div
          className={cn(
            "relative flex flex-col bg-background border-l border-border",
            "transition-all duration-300 ease-in-out",
            isResizing && "transition-none"
          )}
          style={{ width: sidePanelWidth, minWidth: sidePanelWidth }}
        >
          {/* Resize Handle */}
          <div
            className={cn(
              "absolute left-0 top-0 bottom-0 w-1.5 cursor-ew-resize z-10",
              "hover:bg-primary/30 active:bg-primary/50 transition-colors",
              isResizing && "bg-primary/50"
            )}
            onMouseDown={handleResizeStart}
          >
            {/* Visual indicator */}
            <div className="absolute left-0.5 top-1/2 -translate-y-1/2 w-0.5 h-8 bg-border rounded-full opacity-0 hover:opacity-100 transition-opacity" />
          </div>

          {/* Header */}
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-border bg-muted/30 flex-shrink-0">
            <div className="flex items-center gap-2 min-w-0">
              {sidePanel.icon || <Icons.PanelRight className="w-4 h-4 text-primary flex-shrink-0" />}
              <span className="font-medium text-sm truncate">{sidePanel.title}</span>
            </div>
            <div className="flex items-center gap-0.5">
              {sidePanel.onReload && (
                <button
                  onClick={sidePanel.onReload}
                  className="p-1.5 rounded-md hover:bg-muted transition-colors"
                  title="Reload"
                >
                  <Icons.RefreshCw className="w-3.5 h-3.5 text-muted-foreground" />
                </button>
              )}
              <button
                onClick={sidePanel.onClose}
                className="p-1.5 rounded-md hover:bg-muted transition-colors"
                title="Close panel"
              >
                <Icons.X className="w-3.5 h-3.5 text-muted-foreground" />
              </button>
            </div>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-auto">
            {sidePanel.content}
          </div>
        </div>
      )}
    </div>
  );
}
