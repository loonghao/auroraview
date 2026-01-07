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
  code: Icons.Code,
  terminal: Icons.Terminal,
  settings: Icons.Settings,
  zap: Icons.Zap,
  globe: Icons.Globe,
};

const iconGradients: Record<string, string> = {
  'wand-2': 'from-violet-500 to-purple-600',
  link: 'from-blue-500 to-cyan-500',
  bell: 'from-purple-500 to-pink-500',
  monitor: 'from-cyan-500 to-blue-500',
  layers: 'from-indigo-500 to-violet-500',
  circle: 'from-pink-500 to-rose-500',
  inbox: 'from-orange-500 to-amber-500',
  menu: 'from-teal-500 to-emerald-500',
  folder: 'from-amber-500 to-yellow-500',
  image: 'from-emerald-500 to-green-500',
  box: 'from-rose-500 to-red-500',
  palette: 'from-violet-500 to-fuchsia-500',
  list: 'from-sky-500 to-blue-500',
  code: 'from-green-500 to-emerald-500',
  terminal: 'from-gray-600 to-gray-700',
  settings: 'from-slate-500 to-gray-600',
  zap: 'from-yellow-500 to-orange-500',
  globe: 'from-blue-500 to-indigo-500',
};

const tagStyles: Record<Tag, { bg: string; text: string; border: string }> = {
  beginner: { bg: 'bg-emerald-500/10', text: 'text-emerald-400', border: 'border-emerald-500/20' },
  advanced: { bg: 'bg-orange-500/10', text: 'text-orange-400', border: 'border-orange-500/20' },
  window: { bg: 'bg-blue-500/10', text: 'text-blue-400', border: 'border-blue-500/20' },
  events: { bg: 'bg-purple-500/10', text: 'text-purple-400', border: 'border-purple-500/20' },
  qt: { bg: 'bg-cyan-500/10', text: 'text-cyan-400', border: 'border-cyan-500/20' },
  standalone: { bg: 'bg-pink-500/10', text: 'text-pink-400', border: 'border-pink-500/20' },
  ui: { bg: 'bg-yellow-500/10', text: 'text-yellow-400', border: 'border-yellow-500/20' },
  api: { bg: 'bg-indigo-500/10', text: 'text-indigo-400', border: 'border-indigo-500/20' },
};

export function SampleCard({ sample, onViewSource, onRun }: SampleCardProps) {
  const Icon = iconMap[sample.icon] || Icons.Circle;
  const gradient = iconGradients[sample.icon] || 'from-gray-500 to-gray-600';

  return (
    <div 
      className={cn(
        "relative bg-card border border-border rounded-xl p-4",
        "card-shadow gradient-border",
        "flex items-center gap-4 transition-all cursor-pointer group",
        "hover:border-primary/40"
      )}
      onClick={() => onViewSource(sample.id)}
    >
      {/* Icon with gradient */}
      <div className={cn(
        "w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0",
        "bg-gradient-to-br shadow-lg",
        gradient
      )}>
        <Icon className="w-5 h-5 text-white" />
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="font-semibold text-sm mb-0.5 text-foreground group-hover:text-primary transition-colors">
          {sample.title}
        </div>
        <div className="text-xs text-muted-foreground line-clamp-1">
          {sample.description}
        </div>
        {sample.tags && sample.tags.length > 0 && (
          <div className="flex flex-wrap gap-1.5 mt-2">
            {sample.tags.slice(0, 3).map((tag) => {
              const style = tagStyles[tag as Tag] || { bg: 'bg-muted', text: 'text-muted-foreground', border: 'border-border' };
              return (
                <span
                  key={tag}
                  className={cn(
                    "px-2 py-0.5 text-[10px] rounded-md font-medium border",
                    style.bg, style.text, style.border
                  )}
                >
                  {tag}
                </span>
              );
            })}
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex gap-2 opacity-0 group-hover:opacity-100 transition-all duration-200">
        <button
          onClick={(e) => { e.stopPropagation(); onViewSource(sample.id); }}
          className={cn(
            "w-9 h-9 rounded-lg flex items-center justify-center transition-all",
            "bg-muted/50 text-muted-foreground",
            "hover:bg-accent hover:text-foreground hover:scale-105"
          )}
          title="View Source"
        >
          <Icons.Code className="w-4 h-4" />
        </button>
        <button
          onClick={(e) => { e.stopPropagation(); onRun(sample.id); }}
          className={cn(
            "w-9 h-9 rounded-lg flex items-center justify-center transition-all",
            "bg-primary/20 text-primary",
            "hover:bg-primary hover:text-primary-foreground hover:scale-105",
            "hover:shadow-lg hover:shadow-primary/25"
          )}
          title="Run Demo"
        >
          <Icons.Play className="w-4 h-4 fill-current" />
        </button>
      </div>
    </div>
  );
}
