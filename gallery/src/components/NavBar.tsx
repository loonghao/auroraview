import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

interface NavBarProps {
  onHome: () => void;
  onRefresh: () => void;
  onSettings: () => void;
}

export function NavBar({ onHome, onRefresh, onSettings }: NavBarProps) {
  return (
    <div className="sticky top-0 z-40 bg-background/80 backdrop-blur-sm border-b border-border">
      <div className="flex items-center gap-2 px-4 py-2">
        {/* Navigation buttons */}
        <div className="flex items-center gap-1">
          <button
            onClick={onHome}
            className={cn(
              "w-8 h-8 rounded-lg flex items-center justify-center transition-all",
              "text-muted-foreground hover:bg-accent hover:text-foreground"
            )}
            title="Home"
          >
            <Icons.Home className="w-4 h-4" />
          </button>
          <button
            onClick={onRefresh}
            className={cn(
              "w-8 h-8 rounded-lg flex items-center justify-center transition-all",
              "text-muted-foreground hover:bg-accent hover:text-foreground"
            )}
            title="Refresh"
          >
            <Icons.RotateCw className="w-4 h-4" />
          </button>
        </div>

        {/* Breadcrumb / Title */}
        <div className="flex-1 flex items-center gap-2 px-3">
          <span className="text-sm text-muted-foreground">AuroraView Gallery</span>
        </div>

        {/* Right actions */}
        <div className="flex items-center gap-1">
          <button
            onClick={onSettings}
            className={cn(
              "w-8 h-8 rounded-lg flex items-center justify-center transition-all",
              "text-muted-foreground hover:bg-accent hover:text-foreground"
            )}
            title="Settings"
          >
            <Icons.Settings className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
}
