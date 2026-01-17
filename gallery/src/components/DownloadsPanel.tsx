/**
 * DownloadsPanel - Download management panel
 */

import { useState, useMemo } from 'react';
import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

export type DownloadState = 'pending' | 'downloading' | 'paused' | 'completed' | 'failed' | 'cancelled';

export interface DownloadItem {
  id: string;
  url: string;
  filename: string;
  path?: string;
  size?: number;
  downloadedBytes: number;
  state: DownloadState;
  error?: string;
  startedAt: string;
  completedAt?: string;
  speed?: number; // bytes per second
}

interface DownloadsPanelProps {
  downloads: DownloadItem[];
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onCancel: (id: string) => void;
  onRetry: (id: string) => void;
  onRemove: (id: string) => void;
  onOpenFile: (id: string) => void;
  onOpenFolder: (id: string) => void;
  onClearCompleted: () => void;
  onClose: () => void;
  className?: string;
}

type FilterType = 'all' | 'active' | 'completed';

export function DownloadsPanel({
  downloads,
  onPause,
  onResume,
  onCancel,
  onRetry,
  onRemove,
  onOpenFile,
  onOpenFolder,
  onClearCompleted,
  onClose,
  className,
}: DownloadsPanelProps) {
  const [filter, setFilter] = useState<FilterType>('all');

  // Filter downloads
  const filteredDownloads = useMemo(() => {
    switch (filter) {
      case 'active':
        return downloads.filter(d => ['pending', 'downloading', 'paused'].includes(d.state));
      case 'completed':
        return downloads.filter(d => d.state === 'completed');
      default:
        return downloads;
    }
  }, [downloads, filter]);

  // Stats
  const activeCount = downloads.filter(d => ['pending', 'downloading', 'paused'].includes(d.state)).length;
  const completedCount = downloads.filter(d => d.state === 'completed').length;

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatSpeed = (bytesPerSecond: number): string => {
    return formatBytes(bytesPerSecond) + '/s';
  };

  const getStateIcon = (state: DownloadState) => {
    switch (state) {
      case 'downloading':
        return <Icons.ArrowDown className="w-4 h-4 text-blue-500 animate-pulse" />;
      case 'paused':
        return <Icons.Pause className="w-4 h-4 text-yellow-500" />;
      case 'completed':
        return <Icons.CheckCircle className="w-4 h-4 text-green-500" />;
      case 'failed':
        return <Icons.XCircle className="w-4 h-4 text-red-500" />;
      case 'cancelled':
        return <Icons.Ban className="w-4 h-4 text-muted-foreground" />;
      default:
        return <Icons.Clock className="w-4 h-4 text-muted-foreground" />;
    }
  };

  const getFileIcon = (filename: string) => {
    const ext = filename.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'pdf':
        return <Icons.FileText className="w-8 h-8 text-red-500" />;
      case 'zip':
      case 'rar':
      case '7z':
        return <Icons.Archive className="w-8 h-8 text-yellow-500" />;
      case 'jpg':
      case 'jpeg':
      case 'png':
      case 'gif':
      case 'webp':
        return <Icons.Image className="w-8 h-8 text-purple-500" />;
      case 'mp4':
      case 'mov':
      case 'avi':
      case 'mkv':
        return <Icons.Video className="w-8 h-8 text-blue-500" />;
      case 'mp3':
      case 'wav':
      case 'flac':
        return <Icons.Music className="w-8 h-8 text-pink-500" />;
      default:
        return <Icons.File className="w-8 h-8 text-muted-foreground" />;
    }
  };

  return (
    <div className={cn("flex flex-col h-full bg-card", className)}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-border">
        <div className="flex items-center gap-2">
          <Icons.Download className="w-5 h-5 text-primary" />
          <h2 className="text-lg font-semibold">Downloads</h2>
          {activeCount > 0 && (
            <span className="px-2 py-0.5 text-xs bg-primary text-primary-foreground rounded-full">
              {activeCount} active
            </span>
          )}
        </div>
        <button
          onClick={onClose}
          className="p-2 hover:bg-muted rounded-lg transition-colors"
        >
          <Icons.X className="w-5 h-5" />
        </button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-1 p-3 border-b border-border">
        {(['all', 'active', 'completed'] as FilterType[]).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={cn(
              "px-3 py-1.5 text-sm rounded-lg transition-colors",
              filter === f
                ? "bg-primary text-primary-foreground"
                : "hover:bg-muted"
            )}
          >
            {f.charAt(0).toUpperCase() + f.slice(1)}
            {f === 'active' && activeCount > 0 && (
              <span className="ml-1.5 text-xs opacity-75">({activeCount})</span>
            )}
            {f === 'completed' && completedCount > 0 && (
              <span className="ml-1.5 text-xs opacity-75">({completedCount})</span>
            )}
          </button>
        ))}
      </div>

      {/* Downloads list */}
      <div className="flex-1 overflow-y-auto">
        {filteredDownloads.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <Icons.Download className="w-12 h-12 mb-3 opacity-50" />
            <p>No downloads</p>
          </div>
        ) : (
          <div className="divide-y divide-border">
            {filteredDownloads.map((download) => (
              <div key={download.id} className="p-4 hover:bg-muted/30">
                <div className="flex items-start gap-3">
                  {/* File icon */}
                  <div className="flex-shrink-0">
                    {getFileIcon(download.filename)}
                  </div>

                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium truncate">{download.filename}</span>
                      {getStateIcon(download.state)}
                    </div>
                    
                    {/* Progress bar for active downloads */}
                    {['downloading', 'paused'].includes(download.state) && download.size && (
                      <div className="mt-2">
                        <div className="h-2 bg-muted rounded-full overflow-hidden">
                          <div
                            className={cn(
                              "h-full transition-all",
                              download.state === 'downloading' ? "bg-primary" : "bg-yellow-500"
                            )}
                            style={{
                              width: `${(download.downloadedBytes / download.size) * 100}%`
                            }}
                          />
                        </div>
                        <div className="flex items-center justify-between mt-1 text-xs text-muted-foreground">
                          <span>
                            {formatBytes(download.downloadedBytes)} / {formatBytes(download.size)}
                          </span>
                          {download.speed && download.state === 'downloading' && (
                            <span>{formatSpeed(download.speed)}</span>
                          )}
                        </div>
                      </div>
                    )}

                    {/* Size for completed downloads */}
                    {download.state === 'completed' && download.size && (
                      <div className="mt-1 text-sm text-muted-foreground">
                        {formatBytes(download.size)}
                      </div>
                    )}

                    {/* Error message */}
                    {download.state === 'failed' && download.error && (
                      <div className="mt-1 text-sm text-red-500">
                        {download.error}
                      </div>
                    )}

                    {/* Actions */}
                    <div className="flex items-center gap-2 mt-2">
                      {download.state === 'downloading' && (
                        <button
                          onClick={() => onPause(download.id)}
                          className="text-xs px-2 py-1 rounded hover:bg-muted transition-colors"
                        >
                          Pause
                        </button>
                      )}
                      {download.state === 'paused' && (
                        <button
                          onClick={() => onResume(download.id)}
                          className="text-xs px-2 py-1 rounded hover:bg-muted transition-colors"
                        >
                          Resume
                        </button>
                      )}
                      {['downloading', 'paused', 'pending'].includes(download.state) && (
                        <button
                          onClick={() => onCancel(download.id)}
                          className="text-xs px-2 py-1 rounded hover:bg-muted transition-colors text-red-500"
                        >
                          Cancel
                        </button>
                      )}
                      {download.state === 'failed' && (
                        <button
                          onClick={() => onRetry(download.id)}
                          className="text-xs px-2 py-1 rounded hover:bg-muted transition-colors"
                        >
                          Retry
                        </button>
                      )}
                      {download.state === 'completed' && (
                        <>
                          <button
                            onClick={() => onOpenFile(download.id)}
                            className="text-xs px-2 py-1 rounded hover:bg-muted transition-colors"
                          >
                            Open
                          </button>
                          <button
                            onClick={() => onOpenFolder(download.id)}
                            className="text-xs px-2 py-1 rounded hover:bg-muted transition-colors"
                          >
                            Show in folder
                          </button>
                        </>
                      )}
                      {['completed', 'failed', 'cancelled'].includes(download.state) && (
                        <button
                          onClick={() => onRemove(download.id)}
                          className="text-xs px-2 py-1 rounded hover:bg-muted transition-colors text-muted-foreground"
                        >
                          Remove
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      {completedCount > 0 && (
        <div className="p-3 border-t border-border">
          <button
            onClick={onClearCompleted}
            className="w-full py-2 text-sm bg-muted hover:bg-muted/80 rounded-lg transition-colors"
          >
            Clear completed downloads
          </button>
        </div>
      )}
    </div>
  );
}
