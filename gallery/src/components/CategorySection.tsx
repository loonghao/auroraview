import type { Category, Sample } from '../hooks/useAuroraView';
import { SampleCard } from './SampleCard';

interface CategorySectionProps {
  categoryId: string;
  category: Category;
  samples: Sample[];
  onViewSource: (sampleId: string) => void;
  onRun: (sampleId: string) => void;
}

export function CategorySection({
  categoryId,
  category,
  samples,
  onViewSource,
  onRun,
}: CategorySectionProps) {
  return (
    <section id={`category-${categoryId}`} className="mb-10">
      <div className="mb-4">
        <h2 className="text-lg font-semibold mb-1">{category.title}</h2>
        <p className="text-sm text-muted-foreground">{category.description}</p>
      </div>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
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
