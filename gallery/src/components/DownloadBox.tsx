/**
 * DownloadBox - A dropdown component to display download history
 *
 * Features:
 * - Shows download history with file name, status, and actions
 * - Supports download_started, download_completed, download_cancelled events
 * - Allows opening downloaded files or showing in folder
 * - Persists download history in localStorage
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import * as Icons from 'lucide-react';
import { cn } from '../lib/utils';

export interface DownloadItem {
  id: string;
  url: string;
  filename: string;
  path?: string;
  status: 'downloading' | 'completed' | 'failed' | 'cancelled';
  startTime: number;
  endTime?: number;
  error?: string;
}

interface DownloadBoxProps {
  isOpen: boolean;
  onClose: () => void;
  downloads: DownloadItem[];
  onClearAll: () => void;
  onRemoveItem: (id: string) => void;
  onOpenFile: (path: string) => void;
  onShowInFolder: (path: string) => void;
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();
  
  if (isToday) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
  return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
}

function getFileIcon(filename: string): React.ReactNode {
  const ext = filename.split('.').pop()?.toLowerCase() || '';
  
  const iconMap: Record<string, React.ReactNode> = {
    pdf: <Icons.FileText className="w-4 h-4 text-red-500" />,
    doc: <Icons.FileText className="w-4 h-4 text-blue-500" />,
    docx: <Icons.FileText className="w-4 h-4 text-blue-500" />,
    xls: <Icons.FileSpreadsheet className="w-4 h-4 text-green-500" />,
    xlsx: <Icons.FileSpreadsheet className="w-4 h-4 text-green-500" />,
    ppt: <Icons.Presentation className="w-4 h-4 text-orange-500" />,
    pptx: <Icons.Presentation className="w-4 h-4 text-orange-500" />,
    zip: <Icons.FileArchive className="w-4 h-4 text-yellow-600" />,
    rar: <Icons.FileArchive className="w-4 h-4 text-yellow-600" />,
    '7z': <Icons.FileArchive className="w-4 h-4 text-yellow-600" />,
    jpg: <Icons.Image className="w-4 h-4 text-purple-500" />,
    jpeg: <Icons.Image className="w-4 h-4 text-purple-500" />,
    png: <Icons.Image className="w-4 h-4 text-purple-500" />,
    gif: <Icons.Image className="w-4 h-4 text-purple-500" />,
    svg: <Icons.Image className="w-4 h-4 text-purple-500" />,
    mp3: <Icons.Music className="w-4 h-4 text-pink-500" />,
    wav: <Icons.Music className="w-4 h-4 text-pink-500" />,
    mp4: <Icons.Video className="w-4 h-4 text-cyan-500" />,
    avi: <Icons.Video className="w-4 h-4 text-cyan-500" />,
    mkv: <Icons.Video className="w-4 h-4 text-cyan-500" />,
    exe: <Icons.Cog className="w-4 h-4 text-gray-500" />,
    msi: <Icons.Cog className="w-4 h-4 text-gray-500" />,
    py: <Icons.FileCode className="w-4 h-4 text-yellow-500" />,
    js: <Icons.FileCode className="w-4 h-4 text-yellow-400" />,
    ts: <Icons.FileCode className="w-4 h-4 text-blue-400" />,
    json: <Icons.FileJson className="w-4 h-4 text-gray-400" />,
  };
  
  return iconMap[ext] || <Icons.File className="w-4 h-4 text-muted-foreground" />;
}

function DownloadItemRow({
  item,
  onRemove,
  onOpenFile,
  onShowInFolder,
}: {
  item: DownloadItem;
  onRemove: () => void;
  onOpenFile: () => void;
  onShowInFolder: () => void;
}) {
  const [isHovered, setIsHovered] = useState(false);

  return (
    <div
      className={cn(
        "flex items-center gap-3 p-3 rounded-lg transition-colors",
        "hover:bg-muted/50 group"
      )}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* File Icon */}
      <div className="flex-shrink-0">
        {item.status === 'downloading' ? (
          <div className="w-4 h-4 border-2 border-primary border-t-transparent rounded-full animate-spin" />
        ) : (
          getFileIcon(item.filename)
        )}
      </div>

      {/* File Info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium truncate" title={item.filename}>
            {item.filename}
          </span>
          {item.status === 'failed' && (
            <Icons.AlertCircle className="w-3 h-3 text-destructive flex-shrink-0" />
          )}
          {item.status === 'cancelled' && (
            <Icons.XCircle className="w-3 h-3 text-muted-foreground flex-shrink-0" />
          )}
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span>{formatTime(item.startTime)}</span>
          {item.status === 'downloading' && (
            <span className="text-primary">Downloading...</span>
          )}
          {item.status === 'failed' && (
            <span className="text-destructive">{item.error || 'Failed'}</span>
          )}
          {item.status === 'cancelled' && (
            <span>Cancelled</span>
          )}
        </div>
      </div>

      {/* Actions */}
      <div className={cn(
        "flex items-center gap-1 flex-shrink-0 transition-opacity",
        isHovered ? "opacity-100" : "opacity-0"
      )}>
        {item.status === 'completed' && item.path && (
          <>
            <button
              onClick={onOpenFile}
              className="p-1.5 rounded-md hover:bg-muted transition-colors"
              title="Open file"
            >
              <Icons.ExternalLink className="w-3.5 h-3.5" />
            </button>
            <button
              onClick={onShowInFolder}
              className="p-1.5 rounded-md hover:bg-muted transition-colors"
              title="Show in folder"
            >
              <Icons.FolderOpen className="w-3.5 h-3.5" />
            </button>
          </>
        )}
        <button
          onClick={onRemove}
          className="p-1.5 rounded-md hover:bg-destructive/10 hover:text-destructive transition-colors"
          title="Remove from list"
        >
          <Icons.X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}

