import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

export type RunMode = 'external' | 'console';
export type LinkMode = 'browser' | 'webview';

export interface Settings {
  runMode: RunMode;
  linkMode: LinkMode;
}

interface SettingsModalProps {
  isOpen: boolean;
  settings: Settings;
  onClose: () => void;
  onSave: (settings: Settings) => void;
}

export function SettingsModal({ isOpen, settings, onClose, onSave }: SettingsModalProps) {
  if (!isOpen) return null;

  const handleRunModeChange = (mode: RunMode) => {
    onSave({ ...settings, runMode: mode });
  };

  const handleLinkModeChange = (mode: LinkMode) => {
    onSave({ ...settings, linkMode: mode });
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
