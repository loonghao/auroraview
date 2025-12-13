// Sample definitions for the AuroraView Gallery
// Note: Actual samples are loaded dynamically from Python backend
export interface Sample {
  id: string;
  title: string;
  category: string;
  description: string;
  icon: string;
  source_file: string;
  tags?: string[];
}

export interface Category {
  title: string;
  icon: string;
  description: string;
}

// Available tags for filtering
export const TAGS = [
  'beginner',
  'advanced',
  'window',
  'events',
  'qt',
  'standalone',
  'ui',
  'api',
] as const;

export type Tag = typeof TAGS[number];

// These are fallback/default values for development mode
// In production, data is loaded from Python backend via api.get_samples() and api.get_categories()
export const CATEGORIES: Record<string, Category> = {
  getting_started: {
    title: "Getting Started",
    icon: "rocket",
    description: "Quick start examples and basic usage patterns",
  },
  api_patterns: {
    title: "API Patterns",
    icon: "code",
    description: "Different ways to use the AuroraView API",
  },
  window_features: {
    title: "Window Features",
    icon: "layout",
    description: "Window styles, events, and customization",
  },
  desktop_features: {
    title: "Desktop Features",
    icon: "monitor",
    description: "File dialogs, shell commands, and system integration",
  },
  dcc_integration: {
    title: "DCC Integration",
    icon: "box",
    description: "Maya, Houdini, Blender, and other DCC apps",
  },
};

// Empty samples array - data is loaded dynamically
export const SAMPLES: Sample[] = [];

export function getSamplesByCategory(): Record<string, Sample[]> {
  const result: Record<string, Sample[]> = {};
  for (const sample of SAMPLES) {
    if (!result[sample.category]) {
      result[sample.category] = [];
    }
    result[sample.category].push(sample);
  }
  return result;
}

export function getSampleById(id: string): Sample | undefined {
  return SAMPLES.find((s) => s.id === id);
}