export function DownloadBox({
  isOpen,
  onClose,
  downloads,
  onClearAll,
  onRemoveItem,
  onOpenFile,
  onShowInFolder,
}: DownloadBoxProps) {
  const boxRef = useRef<HTMLDivElement>(null);

  // Close on click outside
  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (boxRef.current && !boxRef.current.contains(e.target as Node)) {
        onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen, onClose]);

  // Close on Escape
  useEffect(() => {
    if (!isOpen) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const activeDownloads = downloads.filter(d => d.status === 'downloading');
  const completedDownloads = downloads.filter(d => d.status !== 'downloading');

  return (
    <div
      ref={boxRef}
      className={cn(
        "absolute top-full right-0 mt-2 w-80 max-h-96",
        "bg-popover border border-border rounded-xl shadow-lg",
        "overflow-hidden z-50",
        "animate-in fade-in-0 zoom-in-95 slide-in-from-top-2"
      )}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
        <div className="flex items-center gap-2">
          <Icons.Download className="w-4 h-4 text-primary" />
          <span className="font-medium">Downloads</span>
          {activeDownloads.length > 0 && (
            <span className="px-1.5 py-0.5 text-xs bg-primary/10 text-primary rounded-full">
              {activeDownloads.length}
            </span>
          )}
        </div>
        {downloads.length > 0 && (
          <button
            onClick={onClearAll}
            className="text-xs text-muted-foreground hover:text-foreground transition-colors"
          >
            Clear all
          </button>
        )}
      </div>

      {/* Content */}
      <div className="overflow-y-auto max-h-72">
        {downloads.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
            <Icons.Download className="w-8 h-8 mb-2 opacity-50" />
            <span className="text-sm">No downloads yet</span>
          </div>
        ) : (
          <div className="p-2 space-y-1">
            {/* Active downloads first */}
            {activeDownloads.map(item => (
              <DownloadItemRow
                key={item.id}
                item={item}
                onRemove={() => onRemoveItem(item.id)}
                onOpenFile={() => {}}
                onShowInFolder={() => {}}
              />
            ))}
            {/* Completed/failed downloads */}
            {completedDownloads.map(item => (
              <DownloadItemRow
                key={item.id}
                item={item}
                onRemove={() => onRemoveItem(item.id)}
                onOpenFile={() => item.path && onOpenFile(item.path)}
                onShowInFolder={() => item.path && onShowInFolder(item.path)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      {downloads.length > 0 && (
        <div className="px-4 py-2 border-t border-border bg-muted/30">
          <button
            onClick={() => onShowInFolder(downloads[0]?.path || '')}
            className="w-full text-xs text-muted-foreground hover:text-foreground transition-colors text-center"
          >
            Open downloads folder
          </button>
        </div>
      )}
    </div>
  );
}

/**
 * Hook to manage download history
 */
const DOWNLOADS_STORAGE_KEY = 'auroraview-downloads';
const MAX_DOWNLOADS = 50;

export function useDownloads() {
  const [downloads, setDownloads] = useState<DownloadItem[]>(() => {
    try {
      const saved = localStorage.getItem(DOWNLOADS_STORAGE_KEY);
      if (saved) {
        return JSON.parse(saved);
      }
    } catch {
      // Ignore parse errors
    }
    return [];
  });

  // Save to localStorage
  useEffect(() => {
    try {
      localStorage.setItem(DOWNLOADS_STORAGE_KEY, JSON.stringify(downloads));
    } catch {
      // Ignore storage errors
    }
  }, [downloads]);

  // Listen for download events
  useEffect(() => {
    const handleDownloadStarted = (event: CustomEvent) => {
      const { url, path } = event.detail || {};
      if (!url) return;

      const filename = path?.split(/[/\\]/).pop() || url.split('/').pop() || 'download';
      const id = `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;

      setDownloads(prev => {
        const newDownloads = [
          {
            id,
            url,
            filename,
            path,
            status: 'downloading' as const,
            startTime: Date.now(),
          },
          ...prev,
        ].slice(0, MAX_DOWNLOADS);
        return newDownloads;
      });
    };

    const handleDownloadCompleted = (event: CustomEvent) => {
      const { url, path, success } = event.detail || {};
      if (!url) return;

      setDownloads(prev => {
        return prev.map(item => {
          if (item.url === url && item.status === 'downloading') {
            return {
              ...item,
              path,
              status: success ? 'completed' : 'failed',
              endTime: Date.now(),
            };
          }
          return item;
        });
      });
    };

    const handleDownloadCancelled = (event: CustomEvent) => {
      const { url } = event.detail || {};
      if (!url) return;

      setDownloads(prev => {
        return prev.map(item => {
          if (item.url === url && item.status === 'downloading') {
            return {
              ...item,
              status: 'cancelled',
              endTime: Date.now(),
            };
          }
          return item;
        });
      });
    };

    // Subscribe to AuroraView events
    const auroraview = (window as any).auroraview;
    let unsubStarted: (() => void) | undefined;
    let unsubCompleted: (() => void) | undefined;
    let unsubCancelled: (() => void) | undefined;

    if (auroraview?.on) {
      unsubStarted = auroraview.on('download_started', (data: any) => {
        handleDownloadStarted({ detail: data } as CustomEvent);
      });
      unsubCompleted = auroraview.on('download_completed', (data: any) => {
        handleDownloadCompleted({ detail: data } as CustomEvent);
      });
      unsubCancelled = auroraview.on('download_cancelled', (data: any) => {
        handleDownloadCancelled({ detail: data } as CustomEvent);
      });
    }

    // Also listen for window events (fallback)
    window.addEventListener('download_started', handleDownloadStarted as EventListener);
    window.addEventListener('download_completed', handleDownloadCompleted as EventListener);
    window.addEventListener('download_cancelled', handleDownloadCancelled as EventListener);

    return () => {
      unsubStarted?.();
      unsubCompleted?.();
      unsubCancelled?.();
      window.removeEventListener('download_started', handleDownloadStarted as EventListener);
      window.removeEventListener('download_completed', handleDownloadCompleted as EventListener);
      window.removeEventListener('download_cancelled', handleDownloadCancelled as EventListener);
    };
  }, []);

  const clearAll = useCallback(() => {
    setDownloads([]);
  }, []);

  const removeItem = useCallback((id: string) => {
    setDownloads(prev => prev.filter(item => item.id !== id));
  }, []);

  const activeCount = downloads.filter(d => d.status === 'downloading').length;

  return {
    downloads,
    clearAll,
    removeItem,
    activeCount,
  };
}
