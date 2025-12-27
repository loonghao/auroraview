import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';
import { useState, useCallback, useEffect, type DragEvent } from 'react';

export type RunMode = 'external' | 'console';
export type LinkMode = 'browser' | 'webview';

export interface BrowserExtensionStatus {
  enabled: boolean;
  wsPort: number;
  httpPort: number;
  connectedClients: number;
  isRunning: boolean;
}

export interface Settings {
  runMode: RunMode;
  linkMode: LinkMode;
  browserExtension: {
    enabled: boolean;
    wsPort: number;
    httpPort: number;
  };
}

interface SettingsModalProps {
  isOpen: boolean;
  settings: Settings;
  extensionStatus?: BrowserExtensionStatus;
  onClose: () => void;
  onSave: (settings: Settings) => void;
  onToggleExtension?: (enabled: boolean) => Promise<void>;
  onOpenExtensionStore?: () => void;
  onInstallExtension?: (path: string, browser: 'chrome' | 'firefox') => Promise<void>;
  onInstallToWebView?: (path: string) => Promise<{ ok: boolean; message?: string; error?: string; requiresRestart?: boolean }>;
  onOpenExtensionsDir?: () => Promise<void>;
  onRestartApp?: () => Promise<void>;
}

export function SettingsModal({ 
  isOpen, 
  settings, 
  extensionStatus,
  onClose, 
  onSave,
  onToggleExtension,
  onOpenExtensionStore,
  onInstallExtension,
  onInstallToWebView,
  onOpenExtensionsDir,
  onRestartApp,
}: SettingsModalProps) {
  const [isTogglingExtension, setIsTogglingExtension] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [installStatus, setInstallStatus] = useState<{ type: 'success' | 'error' | 'info'; message: string } | null>(null);
  const [installTarget, setInstallTarget] = useState<'browser' | 'webview'>('webview');
  const [pendingRestart, setPendingRestart] = useState(false);
  const [isRestarting, setIsRestarting] = useState(false);

  // Listen for native file drop events from AuroraView
  // WebView2's HTML5 drag-drop doesn't provide full file paths,
  // so we use native events from Wry/WebView instead
  useEffect(() => {
    if (!isOpen) return;

    const handleNativeFileDrop = async (data: { paths?: string[]; position?: { x: number; y: number } }) => {
      console.log('[SettingsModal:handleNativeFileDrop] Received native file drop:', data);
      console.log('[SettingsModal:handleNativeFileDrop] Install target:', installTarget);
      
      const paths = data.paths || [];
      if (paths.length === 0) {
        console.log('[SettingsModal:handleNativeFileDrop] No paths in event');
        return;
      }

      // Get the first path
      const path = paths[0];
      console.log('[SettingsModal:handleNativeFileDrop] Processing path:', path);

      setInstallStatus(null);

      if (installTarget === 'webview' && onInstallToWebView) {
        // Install to WebView2's extensions directory
        try {
          console.log('[SettingsModal:handleNativeFileDrop] Installing to WebView2...');
          const result = await onInstallToWebView(path);
          
          if (result.ok) {
            const message = result.message || 'Extension installed!';
            setInstallStatus({ type: 'info', message });
            // Set pending restart if required
            if (result.requiresRestart) {
              setPendingRestart(true);
            }
          } else {
            setInstallStatus({ type: 'error', message: result.error || 'Failed to install extension' });
          }
          // Don't auto-clear if restart is pending
          if (!result.requiresRestart) {
            setTimeout(() => setInstallStatus(null), 8000);
          }
        } catch (error) {
          console.error('[SettingsModal:handleNativeFileDrop] WebView install error:', error);
          setInstallStatus({ type: 'error', message: `Failed to install: ${error}` });
        }
      } else if (onInstallExtension) {
        // Install to external browser
        const isFirefox = path.toLowerCase().endsWith('.xpi');
        const browser = isFirefox ? 'firefox' : 'chrome';

        try {
          console.log('[SettingsModal:handleNativeFileDrop] Installing to browser...');
          await onInstallExtension(path, browser);
          setInstallStatus({ type: 'success', message: `Opening ${browser} extension installer...` });
          setTimeout(() => setInstallStatus(null), 5000);
        } catch (error) {
          console.error('[SettingsModal:handleNativeFileDrop] Browser install error:', error);
          setInstallStatus({ type: 'error', message: `Failed to install: ${error}` });
        }
      }
    };

    // Subscribe to native file drop event
    const auroraview = window.auroraview;
    if (auroraview?.on) {
      console.log('[SettingsModal] Subscribing to extension:file_drop event');
      const unsubscribe = auroraview.on('extension:file_drop', handleNativeFileDrop as (data: unknown) => void);
      return () => {
        console.log('[SettingsModal] Unsubscribing from extension:file_drop event');
        unsubscribe?.();
      };
    }
  }, [isOpen, installTarget, onInstallExtension, onInstallToWebView]);

  const handleDragEnter = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    console.log('[SettingsModal:handleDragEnter] Drag enter');
    setIsDragging(true);
  }, []);

  const handleDragOver = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    // Don't log on every dragover as it fires continuously
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    console.log('[SettingsModal:handleDragLeave] Drag leave');
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback(async (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    console.log('[SettingsModal:handleDrop] ========== DROP EVENT ==========');
    setIsDragging(false);
    setInstallStatus(null);

    console.log('[SettingsModal:handleDrop] Drop event triggered');
    console.log('[SettingsModal:handleDrop] onInstallExtension available:', !!onInstallExtension);
    console.log('[SettingsModal:handleDrop] dataTransfer.items count:', e.dataTransfer.items.length);
    console.log('[SettingsModal:handleDrop] dataTransfer.files count:', e.dataTransfer.files.length);

    if (!onInstallExtension) {
      console.log('[SettingsModal:handleDrop] ERROR: onInstallExtension not available');
      setInstallStatus({ type: 'error', message: 'Extension installation not available' });
      return;
    }

    const items = Array.from(e.dataTransfer.items);
    const files = Array.from(e.dataTransfer.files);
    
    // Log all files info
    files.forEach((f, i) => {
      const fileWithPath = f as File & { path?: string };
      console.log(`[SettingsModal:handleDrop] File[${i}]:`, {
        name: f.name,
        type: f.type,
        size: f.size,
        path: fileWithPath.path,
        webkitRelativePath: f.webkitRelativePath,
      });
    });
    
    // Check for folder drop (development version)
    // In WebView/Wry, we can detect folders via webkitGetAsEntry
    let isFolder = false;
    let folderPath: string | null = null;
    
    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      console.log(`[SettingsModal:handleDrop] Item[${i}]:`, {
        kind: item.kind,
        type: item.type,
      });
      
      if (item.kind === 'file') {
        const entry = item.webkitGetAsEntry?.();
        console.log(`[SettingsModal:handleDrop] Item[${i}] webkitGetAsEntry:`, entry, 'isDirectory:', entry?.isDirectory);
        
        if (entry?.isDirectory) {
          isFolder = true;
          // Get folder path from the file object
          const file = item.getAsFile();
          if (file) {
            folderPath = (file as File & { path?: string }).path || null;
            console.log(`[SettingsModal:handleDrop] Folder detected, path:`, folderPath);
          }
          break;
        }
      }
    }

    if (isFolder && folderPath) {
      // Development version folder - default to Chrome for unpacked extensions
      console.log('[SettingsModal:handleDrop] Installing folder as unpacked extension:', folderPath);
      try {
        await onInstallExtension(folderPath, 'chrome');
        setInstallStatus({ type: 'success', message: 'Loading unpacked extension folder...' });
        setTimeout(() => setInstallStatus(null), 5000);
      } catch (error) {
        console.error('[SettingsModal:handleDrop] Folder install error:', error);
        setInstallStatus({ type: 'error', message: `Failed to load folder: ${error}` });
      }
      return;
    }

    // Check for extension files (.crx / .xpi)
    const extensionFile = files.find(f => 
      f.name.endsWith('.crx') || f.name.endsWith('.xpi')
    );

    if (!extensionFile) {
      // Check if we have any files at all - show their paths for debugging
      if (files.length > 0) {
        const fileInfo = files.map(f => {
          const fileWithPath = f as File & { path?: string };
          return `${f.name} (path: ${fileWithPath.path || 'N/A'})`;
        }).join(', ');
        console.log('[SettingsModal:handleDrop] Invalid files dropped:', fileInfo);
        setInstallStatus({ type: 'error', message: `Invalid file type. Got: ${files[0].name}` });
      } else {
        console.log('[SettingsModal:handleDrop] No files in drop event');
        setInstallStatus({ type: 'error', message: 'Please drop a .crx/.xpi file or extension folder' });
      }
      return;
    }

    const browser = extensionFile.name.endsWith('.xpi') ? 'firefox' : 'chrome';
    
    try {
      // Get the file path - in Wry/WebView we can access the path
      const filePath = (extensionFile as File & { path?: string }).path;
      console.log('[SettingsModal:handleDrop] Extension file:', {
        name: extensionFile.name,
        path: filePath,
        browser: browser,
      });
      
      if (!filePath) {
        console.log('[SettingsModal:handleDrop] ERROR: Cannot get file path');
        setInstallStatus({ type: 'error', message: 'Cannot get file path. Try running in WebView.' });
        return;
      }
      
      console.log('[SettingsModal:handleDrop] Calling onInstallExtension...');
      await onInstallExtension(filePath, browser);
      console.log('[SettingsModal:handleDrop] onInstallExtension completed');
      setInstallStatus({ type: 'success', message: `Opening ${browser} extension installer...` });
      setTimeout(() => setInstallStatus(null), 5000);
    } catch (error) {
      console.error('[SettingsModal:handleDrop] Install error:', error);
      setInstallStatus({ type: 'error', message: `Failed to install: ${error}` });
    }
  }, [onInstallExtension]);
  
  // Early return AFTER all hooks
  if (!isOpen) return null;

  const handleRunModeChange = (mode: RunMode) => {
    onSave({ ...settings, runMode: mode });
  };

  const handleLinkModeChange = (mode: LinkMode) => {
    onSave({ ...settings, linkMode: mode });
  };

  const handleExtensionToggle = async () => {
    if (!onToggleExtension || isTogglingExtension) return;
    setIsTogglingExtension(true);
    try {
      await onToggleExtension(!extensionStatus?.isRunning);
    } finally {
      setIsTogglingExtension(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative bg-card border border-border rounded-xl shadow-2xl w-full max-w-md mx-4 overflow-hidden max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-border flex-shrink-0">
          <div className="flex items-center gap-2">
            <Icons.Settings className="w-5 h-5 text-primary" />
            <h2 className="text-lg font-semibold">Settings</h2>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 rounded-lg flex items-center justify-center text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
          >
            <Icons.X className="w-4 h-4" />
          </button>
        </div>

        {/* Content */}
        <div className="p-5 space-y-6 overflow-y-auto flex-1">
          {/* Run Mode Section */}
          <div>
            <h3 className="text-sm font-medium mb-3 flex items-center gap-2">
              <Icons.Play className="w-4 h-4" />
              Run Mode
            </h3>
            <p className="text-xs text-muted-foreground mb-3">
              Choose how to run sample demos
            </p>
            <div className="space-y-2">
              <label
                className={cn(
                  "flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-all",
                  settings.runMode === 'external'
                    ? "border-primary bg-primary/5"
                    : "border-border hover:border-primary/30"
                )}
              >
                <input
                  type="radio"
                  name="runMode"
                  value="external"
                  checked={settings.runMode === 'external'}
                  onChange={() => handleRunModeChange('external')}
                  className="sr-only"
                />
                <div className={cn(
                  "w-10 h-10 rounded-lg flex items-center justify-center",
                  settings.runMode === 'external' ? "bg-primary text-white" : "bg-muted text-muted-foreground"
                )}>
                  <Icons.ExternalLink className="w-5 h-5" />
                </div>
                <div className="flex-1">
                  <div className="font-medium text-sm">New Window</div>
                  <div className="text-xs text-muted-foreground">
                    Run demos in a separate window (hidden console)
                  </div>
                </div>
                {settings.runMode === 'external' && (
                  <Icons.Check className="w-5 h-5 text-primary" />
                )}
              </label>

              <label
                className={cn(
                  "flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-all",
                  settings.runMode === 'console'
                    ? "border-primary bg-primary/5"
                    : "border-border hover:border-primary/30"
                )}
              >
                <input
                  type="radio"
                  name="runMode"
                  value="console"
                  checked={settings.runMode === 'console'}
                  onChange={() => handleRunModeChange('console')}
                  className="sr-only"
                />
                <div className={cn(
                  "w-10 h-10 rounded-lg flex items-center justify-center",
                  settings.runMode === 'console' ? "bg-primary text-white" : "bg-muted text-muted-foreground"
                )}>
                  <Icons.Terminal className="w-5 h-5" />
                </div>
                <div className="flex-1">
                  <div className="font-medium text-sm">With Console</div>
                  <div className="text-xs text-muted-foreground">
                    Run demos with visible console window for debugging
                  </div>
                </div>
                {settings.runMode === 'console' && (
                  <Icons.Check className="w-5 h-5 text-primary" />
                )}
              </label>
            </div>
          </div>

          {/* Link Mode Section */}
          <div>
            <h3 className="text-sm font-medium mb-3 flex items-center gap-2">
              <Icons.Link className="w-4 h-4" />
              Open Links
            </h3>
            <p className="text-xs text-muted-foreground mb-3">
              Choose how to open external URLs (GitHub, docs, etc.)
            </p>
            <div className="space-y-2">
              <label
                className={cn(
                  "flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-all",
                  settings.linkMode === 'browser'
                    ? "border-primary bg-primary/5"
                    : "border-border hover:border-primary/30"
                )}
              >
                <input
                  type="radio"
                  name="linkMode"
                  value="browser"
                  checked={settings.linkMode === 'browser'}
                  onChange={() => handleLinkModeChange('browser')}
                  className="sr-only"
                />
                <div className={cn(
                  "w-10 h-10 rounded-lg flex items-center justify-center",
                  settings.linkMode === 'browser' ? "bg-primary text-white" : "bg-muted text-muted-foreground"
                )}>
                  <Icons.Globe className="w-5 h-5" />
                </div>
                <div className="flex-1">
                  <div className="font-medium text-sm">Default Browser</div>
                  <div className="text-xs text-muted-foreground">
                    Open links in your system's default browser
                  </div>
                </div>
                {settings.linkMode === 'browser' && (
                  <Icons.Check className="w-5 h-5 text-primary" />
                )}
              </label>

              <label
                className={cn(
                  "flex items-center gap-3 p-3 rounded-lg border cursor-not-allowed opacity-50",
                  "border-border"
                )}
                title="Coming soon - WebView2 new window support is limited"
              >
                <input
                  type="radio"
                  name="linkMode"
                  value="webview"
                  disabled
                  className="sr-only"
                />
                <div className={cn(
                  "w-10 h-10 rounded-lg flex items-center justify-center",
                  "bg-muted text-muted-foreground"
                )}>
                  <Icons.AppWindow className="w-5 h-5" />
                </div>
                <div className="flex-1">
                  <div className="font-medium text-sm">WebView Window <span className="text-xs text-muted-foreground">(Coming Soon)</span></div>
                  <div className="text-xs text-muted-foreground">
                    Open links in a new AuroraView window
                  </div>
                </div>
              </label>
            </div>
          </div>

          {/* Browser Extension Section */}
          <div>
            <h3 className="text-sm font-medium mb-3 flex items-center gap-2">
              <Icons.Puzzle className="w-4 h-4" />
              Browser Extension
            </h3>
            <p className="text-xs text-muted-foreground mb-3">
              Enable browser extension bridge to communicate with Chrome/Firefox extensions
            </p>
            
            {/* Extension Toggle */}
            <div className={cn(
              "flex items-center gap-3 p-3 rounded-lg border transition-all",
              extensionStatus?.isRunning
                ? "border-green-500/50 bg-green-500/5"
                : "border-border"
            )}>
              <div className={cn(
                "w-10 h-10 rounded-lg flex items-center justify-center",
                extensionStatus?.isRunning ? "bg-green-500 text-white" : "bg-muted text-muted-foreground"
              )}>
                {extensionStatus?.isRunning ? (
                  <Icons.Wifi className="w-5 h-5" />
                ) : (
                  <Icons.WifiOff className="w-5 h-5" />
                )}
              </div>
              <div className="flex-1">
                <div className="font-medium text-sm flex items-center gap-2">
                  Extension Bridge
                  {extensionStatus?.isRunning && (
                    <span className="px-1.5 py-0.5 text-[10px] font-medium bg-green-500/20 text-green-600 rounded">
                      Running
                    </span>
                  )}
                </div>
                <div className="text-xs text-muted-foreground">
                  {extensionStatus?.isRunning ? (
                    <>
                      WS: {extensionStatus.wsPort} | HTTP: {extensionStatus.httpPort} | 
                      Clients: {extensionStatus.connectedClients}
                    </>
                  ) : (
                    "Start bridge to connect browser extensions"
                  )}
                </div>
              </div>
              <button
                onClick={handleExtensionToggle}
                disabled={isTogglingExtension}
                className={cn(
                  "px-3 py-1.5 text-xs font-medium rounded-lg transition-all",
                  extensionStatus?.isRunning
                    ? "bg-red-500/10 text-red-600 hover:bg-red-500/20"
                    : "bg-primary/10 text-primary hover:bg-primary/20",
                  isTogglingExtension && "opacity-50 cursor-not-allowed"
                )}
              >
                {isTogglingExtension ? (
                  <Icons.Loader2 className="w-4 h-4 animate-spin" />
                ) : extensionStatus?.isRunning ? (
                  "Stop"
                ) : (
                  "Start"
                )}
              </button>
            </div>

            {/* Drag & Drop Install Zone */}
            <div
              onDragEnter={handleDragEnter}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
              className={cn(
                "mt-3 p-4 rounded-lg border-2 border-dashed transition-all text-center",
                isDragging
                  ? "border-primary bg-primary/5"
                  : "border-border hover:border-primary/30",
                installStatus?.type === 'success' && "border-green-500/50 bg-green-500/5",
                installStatus?.type === 'error' && "border-red-500/50 bg-red-500/5",
                installStatus?.type === 'info' && "border-blue-500/50 bg-blue-500/5"
              )}
            >
              {installStatus ? (
                <div className={cn(
                  "flex items-center justify-center gap-2 text-sm",
                  installStatus.type === 'success' && "text-green-600",
                  installStatus.type === 'error' && "text-red-600",
                  installStatus.type === 'info' && "text-blue-600"
                )}>
                  {installStatus.type === 'success' ? (
                    <Icons.CheckCircle className="w-4 h-4" />
                  ) : installStatus.type === 'info' ? (
                    <Icons.Info className="w-4 h-4" />
                  ) : (
                    <Icons.XCircle className="w-4 h-4" />
                  )}
                  {installStatus.message}
                </div>
              ) : isDragging ? (
                <div className="flex flex-col items-center gap-2 text-primary">
                  <Icons.Download className="w-6 h-6" />
                  <span className="text-sm font-medium">
                    Drop to install {installTarget === 'webview' ? 'to WebView' : 'to browser'}
                  </span>
                </div>
              ) : (
                <div className="flex flex-col items-center gap-2 text-muted-foreground">
                  <Icons.Upload className="w-6 h-6" />
                  <span className="text-sm">Drag & drop extension to this window</span>
                  <span className="text-xs">Unpacked extension folder (with manifest.json)</span>
                  <span className="text-xs opacity-60">(Drop anywhere on the window)</span>
                </div>
              )}
            </div>

            {/* Install Target Selection */}
            <div className="mt-3 flex items-center gap-2">
              <span className="text-xs text-muted-foreground">Install to:</span>
              <div className="flex gap-1 p-0.5 bg-muted rounded-lg">
                <button
                  onClick={() => setInstallTarget('webview')}
                  className={cn(
                    "px-3 py-1 text-xs font-medium rounded-md transition-colors",
                    installTarget === 'webview' 
                      ? "bg-background shadow text-foreground" 
                      : "text-muted-foreground hover:text-foreground"
                  )}
                >
                  <Icons.Box className="w-3 h-3 inline mr-1" />
                  WebView (Built-in)
                </button>
                <button
                  onClick={() => setInstallTarget('browser')}
                  className={cn(
                    "px-3 py-1 text-xs font-medium rounded-md transition-colors",
                    installTarget === 'browser' 
                      ? "bg-background shadow text-foreground" 
                      : "text-muted-foreground hover:text-foreground"
                  )}
                >
                  <Icons.Globe className="w-3 h-3 inline mr-1" />
                  External Browser
                </button>
              </div>
            </div>

            {/* Extension Store Link */}
            <div className="mt-3 flex gap-2">
              {installTarget === 'webview' && onOpenExtensionsDir && (
                <button
                  onClick={onOpenExtensionsDir}
                  className="flex-1 flex items-center justify-center gap-2 px-3 py-2 text-xs font-medium rounded-lg border border-border hover:bg-accent transition-colors"
                >
                  <Icons.FolderOpen className="w-4 h-4" />
                  Open Extensions Folder
                </button>
              )}
              {installTarget === 'browser' && (
                <button
                  onClick={onOpenExtensionStore}
                  className="flex-1 flex items-center justify-center gap-2 px-3 py-2 text-xs font-medium rounded-lg border border-border hover:bg-accent transition-colors"
                >
                  <Icons.Download className="w-4 h-4" />
                  Get Browser Extension
                </button>
              )}
              <button
                onClick={() => {
                  // Copy connection info to clipboard
                  const info = `WebSocket: ws://127.0.0.1:${extensionStatus?.wsPort || 49152}\nHTTP: http://127.0.0.1:${extensionStatus?.httpPort || 49153}`;
                  navigator.clipboard.writeText(info);
                }}
                className="px-3 py-2 text-xs font-medium rounded-lg border border-border hover:bg-accent transition-colors"
                title="Copy connection info"
              >
                <Icons.Copy className="w-4 h-4" />
              </button>
            </div>

            {/* WebView Extension Info */}
            {installTarget === 'webview' && (
              <div className="mt-3 p-2 bg-muted/50 rounded-lg">
                <div className="flex items-start gap-2">
                  <Icons.Info className="w-3 h-3 text-muted-foreground mt-0.5" />
                  <div className="text-xs text-muted-foreground">
                    Extensions installed to WebView will be loaded on next app restart.
                    Drop an unpacked extension folder (containing manifest.json).
                  </div>
                </div>
              </div>
            )}

            {/* Pending Restart Banner */}
            {pendingRestart && installTarget === 'webview' && (
              <div className="mt-3 p-3 bg-amber-500/10 border border-amber-500/30 rounded-lg">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Icons.RefreshCw className="w-4 h-4 text-amber-600" />
                    <div className="text-sm text-amber-700 dark:text-amber-400">
                      Restart required to load new extension
                    </div>
                  </div>
                  <button
                    onClick={async () => {
                      if (onRestartApp) {
                        setIsRestarting(true);
                        try {
                          await onRestartApp();
                        } catch (e) {
                          console.error('Failed to restart:', e);
                          setIsRestarting(false);
                        }
                      }
                    }}
                    disabled={isRestarting}
                    className={cn(
                      "px-3 py-1.5 text-xs font-medium rounded-lg transition-all",
                      "bg-amber-500 text-white hover:bg-amber-600",
                      isRestarting && "opacity-50 cursor-not-allowed"
                    )}
                  >
                    {isRestarting ? (
                      <span className="flex items-center gap-1">
                        <Icons.Loader2 className="w-3 h-3 animate-spin" />
                        Restarting...
                      </span>
                    ) : (
                      <span className="flex items-center gap-1">
                        <Icons.RefreshCw className="w-3 h-3" />
                        Restart Now
                      </span>
                    )}
                  </button>
                </div>
              </div>
            )}
          </div>

          {/* Info */}
          <div className="p-3 bg-muted/50 rounded-lg">
            <div className="flex items-start gap-2">
              <Icons.Info className="w-4 h-4 text-muted-foreground mt-0.5" />
              <div className="text-xs text-muted-foreground">
                Settings are saved automatically and persist across sessions.
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 px-5 py-4 border-t border-border bg-muted/30 flex-shrink-0">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium rounded-lg hover:bg-accent transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
