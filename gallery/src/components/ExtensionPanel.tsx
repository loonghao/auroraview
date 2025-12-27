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

type ViewMode = 'grid' | 'list';
type FilterMode = 'all' | 'enabled' | 'disabled' | 'development';

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
  const [viewMode, setViewMode] = useState<ViewMode>('grid');
  const [filterMode, setFilterMode] = useState<FilterMode>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [showUrlInput, setShowUrlInput] = useState(false);

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
          setInstallStatus({ type: 'success', message: result.message || 'Extension installed!' });
          if (result.requiresRestart) {
            setLocalPendingRestart(true);
          }
          await onRefresh();
        } else {
          setInstallStatus({ type: 'error', message: result.error || 'Failed to install extension' });
        }
      } catch (error) {
        setInstallStatus({ type: 'error', message: `Failed to install: ${error}` });
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
    if (!confirm('Are you sure you want to remove this extension?')) return;

    try {
      const result = await onRemoveExtension(id);
      if (result.ok) {
        setInstallStatus({ type: 'success', message: 'Extension removed. Restart to apply.' });
        setLocalPendingRestart(true);
        setSelectedExtensionId(null); // Go back to list if viewing details
        await onRefresh();
      } else {
        setInstallStatus({ type: 'error', message: result.error || 'Failed to remove extension' });
      }
    } catch (error) {
      setInstallStatus({ type: 'error', message: `Failed to remove: ${error}` });
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
    // Support both old and new Chrome Web Store URL formats:
    // - Old: https://chrome.google.com/webstore/detail/name/id
    // - New: https://chromewebstore.google.com/detail/name/id
    const chromePatternOld = /^https:\/\/chrome\.google\.com\/webstore\/detail\/[^/]+\/([a-z]{32})/i;
    const chromePatternNew = /^https:\/\/chromewebstore\.google\.com\/detail\/[^/]+\/([a-z]{32})/i;
    const edgePattern = /^https:\/\/microsoftedge\.microsoft\.com\/addons\/detail\/[^/]+\/([a-z]{32})/i;
    
    const chromeMatchOld = url.match(chromePatternOld);
    const chromeMatchNew = url.match(chromePatternNew);
    const edgeMatch = url.match(edgePattern);
    
    if (!chromeMatchOld && !chromeMatchNew && !edgeMatch) {
      setInstallStatus({ 
        type: 'error', 
        message: 'Invalid URL. Please paste a Chrome Web Store or Edge Add-ons URL.' 
      });
      setTimeout(() => setInstallStatus(null), 5000);
      return;
    }

    setIsInstalling(true);
    setInstallStatus({ type: 'info', message: 'Downloading extension...' });

    try {
      const result = await onInstallFromUrl(url);
      if (result.ok) {
        setInstallStatus({ type: 'success', message: result.message || 'Extension installed!' });
        if (result.requiresRestart) {
          setLocalPendingRestart(true);
        }
        setUrlInput('');
        setShowUrlInput(false);
        await onRefresh();
      } else {
        setInstallStatus({ type: 'error', message: result.error || 'Failed to install extension' });
      }
    } catch (error) {
      setInstallStatus({ type: 'error', message: `Failed to install: ${error}` });
    } finally {
      setIsInstalling(false);
      setTimeout(() => setInstallStatus(null), 5000);
    }
  }, [urlInput, onInstallFromUrl, onRefresh]);

  // Filter extensions based on current filter and search
  const filteredExtensions = extensions.filter(ext => {
    // Apply filter
    if (filterMode === 'enabled' && ext.enabled === false) return false;
    if (filterMode === 'disabled' && ext.enabled !== false) return false;
    if (filterMode === 'development' && ext.installType !== 'development') return false;
    
    // Apply search
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

  // Count by category
  const counts = {
    all: extensions.length,
    enabled: extensions.filter(e => e.enabled !== false).length,
    disabled: extensions.filter(e => e.enabled === false).length,
    development: extensions.filter(e => e.installType === 'development').length,
  };

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
        // Extension not found (maybe removed), go back
        setSelectedExtensionId(null);
    }
  }

  // Render List (Grid) View
  return (
    <div className="space-y-4">
      {/* Header - Chrome Style */}
      <div className="flex items-center justify-between border-b border-border pb-4">
        <h2 className="text-2xl font-semibold">Extensions</h2>
        <div className="flex items-center gap-2">
          {/* Developer mode toggle */}
          <div className="flex items-center gap-2 mr-4">
            <span className="text-sm text-muted-foreground">Developer mode</span>
            <button
              onClick={() => onToggleDeveloperMode?.(!developerMode)}
              className={cn(
                "relative inline-flex h-5 w-9 items-center rounded-full transition-colors",
                developerMode ? "bg-primary" : "bg-muted-foreground/30"
              )}
            >
              <span
                className={cn(
                  "inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform",
                  developerMode ? "translate-x-[18px]" : "translate-x-[2px]"
                )}
              />
            </button>
          </div>
        </div>
      </div>

      {/* Developer Mode Actions */}
      {developerMode && (
        <div className="flex items-center gap-2 p-3 bg-muted/30 rounded-lg border border-border">
          <button
            onClick={onOpenExtensionsDir}
            className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded border border-border hover:bg-background transition-colors"
          >
            <Icons.FolderOpen className="w-4 h-4" />
            Load unpacked
          </button>
          <button
            onClick={() => setShowUrlInput(!showUrlInput)}
            className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded border border-border hover:bg-background transition-colors"
          >
            <Icons.Link className="w-4 h-4" />
            Install from URL
          </button>
          <button
            onClick={onRefresh}
            className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded border border-border hover:bg-background transition-colors"
          >
            <Icons.RefreshCw className="w-4 h-4" />
            Update
          </button>
          {onOpenStore && (
            <button
              onClick={onOpenStore}
              className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded border border-border hover:bg-background transition-colors ml-auto"
            >
              <Icons.Store className="w-4 h-4" />
              Open Web Store
            </button>
          )}
        </div>
      )}

      {/* Install from URL (collapsible) */}
      {showUrlInput && (
        <div className="p-4 bg-card border border-border rounded-lg animate-in fade-in slide-in-from-top-2">
          <div className="flex items-center gap-2 mb-3">
            <Icons.Link className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm font-medium">Install from Chrome/Edge Web Store URL</span>
            <button
              onClick={() => setShowUrlInput(false)}
              className="ml-auto p-1 hover:bg-muted rounded"
            >
              <Icons.X className="w-4 h-4" />
            </button>
          </div>
          <div className="flex gap-2">
            <input
              type="text"
              value={urlInput}
              onChange={(e) => setUrlInput(e.target.value)}
              placeholder="https://chromewebstore.google.com/detail/..."
              className={cn(
                "flex-1 px-3 py-2 text-sm rounded-lg",
                "bg-background border border-border",
                "focus:outline-none focus:ring-2 focus:ring-primary/50",
                "placeholder:text-muted-foreground"
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
              {isInstalling ? (
                <span className="flex items-center gap-2">
                  <Icons.Loader2 className="w-4 h-4 animate-spin" />
                  Installing...
                </span>
              ) : (
                'Install'
              )}
            </button>
          </div>
        </div>
      )}

      {/* Pending Restart Banner */}
      {localPendingRestart && (
        <div className="p-3 bg-amber-500/10 border border-amber-500/30 rounded-lg flex items-center justify-between animate-in fade-in slide-in-from-top-2">
          <div className="flex items-center gap-3">
            <Icons.AlertTriangle className="w-5 h-5 text-amber-600" />
            <span className="text-sm font-medium text-amber-700 dark:text-amber-400">
              Restart required to apply changes
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
            {isRestarting ? 'Restarting...' : 'Restart Now'}
          </button>
        </div>
      )}

      {/* Install Status */}
      {installStatus && (
        <div className={cn(
          "p-3 rounded-lg text-sm flex items-center gap-3 animate-in fade-in slide-in-from-top-2",
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

      {/* Search and Filter Bar */}
      <div className="flex items-center gap-4">
        {/* Search */}
        <div className="relative flex-1 max-w-md">
          <Icons.Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search extensions"
            className={cn(
              "w-full pl-9 pr-3 py-2 text-sm rounded-lg",
              "bg-background border border-border",
              "focus:outline-none focus:ring-2 focus:ring-primary/50"
            )}
          />
        </div>

        {/* Filter Tabs */}
        <div className="flex items-center gap-1 bg-muted/30 rounded-lg p-1">
          {(['all', 'enabled', 'disabled', 'development'] as const).map((mode) => (
            <button
              key={mode}
              onClick={() => setFilterMode(mode)}
              className={cn(
                "px-3 py-1.5 text-xs font-medium rounded transition-colors capitalize",
                filterMode === mode
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              {mode} ({counts[mode]})
            </button>
          ))}
        </div>

        {/* View Mode Toggle */}
        <div className="flex items-center gap-1 bg-muted/30 rounded-lg p-1">
          <button
            onClick={() => setViewMode('grid')}
            className={cn(
              "p-1.5 rounded transition-colors",
              viewMode === 'grid' ? "bg-background shadow-sm" : "text-muted-foreground hover:text-foreground"
            )}
            title="Grid view"
          >
            <Icons.LayoutGrid className="w-4 h-4" />
          </button>
          <button
            onClick={() => setViewMode('list')}
            className={cn(
              "p-1.5 rounded transition-colors",
              viewMode === 'list' ? "bg-background shadow-sm" : "text-muted-foreground hover:text-foreground"
            )}
            title="List view"
          >
            <Icons.List className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Drop Zone (Full screen overlay when dragging) */}
      {isDragging && (
        <div
          className="fixed inset-0 z-50 bg-primary/10 border-4 border-dashed border-primary flex items-center justify-center backdrop-blur-sm"
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          onDragOver={(e) => e.preventDefault()}
        >
          <div className="bg-card p-8 rounded-xl shadow-2xl flex flex-col items-center gap-4 animate-in zoom-in duration-200">
            <div className="w-16 h-16 bg-primary/20 rounded-full flex items-center justify-center">
              <Icons.Download className="w-8 h-8 text-primary" />
            </div>
            <h3 className="text-xl font-bold">Drop to Install Extension</h3>
            <p className="text-muted-foreground">Release the folder to install it</p>
          </div>
        </div>
      )}

      {/* Extensions Grid/List */}
      {viewMode === 'grid' ? (
        <div 
          className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4"
          onDragOver={handleDragOver}
        >
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
          {filteredExtensions.length === 0 && (
            <div className="col-span-full py-16 text-center border-2 border-dashed border-border rounded-xl">
              <div className="w-12 h-12 bg-muted rounded-full flex items-center justify-center mx-auto mb-4">
                <Icons.Puzzle className="w-6 h-6 text-muted-foreground" />
              </div>
              {extensions.length === 0 ? (
                <>
                  <h3 className="text-lg font-semibold mb-2">No extensions installed</h3>
                  <p className="text-muted-foreground max-w-sm mx-auto mb-4">
                    Drag and drop an extension folder here, or use the buttons above to install.
                  </p>
                  {onOpenStore && (
                    <button
                      onClick={onOpenStore}
                      className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
                    >
                      Browse Web Store
                    </button>
                  )}
                </>
              ) : (
                <>
                  <h3 className="text-lg font-semibold mb-2">No matching extensions</h3>
                  <p className="text-muted-foreground">
                    Try adjusting your search or filter criteria.
                  </p>
                </>
              )}
            </div>
          )}
        </div>
      ) : (
        <div 
          className="space-y-2"
          onDragOver={handleDragOver}
        >
          {filteredExtensions.map((ext) => (
            <ExtensionListItem
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
          {filteredExtensions.length === 0 && (
            <div className="py-16 text-center border-2 border-dashed border-border rounded-xl">
              <div className="w-12 h-12 bg-muted rounded-full flex items-center justify-center mx-auto mb-4">
                <Icons.Puzzle className="w-6 h-6 text-muted-foreground" />
              </div>
              <h3 className="text-lg font-semibold mb-2">
                {extensions.length === 0 ? 'No extensions installed' : 'No matching extensions'}
              </h3>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// List view item component
function ExtensionListItem({
  extension,
  onDetails,
  onToggle,
  onRemove,
  onOpenSidePanel,
  onOpenPopup,
  onOpenOptions,
}: {
  extension: InstalledExtension;
  onDetails: () => void;
  onToggle?: (extension: InstalledExtension, enabled: boolean) => Promise<void>;
  onRemove?: (id: string) => Promise<void>;
  onOpenSidePanel?: (extension: InstalledExtension) => void;
  onOpenPopup?: (extension: InstalledExtension) => void;
  onOpenOptions?: (extension: InstalledExtension) => void;
}) {
  const isEnabled = extension.enabled !== false;
  
  return (
    <div className={cn(
      "flex items-center gap-4 p-4 bg-card border border-border rounded-lg hover:shadow-sm transition-shadow",
      !isEnabled && "opacity-60"
    )}>
      {/* Icon */}
      <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0">
        <Icons.Puzzle className="w-5 h-5 text-primary" />
      </div>
      
      {/* Info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <h3 className="font-medium truncate">{extension.name}</h3>
          <span className="text-xs text-muted-foreground font-mono">{extension.version}</span>
          {extension.installType === 'development' && (
            <span className="px-1.5 py-0.5 text-[10px] font-medium bg-amber-500/10 text-amber-600 rounded">
              DEV
            </span>
          )}
        </div>
        <p className="text-sm text-muted-foreground truncate">{extension.description}</p>
      </div>
      
      {/* Actions */}
      <div className="flex items-center gap-2 flex-shrink-0">
        {extension.hasSidePanel && onOpenSidePanel && (
          <button
            onClick={() => onOpenSidePanel(extension)}
            className="p-2 hover:bg-muted rounded-lg transition-colors"
            title="Open Side Panel"
          >
            <Icons.PanelRight className="w-4 h-4" />
          </button>
        )}
        {extension.hasPopup && onOpenPopup && (
          <button
            onClick={() => onOpenPopup(extension)}
            className="p-2 hover:bg-muted rounded-lg transition-colors"
            title="Open Popup"
          >
            <Icons.ExternalLink className="w-4 h-4" />
          </button>
        )}
        {extension.optionsUrl && onOpenOptions && (
          <button
            onClick={() => onOpenOptions(extension)}
            className="p-2 hover:bg-muted rounded-lg transition-colors"
            title="Options"
          >
            <Icons.Settings className="w-4 h-4" />
          </button>
        )}
        <button
          onClick={onDetails}
          className="p-2 hover:bg-muted rounded-lg transition-colors"
          title="Details"
        >
          <Icons.Info className="w-4 h-4" />
        </button>
        <button
          onClick={() => onRemove?.(extension.id)}
          className="p-2 hover:bg-red-50 hover:text-red-600 rounded-lg transition-colors"
          title="Remove"
        >
          <Icons.Trash2 className="w-4 h-4" />
        </button>
        {onToggle && (
          <button
            onClick={() => onToggle(extension, !isEnabled)}
            className={cn(
              "relative inline-flex h-5 w-9 items-center rounded-full transition-colors",
              isEnabled ? "bg-primary" : "bg-muted-foreground/30"
            )}
          >
            <span
              className={cn(
                "inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform",
                isEnabled ? "translate-x-[18px]" : "translate-x-[2px]"
              )}
            />
          </button>
        )}
      </div>
    </div>
  );
}
