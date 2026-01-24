import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';
import { useState, useCallback, useEffect, type DragEvent } from 'react';
import { ExtensionCard } from './ExtensionCard';
import { ExtensionDetail } from './ExtensionDetail';

export interface InstalledExtension {
  id: string;
  name: string;
  version: string;
  description: string;
  path: string;
  hasSidePanel?: boolean;
  sidePanelPath?: string;
  hasPopup?: boolean;
  popupPath?: string;
  enabled?: boolean;
  permissions?: string[];
  hostPermissions?: string[];
  installType?: 'admin' | 'development' | 'normal' | 'sideload' | 'other';
  homepageUrl?: string;
  optionsUrl?: string;
  icons?: { size: number; url: string }[];
}

type NavSection = 'extensions' | 'shortcuts';

interface ExtensionPanelProps {
  extensions: InstalledExtension[];
  pendingRestart: boolean;
  onInstallExtension: (path: string) => Promise<{ ok: boolean; message?: string; error?: string; requiresRestart?: boolean }>;
  onInstallFromUrl: (url: string) => Promise<{ ok: boolean; message?: string; error?: string; requiresRestart?: boolean }>;
  onRemoveExtension: (id: string) => Promise<{ ok: boolean; error?: string }>;
  onOpenExtensionsDir: () => Promise<void>;
  onRestartApp: () => Promise<void>;
  onRefresh: () => Promise<void>;
  onOpenSidePanel?: (extension: InstalledExtension) => void;
  onOpenPopup?: (extension: InstalledExtension) => void;
  onToggleExtension?: (extension: InstalledExtension, enabled: boolean) => Promise<void>;
  onOpenStore?: () => void;
  onOpenOptions?: (extension: InstalledExtension) => void;
  onViewPermissions?: (extension: InstalledExtension) => void;
  developerMode?: boolean;
  onToggleDeveloperMode?: (enabled: boolean) => void;
}

