import * as Icons from 'lucide-react';
import { cn } from '../lib/utils';
import type { InstalledExtension } from './ExtensionPanel';

interface ExtensionToolbarProps {
  extensions: InstalledExtension[];
  activeExtensionId: string | null;
  onExtensionClick: (id: string) => void;
  onManageExtensions: () => void;
  onOpenStore: () => void;
}

function ToolbarExtensionIcon({ extension, className }: { extension: InstalledExtension; className?: string }) {
  // Get the best icon (prefer smaller sizes for toolbar)
  const icons = extension.icons || [];
  const preferredSizes = [32, 48, 16, 64, 128];
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
        className={cn("object-contain rounded-sm", className)}
        onError={(e) => {
          // Fallback to puzzle icon on error
          e.currentTarget.style.display = 'none';
          e.currentTarget.nextElementSibling?.classList.remove('hidden');
        }}
      />
    );
  }
  
  return <Icons.Puzzle className={cn("", className)} />;
}

export function ExtensionToolbar({ 
  extensions, 
  activeExtensionId,
  onExtensionClick,
  onManageExtensions,
  onOpenStore,
}: ExtensionToolbarProps) {
  // Filter only enabled extensions with side panel
  const activeExtensions = extensions.filter(e => e.enabled && e.hasSidePanel);

  return (
    <div className="flex items-center gap-1 p-1 pl-2 bg-background border border-border rounded-full shadow-sm">
      {activeExtensions.length === 0 && (
        <span className="text-xs text-muted-foreground px-2">No extensions</span>
      )}
      {activeExtensions.map(ext => (
        <button
          key={ext.id}
          onClick={() => onExtensionClick(ext.id)}
          className={cn(
            "w-8 h-8 rounded-full flex items-center justify-center transition-colors relative group overflow-hidden",
            activeExtensionId === ext.id 
              ? "bg-primary/20 text-primary" 
              : "hover:bg-muted text-foreground/70 hover:text-foreground"
          )}
          title={ext.name}
        >
          <ToolbarExtensionIcon extension={ext} className="w-5 h-5" />
          <Icons.Puzzle className="w-4 h-4 hidden" />
          {activeExtensionId === ext.id && (
            <div className="absolute -bottom-0.5 left-1/2 -translate-x-1/2 w-1 h-1 bg-primary rounded-full" />
          )}
        </button>
      ))}
      
      <div className="w-px h-4 bg-border mx-1" />
      
      {/* Chrome Web Store shortcut */}
      <button
        onClick={onOpenStore}
        className="w-8 h-8 rounded-full flex items-center justify-center hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
        title="Chrome Web Store"
      >
        <Icons.Store className="w-4 h-4" />
      </button>
      
      {/* Manage Extensions */}
      <button
        onClick={onManageExtensions}
        className="w-8 h-8 rounded-full flex items-center justify-center hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
        title="Manage Extensions"
      >
        <Icons.Settings className="w-4 h-4" />
      </button>
    </div>
  );
}
