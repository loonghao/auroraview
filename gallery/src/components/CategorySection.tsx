import { cn } from '../lib/utils';
import type { Category, Sample } from '../hooks/useAuroraView';
import { SampleCard } from './SampleCard';
import * as Icons from 'lucide-react';

interface CategorySectionProps {
  categoryId: string;
  category: Category;
  samples: Sample[];
  onViewSource: (sampleId: string) => void;
  onRun: (sampleId: string) => void;
}

const categoryIcons: Record<string, React.ComponentType<{ className?: string }>> = {
  getting_started: Icons.Rocket,
  window_features: Icons.Layout,
  ui_patterns: Icons.Palette,
  api_patterns: Icons.Code,
  advanced: Icons.Zap,
};

const categoryGradients: Record<string, string> = {
  getting_started: 'from-violet-500 to-purple-600',
  window_features: 'from-blue-500 to-cyan-500',
  ui_patterns: 'from-pink-500 to-rose-500',
  api_patterns: 'from-orange-500 to-amber-500',
  advanced: 'from-emerald-500 to-green-500',
};

export function CategorySection({
  categoryId,
  category,
  samples,
  onViewSource,
  onRun,
}: CategorySectionProps) {
  const Icon = categoryIcons[categoryId] || Icons.Circle;
  const gradient = categoryGradients[categoryId] || 'from-gray-500 to-gray-600';

  return (
    <section id={`category-${categoryId}`} className="mb-12">
      <div className="mb-5 flex items-center gap-4">
        <div className={cn(
          "w-10 h-10 rounded-xl flex items-center justify-center",
          "bg-gradient-to-br shadow-lg",
          gradient
        )}>
          <Icon className="w-5 h-5 text-white" />
        </div>
        <div>
          <h2 className="text-xl font-bold text-foreground">{category.title}</h2>
          <p className="text-sm text-muted-foreground">{category.description}</p>
        </div>
      </div>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {samples.map((sample) => (
          <SampleCard
            key={sample.id}
            sample={sample}
            onViewSource={onViewSource}
            onRun={onRun}
          />
        ))}
      </div>
    </section>
  );
}
