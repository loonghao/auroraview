import { useState, useCallback, useMemo, useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { CategorySection } from './components/CategorySection';
import { QuickLinks } from './components/QuickLinks';
import { SourceModal } from './components/SourceModal';
import { SettingsModal, type Settings, type BrowserExtensionStatus } from './components/SettingsModal';
import { Toast } from './components/Toast';
import { SearchBar } from './components/SearchBar';
import { Footer } from './components/Footer';
import { SampleCard } from './components/SampleCard';
import { TagFilter } from './components/TagFilter';
import { ProcessConsole } from './components/ProcessConsole';
import { ExtensionPanel, type InstalledExtension } from './components/ExtensionPanel';
import { ExtensionToolbar } from './components/ExtensionToolbar';
import { SplitLayout } from './components/SplitLayout';
import { ExtensionContent } from './components/ExtensionContent';
import { type Tag } from './data/samples';
import { useAuroraView, type Sample, type Category, type ExtensionInfo } from './hooks/useAuroraView';
import * as Icons from 'lucide-react';

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
  return { 
    runMode: 'external', 
    linkMode: 'browser',
    browserExtension: {
      enabled: false,
      wsPort: 49152,
      httpPort: 49153,
    }
  };
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
  const [viewMode, setViewMode] = useState<'gallery' | 'extensions' | 'extension-content'>('gallery');

  // Browser extension status (legacy)
  const [extensionStatus, setExtensionStatus] = useState<BrowserExtensionStatus>({
    enabled: false,
    wsPort: 49152,
    httpPort: 49153,
    connectedClients: 0,
    isRunning: false,
  });

  // Dynamic samples and categories from Python backend
  const [samples, setSamples] = useState<Sample[]>([]);
  const [categories, setCategories] = useState<Record<string, Category>>({});
  const [dataLoaded, setDataLoaded] = useState(false);

  const { 
    isReady, 
    getSource, 
    runSample, 
    getSamples, 
    getCategories, 
    openUrl, 
    killProcess,
    // Legacy browser extension bridge APIs
    startExtensionBridge,
    stopExtensionBridge,
    getExtensionStatus,
    installExtension,
    // Extension management APIs
    installToWebView,
    listWebViewExtensions,
    removeWebViewExtension,
    openExtensionsDir,
    restartApp,
    // Rust native extension APIs
    listExtensions,
    openSidePanel,
    closeSidePanel,
    // Chrome management API
    managementSetEnabled,
    managementGetPermissionWarnings,
  } = useAuroraView();

  // Installed WebView extensions
  const [installedExtensions, setInstalledExtensions] = useState<InstalledExtension[]>([]);
  // Rust native extensions (reserved for future use)
  const [_activeExtensions, setActiveExtensions] = useState<ExtensionInfo[]>([]);
  const [extensionPendingRestart, setExtensionPendingRestart] = useState(false);
  // Developer mode for extensions panel
  const [developerMode, setDeveloperMode] = useState(() => {
    try {
      const saved = localStorage.getItem('auroraview-developer-mode');
      return saved ? JSON.parse(saved) : true;
    } catch {
      return true;
    }
  });
  // Local enabled state for extensions (persisted in localStorage)
  const [enabledExtensions, setEnabledExtensions] = useState<Set<string>>(() => {
    try {
      const saved = localStorage.getItem('auroraview-enabled-extensions');
      return saved ? new Set(JSON.parse(saved)) : new Set();
    } catch {
      return new Set();
    }
  });

  // Save developer mode to localStorage
  useEffect(() => {
    try {
      localStorage.setItem('auroraview-developer-mode', JSON.stringify(developerMode));
    } catch {
      // Ignore storage errors
    }
  }, [developerMode]);

  // Save enabled extensions to localStorage
  useEffect(() => {
    try {
      localStorage.setItem('auroraview-enabled-extensions', JSON.stringify([...enabledExtensions]));
    } catch {
      // Ignore storage errors
    }
  }, [enabledExtensions]);

  // Merge installed extensions with enabled state for display
  const displayExtensions = useMemo((): InstalledExtension[] => {
    return installedExtensions.map(ext => ({
      ...ext,
      // Check local enabled state, default to true for newly installed extensions
      enabled: enabledExtensions.size === 0 ? true : enabledExtensions.has(ext.id),
    }));
  }, [installedExtensions, enabledExtensions]);

  // Side Panel state
  const [sidePanelOpen, setSidePanelOpen] = useState(false);
  const [activeSidePanelExtension, setActiveSidePanelExtension] = useState<InstalledExtension | null>(null);

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

  // Load installed WebView extensions
  const refreshExtensions = useCallback(async () => {
    if (!isReady) return;
    try {
      const result = await listWebViewExtensions();
      if (result.ok) {
        setInstalledExtensions(result.extensions);
        // Auto-enable newly installed extensions
        setEnabledExtensions(prev => {
          // If this is the first load and no extensions are enabled, enable all
          if (prev.size === 0 && result.extensions.length > 0) {
            return new Set(result.extensions.map(e => e.id));
          }
          return prev;
        });
      }
      // Also try to get Rust native extensions (for toolbar display)
      try {
        console.log('[App] Calling listExtensions()...');
        const rustExtensions = await listExtensions();
        console.log('[App] listExtensions() returned:', rustExtensions);
        setActiveExtensions(rustExtensions);
      } catch (e) {
        // Rust plugin may not be available
        console.error('[App] listExtensions() error:', e);
      }
    } catch (err) {
      console.error('Failed to load extensions:', err);
    }
  }, [isReady, listWebViewExtensions, listExtensions]);

  useEffect(() => {
    refreshExtensions();
  }, [refreshExtensions]);

  // Fetch extension status - only poll when settings modal is open
  useEffect(() => {
    if (!isReady) return;
    
    const fetchStatus = async () => {
      try {
        const status = await getExtensionStatus();
        setExtensionStatus(status);
      } catch {
        // Silently ignore errors to avoid console spam
      }
    };
    
    // Initial fetch
    fetchStatus();
    
    // Only poll when settings modal is open
    if (settingsOpen) {
      const interval = setInterval(fetchStatus, 5000); // Poll every 5 seconds
      return () => clearInterval(interval);
    }
  }, [isReady, settingsOpen, getExtensionStatus]);

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
    setViewMode('gallery');
    setActiveCategory(categoryId);
    // Wait for render switch then scroll
    setTimeout(() => {
        document.getElementById(`category-${categoryId}`)?.scrollIntoView({ behavior: 'smooth' });
    }, 100);
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
        const result = await openUrl(url);
        if (!result.ok) {
          console.error('[handleOpenUrl] Failed:', result.error);
          showToast(result.error || 'Failed to open URL', 'error');
          // Fallback to window.open
          window.open(url, '_blank');
        }
      } catch (err) {
        console.error('[handleOpenUrl] Exception:', err);
        // Fallback to window.open
        window.open(url, '_blank');
      }
    } else {
      // Fallback for development
      window.open(url, '_blank');
    }
  }, [isReady, openUrl, showToast]);

  const handleOpenInWebView = useCallback((url: string, title?: string) => {
    // Use window.open() to create a new child WebView window
    // With new_window_mode="child_webview", this creates a proper WebView2 window
    // that can load any URL without X-Frame-Options restrictions
    const windowName = title || 'AuroraView';
    const features = 'width=1024,height=768,menubar=no,toolbar=no,location=yes,status=no,resizable=yes';
    console.log(`[handleOpenInWebView] Opening: ${url} with name: ${windowName}`);
    const newWindow = window.open(url, windowName, features);
    console.log(`[handleOpenInWebView] window.open() returned:`, newWindow);
  }, []);

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

  // Browser extension toggle handler (legacy)
  const handleToggleExtension = useCallback(async (enabled: boolean) => {
    if (!isReady) {
      showToast('AuroraView not ready', 'error');
      return;
    }
    
    try {
      if (enabled) {
        const result = await startExtensionBridge(
          settings.browserExtension.wsPort,
          settings.browserExtension.httpPort
        );
        if (result.ok) {
          showToast('Extension bridge started');
          // Refresh status
          const status = await getExtensionStatus();
          setExtensionStatus(status);
        } else {
          showToast(result.error || 'Failed to start extension bridge', 'error');
        }
      } else {
        const result = await stopExtensionBridge();
        if (result.ok) {
          showToast('Extension bridge stopped');
          setExtensionStatus(prev => ({ ...prev, isRunning: false, connectedClients: 0 }));
        } else {
          showToast(result.error || 'Failed to stop extension bridge', 'error');
        }
      }
    } catch (err) {
      console.error('Extension toggle error:', err);
      showToast('Failed to toggle extension bridge', 'error');
    }
  }, [isReady, settings.browserExtension, startExtensionBridge, stopExtensionBridge, getExtensionStatus, showToast]);

  // Open Chrome extension store
  const handleOpenExtensionStore = useCallback(() => {
    console.log('[handleOpenExtensionStore] Called');
    // Open new Chrome Web Store in WebView
    const extensionStoreUrl = 'https://chromewebstore.google.com/category/extensions';
    handleOpenInWebView(extensionStoreUrl, 'Chrome Web Store');
  }, [handleOpenInWebView]);

  // Install extension from URL (Chrome/Edge store)
  const handleInstallFromUrl = useCallback(async (url: string): Promise<{ ok: boolean; message?: string; error?: string; requiresRestart?: boolean }> => {
    if (!isReady) {
      return { ok: false, error: 'AuroraView not ready' };
    }
    
    try {
      // Call Python backend to download and install extension from URL
      const result = await window.auroraview?.call?.('api.install_extension_from_url', { url }) as {
        ok?: boolean;
        success?: boolean;
        message?: string;
        error?: string;
        requiresRestart?: boolean;
      } | undefined;
      
      if (result) {
        const success = result.ok || result.success || false;
        if (success) {
          showToast(result.message || 'Extension installed!');
          setExtensionPendingRestart(true);
        }
        return {
          ok: success,
          message: result.message,
          error: result.error,
          requiresRestart: result.requiresRestart,
        };
      }
      return { ok: false, error: 'No response from backend' };
    } catch (err) {
      console.error('[handleInstallFromUrl] Error:', err);
      return { ok: false, error: String(err) };
    }
  }, [isReady, showToast]);

  // Install extension from dropped file (legacy)
  const handleInstallExtension = useCallback(async (path: string, browser: 'chrome' | 'firefox') => {
    if (!isReady) {
      showToast('AuroraView not ready', 'error');
      return;
    }
    
    try {
      const result = await installExtension(path, browser);
      
      if (result.ok) {
        showToast(result.message || `Opening ${browser} extension installer...`);
      } else {
        showToast(result.error || 'Failed to install extension', 'error');
      }
    } catch (err) {
      console.error('[handleInstallExtension] Error:', err);
      showToast('Failed to install extension', 'error');
    }
  }, [isReady, installExtension, showToast]);

  // Install extension to WebView2's extensions directory
  const handleInstallToWebView = useCallback(async (path: string) => {
    if (!isReady) {
      return { ok: false, error: 'AuroraView not ready' };
    }
    
    try {
      const result = await installToWebView(path);
      
      if (result.ok) {
        showToast(result.message || 'Extension installed! Restart required.');
      } else {
        showToast(result.error || 'Failed to install extension', 'error');
      }
      return result;
    } catch (err) {
      console.error('[handleInstallToWebView] Error:', err);
      return { ok: false, error: String(err) };
    }
  }, [isReady, installToWebView, showToast]);

  // Open extensions directory
  const handleOpenExtensionsDir = useCallback(async () => {
    if (!isReady) {
      showToast('AuroraView not ready', 'error');
      return;
    }
    
    try {
      await openExtensionsDir();
    } catch (err) {
      console.error('[handleOpenExtensionsDir] Error:', err);
      showToast('Failed to open extensions folder', 'error');
    }
  }, [isReady, openExtensionsDir, showToast]);

  // Restart the application (for applying extension changes)
  const handleRestartApp = useCallback(async () => {
    if (!isReady) {
      showToast('AuroraView not ready', 'error');
      return;
    }
    
    try {
      showToast('Restarting application...');
      await restartApp();
    } catch (err) {
      console.error('[handleRestartApp] Error:', err);
      showToast('Failed to restart application', 'error');
    }
  }, [isReady, restartApp, showToast]);

  // Handle extension panel install
  const handlePanelInstallExtension = useCallback(async (path: string) => {
    if (!isReady) {
      return { ok: false, error: 'AuroraView not ready' };
    }
    try {
      const result = await installToWebView(path);
      if (result.ok && result.requiresRestart) {
        setExtensionPendingRestart(true);
      }
      return result;
    } catch (err) {
      return { ok: false, error: String(err) };
    }
  }, [isReady, installToWebView]);

  // Handle extension panel remove
  const handlePanelRemoveExtension = useCallback(async (id: string) => {
    if (!isReady) {
      return { ok: false, error: 'AuroraView not ready' };
    }
    try {
      const result = await removeWebViewExtension(id);
      if (result.ok) {
        setExtensionPendingRestart(true);
        // Close side panel if this extension was open
        if (activeSidePanelExtension?.id === id) {
          handleCloseSidePanel();
        }
      }
      return result;
    } catch (err) {
      return { ok: false, error: String(err) };
    }
  }, [isReady, removeWebViewExtension, activeSidePanelExtension]);

  // Handle opening extension side panel (now opens in main area instead of side panel)
  const handleOpenSidePanel = useCallback(async (extension: InstalledExtension) => {
    // Open extension content in main area instead of side panel
    setActiveSidePanelExtension(extension);
    setViewMode('extension-content');
    
    // Notify Rust plugin about side panel state
    if (isReady) {
      try {
        await openSidePanel(extension.id);
      } catch (err) {
        console.warn('[handleOpenSidePanel] Failed to notify Rust:', err);
      }
    }
  }, [isReady, openSidePanel]);

  // Handle opening extension popup (opens in main area, similar to side panel)
  const handleOpenPopup = useCallback(async (extension: InstalledExtension) => {
    // Open extension popup content in main area
    setActiveSidePanelExtension(extension);
    setViewMode('extension-content');
    
    // Notify Rust plugin about popup state (reuse side panel mechanism for now)
    if (isReady) {
      try {
        await openSidePanel(extension.id);
      } catch (err) {
        console.warn('[handleOpenPopup] Failed to notify Rust:', err);
      }
    }
  }, [isReady, openSidePanel]);

  // Handle closing extension side panel
  const handleCloseSidePanel = useCallback(async () => {
    const extensionId = activeSidePanelExtension?.id;
    setSidePanelOpen(false);
    setActiveSidePanelExtension(null);
    setViewMode('gallery');
    
    // Notify Rust plugin about side panel state
    if (isReady && extensionId) {
      try {
        await closeSidePanel(extensionId);
      } catch (err) {
        console.warn('[handleCloseSidePanel] Failed to notify Rust:', err);
      }
    }
  }, [isReady, closeSidePanel, activeSidePanelExtension]);

  // Handle toolbar extension click (show extension content in main area)
  const handleToolbarExtensionClick = useCallback((id: string) => {
    const ext = installedExtensions.find(e => e.id === id);
    if (ext) {
      // If clicking the same extension, toggle back to gallery
      if (activeSidePanelExtension?.id === id && viewMode === 'extension-content') {
        setViewMode('gallery');
        setActiveSidePanelExtension(null);
      } else {
        setActiveSidePanelExtension(ext);
        setViewMode('extension-content');
      }
    }
  }, [installedExtensions, activeSidePanelExtension, viewMode]);

  // Handle closing extension content view
  const handleCloseExtensionContent = useCallback(() => {
    setViewMode('gallery');
    setActiveSidePanelExtension(null);
  }, []);

  // Handle extension toggle (enable/disable)
  // Note: WebView2 extensions are managed locally - the Rust management API is for
  // Chrome extension compatibility, not for WebView2 native extensions
  const handleExtensionToggle = useCallback(async (extension: InstalledExtension, enabled: boolean) => {
    console.log(`[handleExtensionToggle] ${extension.id} -> ${enabled}`);
    
    // Update local state - this is the primary state management for WebView2 extensions
    setEnabledExtensions(prev => {
      const next = new Set(prev);
      if (enabled) {
        next.add(extension.id);
      } else {
        next.delete(extension.id);
      }
      return next;
    });
    
    // Optionally notify Rust layer (fire-and-forget, don't block on errors)
    managementSetEnabled(extension.id, enabled).catch(() => {
      // Silently ignore - Rust layer may not have this extension registered
    });
  }, [managementSetEnabled]);

  // Handle opening extension options
  const handleOpenOptions = useCallback((extension: InstalledExtension) => {
    if (extension.optionsUrl) {
      // Open options page in a new window
      window.open(extension.optionsUrl, `${extension.name} Options`, 'width=800,height=600');
    }
  }, []);

  // Handle viewing extension permissions
  const handleViewPermissions = useCallback(async (extension: InstalledExtension) => {
    try {
      const warnings = await managementGetPermissionWarnings(extension.id);
      if (warnings.length > 0) {
        alert(`Permission warnings for ${extension.name}:\n\n${warnings.join('\n')}`);
      } else {
        alert(`${extension.name} has no special permission warnings.`);
      }
    } catch (e) {
      console.error('[handleViewPermissions] Error:', e);
    }
  }, [managementGetPermissionWarnings]);

  // Handle developer mode toggle
  const handleToggleDeveloperMode = useCallback((enabled: boolean) => {
    setDeveloperMode(enabled);
  }, []);

  // Build side panel config
  const sidePanelConfig = useMemo(() => {
    if (!sidePanelOpen || !activeSidePanelExtension) return null;
    return {
      title: activeSidePanelExtension.name,
      icon: <Icons.Puzzle className="w-4 h-4 text-primary" />,
      content: (
        <ExtensionContent
          extensionId={activeSidePanelExtension.id}
          extensionName={activeSidePanelExtension.name}
          extensionPath={activeSidePanelExtension.path}
          sidePanelPath={activeSidePanelExtension.sidePanelPath || 'sidepanel.html'}
          onOpenInBrowser={handleOpenUrl}
        />
      ),
      onClose: handleCloseSidePanel,
    };
  }, [sidePanelOpen, activeSidePanelExtension, handleCloseSidePanel, handleOpenUrl]);

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
        onExtensionsClick={() => setViewMode('extensions')}
        consoleOpen={consoleOpen}
      />

      <SplitLayout sidePanel={sidePanelConfig}>
        <main className="flex-1 p-8 max-w-5xl ml-14">
        {/* Header */}
        <header className="mb-8 flex items-start justify-between">
          <div>
            <h1 className="text-4xl font-bold mb-2 text-foreground">
                AuroraView Gallery
            </h1>
            <p className="text-muted-foreground">
                Explore all features and components with live demos and source code
            </p>
          </div>
          <ExtensionToolbar 
            extensions={displayExtensions}
            activeExtensionId={viewMode === 'extension-content' ? activeSidePanelExtension?.id ?? null : null}
            onExtensionClick={handleToolbarExtensionClick}
            onManageExtensions={() => setViewMode('extensions')}
            onOpenStore={handleOpenExtensionStore}
          />
        </header>

        {/* Quick Links */}
        {viewMode === 'gallery' && (
             <QuickLinks onCategoryClick={handleCategoryClick} onOpenLink={handleOpenLink} />
        )}

        {/* Extension Panel (Management) */}
        {viewMode === 'extensions' && (
            <ExtensionPanel
            extensions={displayExtensions}
            pendingRestart={extensionPendingRestart}
            onInstallExtension={handlePanelInstallExtension}
            onInstallFromUrl={handleInstallFromUrl}
            onRemoveExtension={handlePanelRemoveExtension}
            onOpenExtensionsDir={handleOpenExtensionsDir}
            onRestartApp={handleRestartApp}
            onRefresh={refreshExtensions}
            onOpenSidePanel={handleOpenSidePanel}
            onOpenPopup={handleOpenPopup}
            onToggleExtension={handleExtensionToggle}
            onOpenStore={handleOpenExtensionStore}
            onOpenOptions={handleOpenOptions}
            onViewPermissions={handleViewPermissions}
            developerMode={developerMode}
            onToggleDeveloperMode={handleToggleDeveloperMode}
            />
        )}

        {/* Extension Content (Full WebView display) */}
        {viewMode === 'extension-content' && activeSidePanelExtension && (
          <div className="flex flex-col h-[calc(100vh-4rem)]">
            {/* Header */}
            <div className="flex items-center justify-between mb-4 flex-shrink-0">
              <div className="flex items-center gap-3">
                <button
                  onClick={handleCloseExtensionContent}
                  className="p-2 rounded-lg hover:bg-muted transition-colors"
                  title="Back to Gallery"
                >
                  <Icons.ArrowLeft className="w-5 h-5" />
                </button>
                <div>
                  <h2 className="text-xl font-bold">{activeSidePanelExtension.name}</h2>
                  <p className="text-sm text-muted-foreground">{activeSidePanelExtension.description}</p>
                </div>
              </div>
            </div>
            {/* Extension Content - fills remaining space */}
            <div className="flex-1 bg-card border border-border rounded-xl overflow-hidden min-h-0">
              <ExtensionContent
                extensionId={activeSidePanelExtension.id}
                extensionName={activeSidePanelExtension.name}
                extensionPath={activeSidePanelExtension.path}
                sidePanelPath={activeSidePanelExtension.sidePanelPath || activeSidePanelExtension.popupPath || 'sidepanel.html'}
                onOpenInBrowser={handleOpenUrl}
              />
            </div>
          </div>
        )}

        {/* Search and Filter */}
        {viewMode === 'gallery' && (
            <>
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
            </>
        )}

        <Footer />
        </main>
      </SplitLayout>

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
        extensionStatus={extensionStatus}
        onClose={() => setSettingsOpen(false)}
        onSave={handleSettingsSave}
        onToggleExtension={handleToggleExtension}
        onOpenExtensionStore={handleOpenExtensionStore}
        onInstallExtension={handleInstallExtension}
        onInstallToWebView={handleInstallToWebView}
        onOpenExtensionsDir={handleOpenExtensionsDir}
        onRestartApp={handleRestartApp}
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
