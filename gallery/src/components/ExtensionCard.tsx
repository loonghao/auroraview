import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';
import type { InstalledExtension } from './ExtensionPanel';

interface ExtensionCardProps {
  extension: InstalledExtension;
  onDetails: () => void;
  onToggle?: (extension: InstalledExtension, enabled: boolean) => Promise<void>;
  onRemove?: (id: string) => Promise<void>;
  onOpenSidePanel?: (extension: InstalledExtension) => void;
  onOpenPopup?: (extension: InstalledExtension) => void;
  onOpenOptions?: (extension: InstalledExtension) => void;
}

function ExtensionIcon({ extension, className }: { extension: InstalledExtension; className?: string }) {
  // Get the best icon (prefer larger sizes like 48 or 128)
  const icons = extension.icons || [];
  const preferredSizes = [48, 128, 32, 64, 16];
  let iconUrl: string | null = null;
  
  for (const size of preferredSizes) {
    const icon = icons.find(i => i.size === size);
    if (icon) {
      iconUrl = icon.url;
      break;
    }
  }
  
  // Fallback to first available icon
  if (!iconUrl && icons.length > 0) {
    iconUrl = icons[0].url;
  }
  
  if (iconUrl) {
    return (
      <img 
        src={iconUrl} 
        alt={extension.name}
        className={cn("object-contain", className)}
        onError={(e) => {
          // Fallback to puzzle icon on error
          e.currentTarget.style.display = 'none';
          e.currentTarget.nextElementSibling?.classList.remove('hidden');
        }}
      />
    );
  }
  
  return <Icons.Puzzle className={cn("text-primary", className)} />;
}

export function ExtensionCard({
  extension,
  onDetails,
  onToggle,
  onRemove,
  onOpenSidePanel,
  onOpenPopup,
  onOpenOptions,
}: ExtensionCardProps) {
  const isEnabled = extension.enabled !== false;
  const isDev = extension.installType === 'development';
  
  return (
    <div className={cn(
      "bg-card border border-border rounded-lg shadow-sm hover:shadow-md transition-all flex flex-col h-full overflow-hidden group",
      !isEnabled && "opacity-60"
    )}>
      <div className="p-4 flex gap-4">
        {/* Icon */}
        <div className="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0 overflow-hidden">
          <ExtensionIcon extension={extension} className="w-8 h-8" />
          <Icons.Puzzle className="w-7 h-7 text-primary hidden" />
        </div>
        
        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-start justify-between gap-2">
            <div className="min-w-0">
              <div className="flex items-center gap-2">
                <h3 className="font-semibold text-foreground truncate" title={extension.name}>
                  {extension.name}
                </h3>
                {isDev && (
                  <span className="px-1.5 py-0.5 text-[10px] font-medium bg-amber-500/10 text-amber-600 rounded flex-shrink-0">
                    DEV
                  </span>
                )}
              </div>
              <p className="text-xs text-muted-foreground font-mono">
                {extension.version}
              </p>
            </div>
            {onToggle && (
              <button
                onClick={() => onToggle(extension, !isEnabled)}
                className={cn(
                  "relative inline-flex h-5 w-9 items-center rounded-full transition-colors flex-shrink-0",
                  isEnabled ? "bg-primary" : "bg-muted-foreground/30"
                )}
                title={isEnabled ? "Disable" : "Enable"}
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
          <p className="text-sm text-muted-foreground mt-2 line-clamp-2 min-h-[2.5rem]">
            {extension.description || 'No description provided'}
          </p>
        </div>
      </div>

      {/* Footer Actions */}
      <div className="mt-auto px-3 py-2.5 bg-muted/30 border-t border-border flex items-center justify-between gap-2">
        <div className="flex items-center gap-1 flex-wrap">
          <button
            onClick={onDetails}
            className="px-2.5 py-1 text-xs font-medium border border-border rounded hover:bg-background transition-colors"
          >
            Details
          </button>
          {extension.hasSidePanel && onOpenSidePanel && (
            <button
              onClick={() => onOpenSidePanel(extension)}
              className="px-2.5 py-1 text-xs font-medium border border-primary/50 rounded hover:bg-primary/10 transition-colors text-primary flex items-center gap-1"
              title="Open Side Panel"
            >
              <Icons.PanelRight className="w-3 h-3" />
              Panel
            </button>
          )}
          {extension.hasPopup && onOpenPopup && (
            <button
              onClick={() => onOpenPopup(extension)}
              className="px-2.5 py-1 text-xs font-medium border border-primary/50 rounded hover:bg-primary/10 transition-colors text-primary flex items-center gap-1"
              title="Open Popup"
            >
              <Icons.ExternalLink className="w-3 h-3" />
              Popup
            </button>
          )}
          {extension.optionsUrl && onOpenOptions && (
            <button
              onClick={() => onOpenOptions(extension)}
              className="px-2.5 py-1 text-xs font-medium border border-border rounded hover:bg-background transition-colors flex items-center gap-1"
              title="Options"
            >
              <Icons.Settings className="w-3 h-3" />
            </button>
          )}
        </div>
        
        <button
          onClick={() => onRemove?.(extension.id)}
          className="p-1.5 text-muted-foreground hover:text-red-600 hover:bg-red-50 rounded transition-colors"
          title="Remove extension"
        >
          <Icons.Trash2 className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
