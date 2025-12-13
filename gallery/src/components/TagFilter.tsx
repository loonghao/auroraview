import { cn } from '../lib/utils';
import { TAGS, type Tag } from '../data/samples';
import * as Icons from 'lucide-react';

interface TagFilterProps {
  selectedTags: Set<Tag>;
  onTagToggle: (tag: Tag) => void;
  onClear: () => void;
}

const tagStyles: Record<Tag, string> = {
  beginner: 'bg-green-500/10 text-green-600 border-green-500/20 hover:bg-green-500/20',
  advanced: 'bg-orange-500/10 text-orange-600 border-orange-500/20 hover:bg-orange-500/20',
  window: 'bg-blue-500/10 text-blue-600 border-blue-500/20 hover:bg-blue-500/20',
  events: 'bg-purple-500/10 text-purple-600 border-purple-500/20 hover:bg-purple-500/20',
  qt: 'bg-cyan-500/10 text-cyan-600 border-cyan-500/20 hover:bg-cyan-500/20',
  standalone: 'bg-pink-500/10 text-pink-600 border-pink-500/20 hover:bg-pink-500/20',
  ui: 'bg-yellow-500/10 text-yellow-700 border-yellow-500/20 hover:bg-yellow-500/20',
  api: 'bg-indigo-500/10 text-indigo-600 border-indigo-500/20 hover:bg-indigo-500/20',
};

export function TagFilter({ selectedTags, onTagToggle, onClear }: TagFilterProps) {
  return (
    <div className="flex flex-wrap items-center gap-2 mb-6">
      <span className="text-xs text-muted-foreground mr-1">Filter:</span>
      {TAGS.map((tag) => (
        <button
          key={tag}
          onClick={() => onTagToggle(tag)}
          className={cn(
            "px-2.5 py-1 text-xs rounded-md border transition-all font-medium",
            selectedTags.has(tag)
              ? tagStyles[tag].replace('hover:', '') + ' ring-1 ring-offset-1 ring-current/20'
              : 'bg-card border-border text-muted-foreground hover:text-foreground hover:bg-accent'
          )}
        >
          {tag}
        </button>
      ))}
      {selectedTags.size > 0 && (
        <button
          onClick={onClear}
          className="px-2 py-1 text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
        >
          <Icons.X className="w-3 h-3" />
          Clear
        </button>
      )}
    </div>
  );
}
