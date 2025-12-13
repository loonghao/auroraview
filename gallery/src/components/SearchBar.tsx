import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
}

export function SearchBar({ value, onChange }: SearchBarProps) {
  return (
    <div className="relative mb-4">
      <Icons.Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
      <input
        type="text"
        placeholder="Search samples..."
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className={cn(
          "w-full h-10 pl-10 pr-10 rounded-lg",
          "bg-card border border-border",
          "text-sm placeholder:text-muted-foreground",
          "focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary/50",
          "transition-all"
        )}
      />
      {value && (
        <button
          onClick={() => onChange('')}
          className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
        >
          <Icons.X className="w-4 h-4" />
        </button>
      )}
    </div>
  );
}
