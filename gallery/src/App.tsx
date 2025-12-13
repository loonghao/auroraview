import { useState, useCallback, useMemo, useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { CategorySection } from './components/CategorySection';
import { QuickLinks } from './components/QuickLinks';
import { SourceModal } from './components/SourceModal';
import { SettingsModal, type Settings } from './components/SettingsModal';
import { Toast } from './components/Toast';
import { SearchBar } from './components/SearchBar';
import { Footer } from './components/Footer';
import { SampleCard } from './components/SampleCard';
import { TagFilter } from './components/TagFilter';
import { ProcessConsole } from './components/ProcessConsole';
import { type Tag } from './data/samples';
import { useAuroraView, type Sample, type Category } from './hooks/useAuroraView';

const SETTINGS_KEY = 'auroraview-gallery-settings';

function loadSettings(): Settings {
  try {
    const saved = localStorage.getItem(SETTINGS_KEY);
    if (saved) {
      return JSON.parse(saved);
    }
  } catch {
    // Ignore parse errors
  }
  return { runMode: 'external', linkMode: 'browser' };
}

function saveSettings(settings: Settings) {
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
  } catch {
    // Ignore storage errors
  }
}

function App() {
  // Handle hash navigation - use initializer function for useState
  const initialCategory = (() => {
    if (typeof window !== 'undefined') {
      const hash = window.location.hash.slice(1);
      if (hash.startsWith('category-')) {
        return hash.replace('category-', '');
      }
    }
    return null;
  })();

  const [activeCategory, setActiveCategory] = useState<string | null>(initialCategory);
  const [modalOpen, setModalOpen] = useState(false);
  const [modalTitle, setModalTitle] = useState('');
  const [modalSource, setModalSource] = useState('');
  const [currentSampleId, setCurrentSampleId] = useState<string | null>(null);
  const [toast, setToast] = useState({ visible: false, message: '', type: 'success' as 'success' | 'error' });
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTags, setSelectedTags] = useState<Set<Tag>>(new Set());
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [settings, setSettings] = useState<Settings>(loadSettings);
  const [consoleOpen, setConsoleOpen] = useState(false);

  // Dynamic samples and categories from Python backend
  const [samples, setSamples] = useState<Sample[]>([]);
  const [categories, setCategories] = useState<Record<string, Category>>({});
  const [dataLoaded, setDataLoaded] = useState(false);

  const { isReady, getSource, runSample, getSamples, getCategories, openUrl, openInWebView, killProcess } = useAuroraView();

  // Load samples and categories from Python backend
  useEffect(() => {
    if (isReady && !dataLoaded) {
      Promise.all([getSamples(), getCategories()])
        .then(([samplesData, categoriesData]) => {
          setSamples(samplesData);
          setCategories(categoriesData);
          setDataLoaded(true);
        })
        .catch((err) => {
          console.error('Failed to load samples/categories:', err);
        });
    }
  }, [isReady, dataLoaded, getSamples, getCategories]);

  // Save settings when changed
  useEffect(() => {
    saveSettings(settings);
  }, [settings]);

  // Group samples by category
  const samplesByCategory = useMemo(() => {
    const result: Record<string, Sample[]> = {};
    for (const sample of samples) {
      if (!result[sample.category]) {
        result[sample.category] = [];
      }
      result[sample.category].push(sample);
    }
    return result;
  }, [samples]);

  // Get sample by ID
  const getSampleById = useCallback((id: string): Sample | undefined => {
    return samples.find((s) => s.id === id);
  }, [samples]);

  // Filter samples based on search query and tags
  const filteredSamples = useMemo(() => {
    const hasSearch = searchQuery.trim().length > 0;
    const hasTags = selectedTags.size > 0;

    if (!hasSearch && !hasTags) return null;

    const query = searchQuery.toLowerCase();
    return samples.filter((sample: Sample) => {
      // Search filter
      const matchesSearch = !hasSearch || (
        sample.title.toLowerCase().includes(query) ||
        sample.description.toLowerCase().includes(query) ||
        sample.source_file.toLowerCase().includes(query)
      );

      // Tag filter (sample must have ALL selected tags)
      const matchesTags = !hasTags || (
        sample.tags && Array.from(selectedTags).every(tag => sample.tags?.includes(tag))
      );

      return matchesSearch && matchesTags;
    });
  }, [samples, searchQuery, selectedTags]);

  const handleTagToggle = useCallback((tag: Tag) => {
    setSelectedTags(prev => {
      const next = new Set(prev);
      if (next.has(tag)) {
        next.delete(tag);
      } else {
        next.add(tag);
      }
      return next;
    });
  }, []);

  const handleClearTags = useCallback(() => {
    setSelectedTags(new Set());
  }, []);

  const showToast = useCallback((message: string, type: 'success' | 'error' = 'success') => {
    setToast({ visible: true, message, type });
  }, []);

  const hideToast = useCallback(() => {
    setToast(prev => ({ ...prev, visible: false }));
  }, []);

  const handleCategoryClick = useCallback((categoryId: string) => {
    setActiveCategory(categoryId);
    document.getElementById(`category-${categoryId}`)?.scrollIntoView({ behavior: 'smooth' });
  }, []);

  const handleViewSource = useCallback(async (sampleId: string) => {
    const sample = getSampleById(sampleId);
    if (!sample) return;

    setCurrentSampleId(sampleId);
    setModalTitle(`${sample.title} - Source Code`);

    if (isReady) {
      try {
        const source = await getSource(sampleId);
        setModalSource(source);
      } catch {
        setModalSource(`# Failed to load source for: ${sampleId}`);
      }
    } else {
      // Fallback for development without AuroraView
      setModalSource(`# Source code for: ${sample.source_file}\n# (AuroraView bridge not available)`);
    }
    setModalOpen(true);
  }, [isReady, getSource, getSampleById]);

  const handleRun = useCallback(async (sampleId: string) => {
    if (isReady) {
      try {
        const showConsole = settings.runMode === 'console';
        const result = await runSample(sampleId, { showConsole });
        if (result.ok) {
          const modeText = showConsole ? ' (with console)' : '';
          showToast(`Demo started${modeText}`);
          // Open process console to show output
          if (!showConsole) {
            setConsoleOpen(true);
          }
        } else {
          showToast(result.error || 'Failed to run demo', 'error');
        }
      } catch {
        showToast('Failed to run demo', 'error');
      }
    } else {
      showToast('AuroraView bridge not available', 'error');
    }
  }, [isReady, runSample, settings.runMode, showToast]);

  const handleKillProcess = useCallback(async (pid: number) => {
    if (isReady) {
      try {
        const result = await killProcess(pid);
        if (result.ok) {
          showToast(`Process ${pid} terminated`);
        } else {
          showToast(result.error || 'Failed to kill process', 'error');
        }
      } catch {
        showToast('Failed to kill process', 'error');
      }
    }
  }, [isReady, killProcess, showToast]);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(modalSource);
      showToast('Copied to clipboard!');
    } catch {
      showToast('Failed to copy', 'error');
    }
  }, [modalSource, showToast]);

  const handleOpenUrl = useCallback(async (url: string) => {
    if (isReady) {
      try {
        await openUrl(url);
      } catch {
        // Fallback to window.open
        window.open(url, '_blank');
      }
    } else {
      // Fallback for development
      window.open(url, '_blank');
    }
  }, [isReady, openUrl]);

  const handleOpenInWebView = useCallback(async (url: string, title?: string) => {
    if (isReady) {
      try {
        await openInWebView(url, title);
      } catch {
        // Fallback to window.open
        window.open(url, '_blank');
      }
    } else {
      // Fallback for development
      window.open(url, '_blank');
    }
  }, [isReady, openInWebView]);

  // Unified link handler based on settings
  const handleOpenLink = useCallback(async (url: string, title?: string) => {
    if (settings.linkMode === 'webview') {
      await handleOpenInWebView(url, title);
    } else {
      await handleOpenUrl(url);
    }
  }, [settings.linkMode, handleOpenInWebView, handleOpenUrl]);

  const handleSettingsClick = useCallback(() => {
    setSettingsOpen(true);
  }, []);

  const handleSettingsSave = useCallback((newSettings: Settings) => {
    setSettings(newSettings);
  }, []);

  // Show loading state while data is being fetched
  if (isReady && !dataLoaded) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="text-muted-foreground">Loading samples...</div>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen">
      <Sidebar
        activeCategory={activeCategory}
        onCategoryClick={handleCategoryClick}
        onSettingsClick={handleSettingsClick}
        onOpenLink={handleOpenLink}
        onConsoleClick={() => setConsoleOpen(!consoleOpen)}
        consoleOpen={consoleOpen}
      />

      <main className="flex-1 ml-14 p-8 max-w-5xl">
        {/* Header */}
        <header className="mb-8">
          <h1 className="text-4xl font-bold mb-2 text-foreground">
            AuroraView Gallery
          </h1>
          <p className="text-muted-foreground">
            Explore all features and components with live demos and source code
          </p>
        </header>

        {/* Quick Links */}
        <QuickLinks onCategoryClick={handleCategoryClick} onOpenLink={handleOpenLink} />

        {/* Search and Filter */}
        <SearchBar value={searchQuery} onChange={setSearchQuery} />
        <TagFilter selectedTags={selectedTags} onTagToggle={handleTagToggle} onClear={handleClearTags} />

        {filteredSamples ? (
          // Show search/filter results
          <section className="mb-10">
            <div className="mb-4">
              <h2 className="text-lg font-semibold mb-1">
                {searchQuery ? 'Search Results' : 'Filtered Results'}
              </h2>
              <p className="text-sm text-muted-foreground">
                Found {filteredSamples.length} sample{filteredSamples.length !== 1 ? 's' : ''}
                {searchQuery && ` matching "${searchQuery}"`}
                {selectedTags.size > 0 && ` with tags: ${Array.from(selectedTags).join(', ')}`}
              </p>
            </div>
            {filteredSamples.length > 0 ? (
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
                {filteredSamples.map((sample: Sample) => (
                  <SampleCard
                    key={sample.id}
                    sample={sample}
                    onViewSource={handleViewSource}
                    onRun={handleRun}
                  />
                ))}
              </div>
            ) : (
              <div className="text-center py-12 text-muted-foreground">
                No samples found matching your search.
              </div>
            )}
          </section>
        ) : (
          // Show normal category view
          <>
            {Object.entries(categories).map(([catId, catInfo]) => {
              const catSamples = samplesByCategory[catId];
              if (!catSamples || catSamples.length === 0) return null;
              return (
                <CategorySection
                  key={catId}
                  categoryId={catId}
                  category={catInfo}
                  samples={catSamples}
                  onViewSource={handleViewSource}
                  onRun={handleRun}
                />
              );
            })}
          </>
        )}

        <Footer />
      </main>

      <SourceModal
        isOpen={modalOpen}
        title={modalTitle}
        source={modalSource}
        onClose={() => setModalOpen(false)}
        onCopy={handleCopy}
        onRun={() => currentSampleId && handleRun(currentSampleId)}
      />

      <SettingsModal
        isOpen={settingsOpen}
        settings={settings}
        onClose={() => setSettingsOpen(false)}
        onSave={handleSettingsSave}
      />

      <Toast
        message={toast.message}
        isVisible={toast.visible}
        onHide={hideToast}
        type={toast.type}
      />

      <ProcessConsole
        isOpen={consoleOpen}
        onClose={() => setConsoleOpen(false)}
        onKillProcess={handleKillProcess}
      />
    </div>
  );
}

export default App;
