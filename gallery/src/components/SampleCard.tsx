import { cn } from '../lib/utils';
import type { Sample } from '../hooks/useAuroraView';
import type { Tag } from '../data/samples';
import * as Icons from 'lucide-react';

interface SampleCardProps {
  sample: Sample;
  onViewSource: (sampleId: string) => void;
  onRun: (sampleId: string) => void;
}

const iconMap: Record<string, React.ComponentType<{ className?: string }>> = {
  'wand-2': Icons.Wand2,
  link: Icons.Link,
  bell: Icons.Bell,
  monitor: Icons.Monitor,
  layers: Icons.Layers,
  circle: Icons.Circle,
  inbox: Icons.Inbox,
  menu: Icons.Menu,
  folder: Icons.Folder,
  image: Icons.Image,
  box: Icons.Box,
  palette: Icons.Palette,
  list: Icons.List,
};

const iconColors: Record<string, string> = {
  'wand-2': 'bg-primary text-white',
  link: 'bg-blue-500 text-white',
  bell: 'bg-purple-500 text-white',
  monitor: 'bg-cyan-500 text-white',
  layers: 'bg-indigo-500 text-white',
  circle: 'bg-pink-500 text-white',
  inbox: 'bg-orange-500 text-white',
  menu: 'bg-teal-500 text-white',
  folder: 'bg-amber-500 text-white',
  image: 'bg-emerald-500 text-white',
  box: 'bg-rose-500 text-white',
  palette: 'bg-violet-500 text-white',
  list: 'bg-sky-500 text-white',
};

const tagColors: Record<Tag, string> = {
  beginner: 'bg-green-500/10 text-green-600',
  advanced: 'bg-orange-500/10 text-orange-600',
  window: 'bg-blue-500/10 text-blue-600',
  events: 'bg-purple-500/10 text-purple-600',
  qt: 'bg-cyan-500/10 text-cyan-600',
  standalone: 'bg-pink-500/10 text-pink-600',
  ui: 'bg-yellow-500/10 text-yellow-700',
  api: 'bg-indigo-500/10 text-indigo-600',
};

export function SampleCard({ sample, onViewSource, onRun }: SampleCardProps) {
  const Icon = iconMap[sample.icon] || Icons.Circle;
  const iconColor = iconColors[sample.icon] || 'bg-gray-500 text-white';

  return (
    <div className={cn(
      "bg-card border border-border rounded-xl p-4",
      "fluent-card flex items-center gap-4 transition-all cursor-pointer group",
      "hover:border-primary/30"
    )}
    onClick={() => onViewSource(sample.id)}
    >
      {/* Icon */}
      <div className={cn(
        "w-11 h-11 rounded-xl flex items-center justify-center flex-shrink-0",
        iconColor
      )}>
        <Icon className="w-5 h-5" />
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="font-semibold text-sm mb-0.5">{sample.title}</div>
        <div className="text-xs text-muted-foreground line-clamp-1">{sample.description}</div>
        {sample.tags && sample.tags.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-1.5">
            {sample.tags.slice(0, 3).map((tag) => (
              <span
                key={tag}
                className={cn(
                  "px-1.5 py-0.5 text-[10px] rounded font-medium",
                  tagColors[tag as Tag] || "bg-muted text-muted-foreground"
                )}
              >
                {tag}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
        <button
          onClick={(e) => { e.stopPropagation(); onViewSource(sample.id); }}
          className={cn(
            "w-8 h-8 rounded-lg flex items-center justify-center transition-all",
            "text-muted-foreground hover:bg-accent hover:text-foreground"
          )}
          title="View Source"
        >
          <Icons.Code className="w-4 h-4" />
        </button>
        <button
          onClick={(e) => { e.stopPropagation(); onRun(sample.id); }}
          className={cn(
            "w-8 h-8 rounded-lg flex items-center justify-center transition-all",
            "bg-primary/10 text-primary hover:bg-primary/20"
          )}
          title="Run Demo"
        >
          <Icons.Play className="w-4 h-4 fill-current" />
        </button>
      </div>
    </div>
  );
}
