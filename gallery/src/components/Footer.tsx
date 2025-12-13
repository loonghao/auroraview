import * as Icons from 'lucide-react';

export function Footer() {
  return (
    <footer className="mt-12 pt-6 border-t border-border">
      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <div className="flex items-center gap-4">
          <span>AuroraView Gallery v0.1.0</span>
          <a
            href="https://github.com/loonghao/auroraview"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-1 hover:text-foreground transition-colors"
          >
            <Icons.Github className="w-3 h-3" />
            GitHub
          </a>
          <a
            href="https://github.com/loonghao/auroraview/issues"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-foreground transition-colors"
          >
            Report Issue
          </a>
        </div>
        <div>
          MIT License
        </div>
      </div>
    </footer>
  );
}