export function ExtensionPanel({
  extensions,
  pendingRestart,
  onInstallExtension,
  onInstallFromUrl,
  onRemoveExtension,
  onOpenExtensionsDir,
  onRestartApp,
  onRefresh,
  onOpenSidePanel,
  onOpenPopup,
  onToggleExtension,
  onOpenStore,
  onOpenOptions,
  onViewPermissions,
  developerMode = true,
  onToggleDeveloperMode,
}: ExtensionPanelProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [isRestarting, setIsRestarting] = useState(false);
  const [installStatus, setInstallStatus] = useState<{ type: 'success' | 'error' | 'info'; message: string } | null>(null);
  const [localPendingRestart, setLocalPendingRestart] = useState(pendingRestart);
  const [selectedExtensionId, setSelectedExtensionId] = useState<string | null>(null);
  const [urlInput, setUrlInput] = useState('');
  const [isInstalling, setIsInstalling] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [navSection, setNavSection] = useState<NavSection>('extensions');

  useEffect(() => {
    setLocalPendingRestart(pendingRestart);
  }, [pendingRestart]);

  // Listen for native file drop events from AuroraView
  useEffect(() => {
    const handleNativeFileDrop = async (data: unknown) => {
      const fileDropData = data as { paths?: string[] };
      const paths = fileDropData.paths || [];
      if (paths.length === 0) return;

      const path = paths[0];
      console.log('[ExtensionPanel] Installing extension from:', path);

      setInstallStatus(null);
      try {
        const result = await onInstallExtension(path);
        if (result.ok) {
          setInstallStatus({ type: 'success', message: result.message || '扩展程序已安装！' });
          if (result.requiresRestart) {
            setLocalPendingRestart(true);
          }
          await onRefresh();
        } else {
          setInstallStatus({ type: 'error', message: result.error || '安装扩展程序失败' });
        }
      } catch (error) {
        setInstallStatus({ type: 'error', message: `安装失败: ${error}` });
      }

      setTimeout(() => setInstallStatus(null), 5000);
    };

    if (window.auroraview?.on) {
      window.auroraview.on('file_drop', handleNativeFileDrop);
    }

    return () => {
      if (window.auroraview?.off) {
        window.auroraview.off('file_drop', handleNativeFileDrop);
      }
    };
  }, [onInstallExtension, onRefresh]);

  // HTML5 drag handlers (for visual feedback only)
  const handleDragOver = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
    // Actual file handling is done via native file_drop event
  }, []);

  const handleRemove = useCallback(async (id: string) => {
    // Confirm removal
    if (!confirm('确定要移除此扩展程序吗？')) return;

    try {
      const result = await onRemoveExtension(id);
      if (result.ok) {
        setInstallStatus({ type: 'success', message: '扩展程序已移除，重启后生效。' });
        setLocalPendingRestart(true);
        setSelectedExtensionId(null);
        await onRefresh();
      } else {
        setInstallStatus({ type: 'error', message: result.error || '移除扩展程序失败' });
      }
    } catch (error) {
      setInstallStatus({ type: 'error', message: `移除失败: ${error}` });
    }
    setTimeout(() => setInstallStatus(null), 5000);
  }, [onRemoveExtension, onRefresh]);

  const handleRestart = useCallback(async () => {
    setIsRestarting(true);
    try {
      await onRestartApp();
    } catch (e) {
      console.error('Failed to restart:', e);
      setIsRestarting(false);
    }
  }, [onRestartApp]);

  const handleToggle = useCallback(async (ext: InstalledExtension, enabled: boolean) => {
    if (onToggleExtension) {
      try {
        await onToggleExtension(ext, enabled);
        await onRefresh();
      } catch (e) {
        console.error('Failed to toggle extension:', e);
      }
    }
  }, [onToggleExtension, onRefresh]);

  // Handle URL installation
  const handleInstallFromUrl = useCallback(async () => {
    const url = urlInput.trim();
    if (!url) return;

    // Validate URL format (Chrome/Edge web store URLs)
    const chromePatternOld = /^https:\/\/chrome\.google\.com\/webstore\/detail\/[^/]+\/([a-z]{32})/i;
    const chromePatternNew = /^https:\/\/chromewebstore\.google\.com\/detail\/[^/]+\/([a-z]{32})/i;
    const edgePattern = /^https:\/\/microsoftedge\.microsoft\.com\/addons\/detail\/[^/]+\/([a-z]{32})/i;
    
    const chromeMatchOld = url.match(chromePatternOld);
    const chromeMatchNew = url.match(chromePatternNew);
    const edgeMatch = url.match(edgePattern);
    
    if (!chromeMatchOld && !chromeMatchNew && !edgeMatch) {
      setInstallStatus({ 
        type: 'error', 
        message: '无效的 URL。请粘贴 Chrome 网上应用店或 Edge 加载项 URL。' 
      });
      setTimeout(() => setInstallStatus(null), 5000);
      return;
    }

    setIsInstalling(true);
    setInstallStatus({ type: 'info', message: '正在下载扩展程序...' });

    try {
      const result = await onInstallFromUrl(url);
      if (result.ok) {
        setInstallStatus({ type: 'success', message: result.message || '扩展程序已安装！' });
        if (result.requiresRestart) {
          setLocalPendingRestart(true);
        }
        setUrlInput('');
        await onRefresh();
      } else {
        setInstallStatus({ type: 'error', message: result.error || '安装扩展程序失败' });
      }
    } catch (error) {
      setInstallStatus({ type: 'error', message: `安装失败: ${error}` });
    } finally {
      setIsInstalling(false);
      setTimeout(() => setInstallStatus(null), 5000);
    }
  }, [urlInput, onInstallFromUrl, onRefresh]);

  // Filter extensions based on search
  const filteredExtensions = extensions.filter(ext => {
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      return (
        ext.name.toLowerCase().includes(query) ||
        ext.description.toLowerCase().includes(query) ||
        ext.id.toLowerCase().includes(query)
      );
    }
    return true;
  });

  // Render Details View
  if (selectedExtensionId) {
    const extension = extensions.find(e => e.id === selectedExtensionId);
    if (extension) {
      return (
        <ExtensionDetail
          extension={extension}
          onBack={() => setSelectedExtensionId(null)}
          onToggle={handleToggle}
          onRemove={handleRemove}
          onOpenSidePanel={onOpenSidePanel}
          onOpenOptions={onOpenOptions}
          onViewPermissions={onViewPermissions}
        />
      );
    } else {
      setSelectedExtensionId(null);
    }
  }

  // Chrome-style Extension Manager Layout
  return (
    <div className="flex min-h-[600px]">
      {/* Left Sidebar - Chrome Style */}
      <div className="w-56 border-r border-border bg-muted/20 flex-shrink-0">
        {/* Header with puzzle icon */}
        <div className="p-4 flex items-center gap-3 border-b border-border">
          <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
            <Icons.Puzzle className="w-5 h-5 text-primary" />
          </div>
          <h1 className="text-lg font-semibold">扩展程序</h1>
        </div>

        {/* Developer mode actions */}
        {developerMode && (
          <div className="p-3 space-y-1 border-b border-border">
            <button
              onClick={onOpenExtensionsDir}
              className="w-full flex items-center gap-2 px-3 py-2 text-sm text-primary font-medium rounded-lg hover:bg-primary/5 border border-primary transition-colors"
            >
              加载已解压的扩展程序
            </button>
            <button
              onClick={() => {}}
              className="w-full flex items-center gap-2 px-3 py-2 text-sm text-primary font-medium rounded-lg hover:bg-primary/5 border border-primary transition-colors"
            >
              打包扩展程序
            </button>
            <button
              onClick={onRefresh}
              className="w-full flex items-center gap-2 px-3 py-2 text-sm text-primary font-medium rounded-lg hover:bg-primary/5 border border-primary transition-colors"
            >
              更新
            </button>
          </div>
        )}

        {/* Navigation */}
        <nav className="p-2">
          <button
            onClick={() => setNavSection('extensions')}
            className={cn(
              "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors",
              navSection === 'extensions'
                ? "bg-primary/10 text-primary"
                : "text-foreground hover:bg-muted"
            )}
          >
            <Icons.Puzzle className="w-5 h-5" />
            我的扩展程序
          </button>
          <button
            onClick={() => setNavSection('shortcuts')}
            className={cn(
              "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors",
              navSection === 'shortcuts'
                ? "bg-primary/10 text-primary"
                : "text-foreground hover:bg-muted"
            )}
          >
            <Icons.Keyboard className="w-5 h-5" />
            键盘快捷键
          </button>
        </nav>

        {/* Help text */}
        <div className="p-4 mt-4 text-xs text-muted-foreground">
          <p className="mb-3">
            正在开发扩展程序？请参阅
            <br />
            Chrome 扩展程序开发者文档。
          </p>
          <a 
            href="https://chromewebstore.google.com"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 text-primary hover:underline"
          >
            <Icons.Store className="w-4 h-4" />
            在 Chrome 应用商店 中发现更多扩展程序和主题
          </a>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-auto">
        {/* Top Bar */}
        <div className="sticky top-0 z-10 bg-background border-b border-border p-4 flex items-center justify-between gap-4">
          {/* Search Bar */}
          <div className="relative flex-1 max-w-md">
            <Icons.Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="搜索扩展程序"
              className={cn(
                "w-full pl-9 pr-3 py-2 text-sm rounded-full",
                "bg-muted/50 border border-transparent",
                "focus:outline-none focus:border-primary focus:bg-background"
              )}
            />
          </div>

          {/* Developer mode toggle */}
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">开发者模式</span>
            <button
              onClick={() => onToggleDeveloperMode?.(!developerMode)}
              className={cn(
                "relative inline-flex h-6 w-11 items-center rounded-full transition-colors",
                developerMode ? "bg-primary" : "bg-muted-foreground/30"
              )}
            >
              <span
                className={cn(
                  "inline-block h-5 w-5 transform rounded-full bg-white shadow-sm transition-transform",
                  developerMode ? "translate-x-[22px]" : "translate-x-[2px]"
                )}
              />
            </button>
          </div>
        </div>

        {/* Install from URL (when in developer mode) */}
        {developerMode && (
          <div className="p-4 border-b border-border">
            <div className="flex gap-2 max-w-2xl">
              <input
                type="text"
                value={urlInput}
                onChange={(e) => setUrlInput(e.target.value)}
                placeholder="从 Chrome 应用商店 URL 安装扩展程序..."
                className={cn(
                  "flex-1 px-4 py-2 text-sm rounded-lg",
                  "bg-background border border-border",
                  "focus:outline-none focus:ring-2 focus:ring-primary/50"
                )}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !isInstalling) {
                    handleInstallFromUrl();
                  }
                }}
              />
              <button
                onClick={handleInstallFromUrl}
                disabled={isInstalling || !urlInput.trim()}
                className={cn(
                  "px-4 py-2 text-sm font-medium rounded-lg transition-colors",
                  "bg-primary text-primary-foreground hover:bg-primary/90",
                  (isInstalling || !urlInput.trim()) && "opacity-50 cursor-not-allowed"
                )}
              >
                {isInstalling ? '安装中...' : '安装'}
              </button>
            </div>
          </div>
        )}

        {/* Status Messages */}
        {(localPendingRestart || installStatus) && (
          <div className="p-4 space-y-2">
            {/* Pending Restart Banner */}
            {localPendingRestart && (
              <div className="p-3 bg-amber-500/10 border border-amber-500/30 rounded-lg flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <Icons.AlertTriangle className="w-5 h-5 text-amber-600" />
                  <span className="text-sm font-medium text-amber-700 dark:text-amber-400">
                    需要重启才能应用更改
                  </span>
                </div>
                <button
                  onClick={handleRestart}
                  disabled={isRestarting}
                  className={cn(
                    "px-3 py-1.5 text-xs font-medium rounded transition-all",
                    "bg-amber-500 text-white hover:bg-amber-600",
                    isRestarting && "opacity-50 cursor-not-allowed"
                  )}
                >
                  {isRestarting ? '正在重启...' : '立即重启'}
                </button>
              </div>
            )}

            {/* Install Status */}
            {installStatus && (
              <div className={cn(
                "p-3 rounded-lg text-sm flex items-center gap-3",
                installStatus.type === 'success' && "bg-green-500/10 text-green-700 dark:text-green-400 border border-green-500/20",
                installStatus.type === 'error' && "bg-red-500/10 text-red-700 dark:text-red-400 border border-red-500/20",
                installStatus.type === 'info' && "bg-blue-500/10 text-blue-700 dark:text-blue-400 border border-blue-500/20"
              )}>
                {installStatus.type === 'success' && <Icons.CheckCircle className="w-4 h-4" />}
                {installStatus.type === 'error' && <Icons.XCircle className="w-4 h-4" />}
                {installStatus.type === 'info' && <Icons.Loader2 className="w-4 h-4 animate-spin" />}
                <span>{installStatus.message}</span>
              </div>
            )}
          </div>
        )}

        {/* Drag overlay */}
        {isDragging && (
          <div
            className="fixed inset-0 z-50 bg-primary/10 border-4 border-dashed border-primary flex items-center justify-center backdrop-blur-sm"
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            onDragOver={(e) => e.preventDefault()}
          >
            <div className="bg-card p-8 rounded-xl shadow-2xl flex flex-col items-center gap-4">
              <div className="w-16 h-16 bg-primary/20 rounded-full flex items-center justify-center">
                <Icons.Download className="w-8 h-8 text-primary" />
              </div>
              <h3 className="text-xl font-bold">拖放安装扩展程序</h3>
              <p className="text-muted-foreground">释放文件夹以安装</p>
            </div>
          </div>
        )}

        {/* Extensions Content */}
        {navSection === 'extensions' && (
          <div className="p-6" onDragOver={handleDragOver}>
            <h2 className="text-lg font-medium mb-4">所有扩展程序</h2>
            
            {filteredExtensions.length === 0 ? (
              <div className="py-16 text-center border-2 border-dashed border-border rounded-xl">
                <div className="w-12 h-12 bg-muted rounded-full flex items-center justify-center mx-auto mb-4">
                  <Icons.Puzzle className="w-6 h-6 text-muted-foreground" />
                </div>
                {extensions.length === 0 ? (
                  <>
                    <h3 className="text-lg font-semibold mb-2">没有安装扩展程序</h3>
                    <p className="text-muted-foreground max-w-sm mx-auto mb-4">
                      将扩展程序文件夹拖放到此处，或使用上方的按钮安装。
                    </p>
                    {onOpenStore && (
                      <button
                        onClick={onOpenStore}
                        className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
                      >
                        浏览应用商店
                      </button>
                    )}
                  </>
                ) : (
                  <>
                    <h3 className="text-lg font-semibold mb-2">没有匹配的扩展程序</h3>
                    <p className="text-muted-foreground">
                      尝试调整搜索条件。
                    </p>
                  </>
                )}
              </div>
            ) : (
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                {filteredExtensions.map((ext) => (
                  <ExtensionCard
                    key={ext.id}
                    extension={ext}
                    onDetails={() => setSelectedExtensionId(ext.id)}
                    onToggle={handleToggle}
                    onRemove={handleRemove}
                    onOpenSidePanel={onOpenSidePanel}
                    onOpenPopup={onOpenPopup}
                    onOpenOptions={onOpenOptions}
                  />
                ))}
              </div>
            )}
          </div>
        )}

        {/* Keyboard Shortcuts Section */}
        {navSection === 'shortcuts' && (
          <div className="p-6">
            <h2 className="text-lg font-medium mb-4">键盘快捷键</h2>
            <div className="space-y-4">
              {extensions.length === 0 ? (
                <p className="text-muted-foreground">没有安装扩展程序。</p>
              ) : (
                extensions.map((ext) => (
                  <div key={ext.id} className="p-4 bg-card border border-border rounded-lg">
                    <div className="flex items-center gap-3 mb-3">
                      <div className="w-8 h-8 rounded bg-primary/10 flex items-center justify-center">
                        <Icons.Puzzle className="w-4 h-4 text-primary" />
                      </div>
                      <span className="font-medium">{ext.name}</span>
                    </div>
                    <div className="pl-11 space-y-2">
                      <div className="flex items-center justify-between text-sm">
                        <span className="text-muted-foreground">激活扩展程序</span>
                        <button className="px-3 py-1 border border-border rounded text-xs hover:bg-muted">
                          设置快捷键
                        </button>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
