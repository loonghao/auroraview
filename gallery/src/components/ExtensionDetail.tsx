import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';
import type { InstalledExtension } from './ExtensionPanel';

interface ExtensionDetailProps {
  extension: InstalledExtension;
  onBack: () => void;
  onToggle?: (extension: InstalledExtension, enabled: boolean) => Promise<void>;
  onRemove?: (id: string) => Promise<void>;
  onOpenSidePanel?: (extension: InstalledExtension) => void;
  onOpenOptions?: (extension: InstalledExtension) => void;
  onViewPermissions?: (extension: InstalledExtension) => void;
}

export function ExtensionDetail({
  extension,
  onBack,
  onToggle,
  onRemove,
  onOpenSidePanel,
  onOpenOptions,
  onViewPermissions,
}: ExtensionDetailProps) {
  const isEnabled = extension.enabled !== false;
  const isDev = extension.installType === 'development';
  
  // Format permissions for display
  const formatPermission = (perm: string) => {
    const permMap: Record<string, string> = {
      'storage': 'Store data locally',
      'tabs': 'Read your browsing activity',
      'activeTab': 'Access current tab',
      'notifications': 'Display notifications',
      'alarms': 'Schedule tasks',
      'bookmarks': 'Read and modify bookmarks',
      'history': 'Read browsing history',
      'downloads': 'Manage downloads',
      'cookies': 'Read and modify cookies',
      'contextMenus': 'Add context menu items',
      'webRequest': 'Intercept network requests',
      'scripting': 'Inject scripts into pages',
      'sidePanel': 'Display side panel',
      'clipboardRead': 'Read clipboard',
      'clipboardWrite': 'Write to clipboard',
      'geolocation': 'Access your location',
      'identity': 'Access your identity',
    };
    return permMap[perm] || perm;
  };

  return (
    <div className="animate-in fade-in slide-in-from-right-4 duration-200">
      {/* Header */}
      <div className="flex items-center gap-4 mb-6 pb-4 border-b border-border">
        <button
          onClick={onBack}
          className="p-2 rounded-full hover:bg-muted transition-colors"
          title="Back"
        >
          <Icons.ArrowLeft className="w-5 h-5" />
        </button>
        <div className="flex items-center gap-3 flex-1">
          <div className="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center">
            <Icons.Puzzle className="w-7 h-7 text-primary" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-xl font-semibold">{extension.name}</h2>
              {isDev && (
                <span className="px-2 py-0.5 text-xs font-medium bg-amber-500/10 text-amber-600 rounded">
                  Developer
                </span>
              )}
            </div>
            <p className="text-sm text-muted-foreground">{extension.version}</p>
          </div>
        </div>
        {onToggle && (
          <button
            onClick={() => onToggle(extension, !isEnabled)}
            className={cn(
              "relative inline-flex h-6 w-11 items-center rounded-full transition-colors",
              isEnabled ? "bg-primary" : "bg-muted-foreground/30"
            )}
          >
            <span
              className={cn(
                "inline-block h-4 w-4 transform rounded-full bg-white transition-transform",
                isEnabled ? "translate-x-6" : "translate-x-1"
              )}
            />
          </button>
        )}
      </div>

      <div className="space-y-6 max-w-3xl">
        {/* Quick Actions */}
        <div className="flex flex-wrap gap-2">
          {extension.hasSidePanel && onOpenSidePanel && (
            <button
              onClick={() => onOpenSidePanel(extension)}
              className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
            >
              <Icons.PanelRight className="w-4 h-4" />
              Open Side Panel
            </button>
          )}
          {extension.optionsUrl && onOpenOptions && (
            <button
              onClick={() => onOpenOptions(extension)}
              className="flex items-center gap-2 px-4 py-2 border border-border rounded-lg hover:bg-muted transition-colors"
            >
              <Icons.Settings className="w-4 h-4" />
              Extension Options
            </button>
          )}
          {extension.homepageUrl && (
            <a
              href={extension.homepageUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 px-4 py-2 border border-border rounded-lg hover:bg-muted transition-colors"
            >
              <Icons.Globe className="w-4 h-4" />
              Homepage
            </a>
          )}
        </div>

        {/* Description */}
        <div className="p-4 bg-card border border-border rounded-lg">
          <h3 className="text-sm font-medium text-muted-foreground mb-2">Description</h3>
          <p className="text-sm">{extension.description || 'No description provided'}</p>
        </div>

        {/* Info Grid */}
        <div className="bg-card border border-border rounded-lg overflow-hidden">
          <div className="divide-y divide-border">
            {/* Version */}
            <div className="p-4 grid grid-cols-[180px_1fr] gap-4">
              <div className="text-sm font-medium text-muted-foreground">Version</div>
              <div className="text-sm font-mono">{extension.version}</div>
            </div>

            {/* ID */}
            <div className="p-4 grid grid-cols-[180px_1fr] gap-4">
              <div className="text-sm font-medium text-muted-foreground">ID</div>
              <div className="text-sm font-mono select-all">{extension.id}</div>
            </div>

            {/* Install Type */}
            <div className="p-4 grid grid-cols-[180px_1fr] gap-4">
              <div className="text-sm font-medium text-muted-foreground">Source</div>
              <div className="text-sm capitalize">
                {extension.installType === 'development' ? (
                  <span className="flex items-center gap-2">
                    <Icons.Code className="w-4 h-4 text-amber-500" />
                    Loaded unpacked (Developer)
                  </span>
                ) : extension.installType === 'normal' ? (
                  <span className="flex items-center gap-2">
                    <Icons.Store className="w-4 h-4 text-blue-500" />
                    Web Store
                  </span>
                ) : (
                  extension.installType || 'Unknown'
                )}
              </div>
            </div>

            {/* Path */}
            <div className="p-4 grid grid-cols-[180px_1fr] gap-4">
              <div className="text-sm font-medium text-muted-foreground">Location</div>
              <div className="text-sm font-mono select-all break-all">{extension.path}</div>
            </div>

            {/* Side Panel */}
            {extension.hasSidePanel && (
              <div className="p-4 grid grid-cols-[180px_1fr] gap-4">
                <div className="text-sm font-medium text-muted-foreground">Side Panel</div>
                <div className="flex items-center gap-2">
                  <span className="text-sm font-mono">{extension.sidePanelPath || 'sidepanel.html'}</span>
                  {onOpenSidePanel && (
                    <button 
                      onClick={() => onOpenSidePanel(extension)}
                      className="text-xs text-primary hover:underline"
                    >
                      Open
                    </button>
                  )}
                </div>
              </div>
            )}

            {/* Popup */}
            {extension.hasPopup && (
              <div className="p-4 grid grid-cols-[180px_1fr] gap-4">
                <div className="text-sm font-medium text-muted-foreground">Popup</div>
                <div className="text-sm font-mono">{extension.popupPath || 'popup.html'}</div>
              </div>
            )}
          </div>
        </div>

        {/* Permissions */}
        {(extension.permissions?.length || extension.hostPermissions?.length) && (
          <div className="bg-card border border-border rounded-lg overflow-hidden">
            <div className="p-4 border-b border-border flex items-center justify-between">
              <h3 className="font-medium">Permissions</h3>
              {onViewPermissions && (
                <button
                  onClick={() => onViewPermissions(extension)}
                  className="text-xs text-primary hover:underline"
                >
                  View all
                </button>
              )}
            </div>
            <div className="p-4 space-y-3">
              {extension.permissions?.map(perm => (
                <div key={perm} className="flex items-start gap-3">
                  <Icons.CheckCircle className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                  <div>
                    <div className="text-sm">{formatPermission(perm)}</div>
                    <div className="text-xs text-muted-foreground font-mono">{perm}</div>
                  </div>
                </div>
              ))}
              {extension.hostPermissions?.map(host => (
                <div key={host} className="flex items-start gap-3">
                  <Icons.Globe className="w-4 h-4 text-blue-500 mt-0.5 flex-shrink-0" />
                  <div>
                    <div className="text-sm">Access to {host === '<all_urls>' ? 'all websites' : host}</div>
                    <div className="text-xs text-muted-foreground font-mono">{host}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Danger Zone */}
        <div className="pt-4 border-t border-border">
          <h3 className="text-sm font-medium text-muted-foreground mb-3">Danger Zone</h3>
          <div className="flex flex-wrap gap-2">
            {onRemove && (
              <button
                onClick={() => onRemove(extension.id)}
                className="flex items-center gap-2 px-4 py-2 border border-red-200 rounded-lg hover:bg-red-50 text-red-600 transition-colors"
              >
                <Icons.Trash2 className="w-4 h-4" />
                Remove extension
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
