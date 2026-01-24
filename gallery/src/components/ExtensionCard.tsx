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
          e.currentTarget.style.display = 'none';
          e.currentTarget.nextElementSibling?.classList.remove('hidden');
        }}
      />
    );
  }
  
  return <Icons.Puzzle className={cn("text-primary", className)} />;
}

// Generate permission summary text based on extension permissions
function getPermissionSummary(extension: InstalledExtension): string {
  const permissions = extension.permissions || [];
  const hostPermissions = extension.hostPermissions || [];
  
  // Check for dangerous permissions
  const hasSiteAccess = hostPermissions.some(h => 
    h === '<all_urls>' || h.includes('*') || h.includes('http')
  );
  const hasTabAccess = permissions.includes('tabs') || permissions.includes('activeTab');
  const hasBackgroundAccess = permissions.includes('background') || 
    permissions.includes('webRequest') || 
    permissions.includes('webRequestBlocking');
  
  if (hasSiteAccess && hasBackgroundAccess) {
    return '此扩展程序可以读取和更改网站信息，并在后台运行';
  } else if (hasSiteAccess) {
    return '此扩展程序可以读取和更改网站信息';
  } else if (hasTabAccess) {
    return '此扩展程序可以读取您的浏览活动';
  } else if (hasBackgroundAccess) {
    return '此扩展程序可以在后台运行';
  } else {
    return '此扩展程序无法读取和更改网站信息，也无法在后台运行';
  }
}

export function ExtensionCard({
  extension,
  onDetails,
  onToggle,
  onRemove,
}: ExtensionCardProps) {
  const isEnabled = extension.enabled !== false;
  const permissionSummary = getPermissionSummary(extension);
  
  return (
    <div className={cn(
      "bg-card border border-border rounded-xl shadow-sm hover:shadow-md transition-all p-5 relative",
      !isEnabled && "grayscale"
    )}>
      {/* Header with Icon and Info */}
      <div className="flex items-start gap-4 mb-3">
        {/* Icon */}
        <div className={cn(
          "w-14 h-14 rounded-xl flex items-center justify-center flex-shrink-0 overflow-hidden border border-border/50",
          isEnabled ? "bg-gradient-to-br from-primary/5 to-primary/10" : "bg-muted"
        )}>
          <ExtensionIcon extension={extension} className="w-10 h-10" />
          <Icons.Puzzle className="w-9 h-9 text-muted-foreground hidden" />
        </div>
        
        {/* Name and Version */}
        <div className="flex-1 min-w-0">
          <div className="flex items-baseline gap-2">
            <h3 className={cn(
              "font-semibold leading-tight",
              isEnabled ? "text-foreground" : "text-muted-foreground"
            )} title={extension.name}>
              {extension.name}
            </h3>
            <span className="text-sm text-muted-foreground flex-shrink-0">{extension.version}</span>
          </div>
          {/* Description */}
          <p className={cn(
            "text-sm mt-1 line-clamp-2",
            isEnabled ? "text-muted-foreground" : "text-muted-foreground/70"
          )}>
            {extension.description || 'No description provided'}
          </p>
        </div>
      </div>

      {/* ID */}
      <div className="mb-4">
        <span className="text-xs text-muted-foreground">ID：</span>
        <span className="text-xs font-mono text-muted-foreground select-all ml-1">{extension.id}</span>
      </div>

      {/* Bottom Section - Chrome style */}
      <div className={cn(
        "rounded-2xl overflow-hidden",
        isEnabled ? "bg-muted/50" : "bg-muted"
      )}>
        {/* Permission Summary Banner */}
        <div className="px-4 py-2 text-xs text-muted-foreground">
          {permissionSummary}
        </div>
        
        {/* Action Bar */}
        <div className="flex items-center justify-between px-4 py-2 border-t border-border/30">
          {/* Left: Action buttons */}
          <div className="flex items-center gap-2">
            <button
              onClick={onDetails}
              className={cn(
                "px-4 py-1.5 text-sm font-medium rounded-full transition-colors",
                isEnabled 
                  ? "text-primary border border-primary hover:bg-primary/5"
                  : "text-muted-foreground border border-muted-foreground/50"
              )}
            >
              详情
            </button>
            <button
              onClick={() => onRemove?.(extension.id)}
              className={cn(
                "px-4 py-1.5 text-sm font-medium rounded-full transition-colors",
                isEnabled
                  ? "text-primary border border-primary hover:bg-primary/5"
                  : "text-muted-foreground border border-muted-foreground/50"
              )}
            >
              移除
            </button>
          </div>
          
          {/* Right: Toggle switch */}
          {onToggle && (
            <button
              onClick={() => onToggle(extension, !isEnabled)}
              className={cn(
                "relative inline-flex h-6 w-11 items-center rounded-full transition-colors flex-shrink-0",
                isEnabled ? "bg-primary" : "bg-muted-foreground/30"
              )}
              title={isEnabled ? "点击禁用" : "点击启用"}
            >
              <span
                className={cn(
                  "inline-block h-5 w-5 transform rounded-full bg-white shadow-sm transition-transform",
                  isEnabled ? "translate-x-[22px]" : "translate-x-[2px]"
                )}
              />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
