/**
 * React hooks for AuroraView features (bookmarks, history, downloads, etc.)
 */

import { useState, useEffect, useCallback } from 'react';
import { useAuroraView, type Bookmark, type BookmarkFolder, type HistoryEntry, type DownloadItem } from './useAuroraView';

/**
 * Hook for managing bookmarks
 */
export function useBookmarks() {
  const {
    isReady,
    getBookmarks,
    addBookmark,
    removeBookmark,
    isBookmarked,
  } = useAuroraView();

  const [bookmarks, setBookmarks] = useState<Bookmark[]>([]);
  const [folders, setFolders] = useState<BookmarkFolder[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!isReady) return;
    setLoading(true);
    try {
      const result = await getBookmarks();
      if (result.ok) {
        setBookmarks(result.bookmarks);
        setFolders(result.folders);
      }
    } finally {
      setLoading(false);
    }
  }, [isReady, getBookmarks]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const add = useCallback(async (url: string, title: string, folderId?: string) => {
    const result = await addBookmark(url, title, folderId);
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [addBookmark, refresh]);

  const remove = useCallback(async (id: string) => {
    const result = await removeBookmark(id);
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [removeBookmark, refresh]);

  const checkBookmarked = useCallback(async (url: string) => {
    return isBookmarked(url);
  }, [isBookmarked]);

  return {
    bookmarks,
    folders,
    loading,
    refresh,
    add,
    remove,
    checkBookmarked,
  };
}

/**
 * Hook for managing browsing history
 */
export function useHistory() {
  const {
    isReady,
    getHistory,
    addHistory,
    removeHistory,
    clearHistory,
    searchHistory,
  } = useAuroraView();

  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async (limit = 100) => {
    if (!isReady) return;
    setLoading(true);
    try {
      const result = await getHistory(limit);
      if (result.ok) {
        setEntries(result.entries);
        setTotal(result.total);
      }
    } finally {
      setLoading(false);
    }
  }, [isReady, getHistory]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const add = useCallback(async (url: string, title: string) => {
    const result = await addHistory(url, title);
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [addHistory, refresh]);

  const remove = useCallback(async (id: string) => {
    const result = await removeHistory(id);
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [removeHistory, refresh]);

  const clear = useCallback(async () => {
    const result = await clearHistory();
    if (result.ok) {
      setEntries([]);
      setTotal(0);
    }
    return result;
  }, [clearHistory]);

  const search = useCallback(async (query: string, limit = 50) => {
    if (!query.trim()) {
      return { ok: true, entries: [] };
    }
    return searchHistory(query, limit);
  }, [searchHistory]);

  return {
    entries,
    total,
    loading,
    refresh,
    add,
    remove,
    clear,
    search,
  };
}

/**
 * Hook for managing downloads
 */
export function useDownloads() {
  const {
    isReady,
    getDownloads,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    retryDownload,
    removeDownload,
    clearCompletedDownloads,
  } = useAuroraView();

  const [downloads, setDownloads] = useState<DownloadItem[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!isReady) return;
    setLoading(true);
    try {
      const result = await getDownloads();
      if (result.ok) {
        setDownloads(result.downloads);
      }
    } finally {
      setLoading(false);
    }
  }, [isReady, getDownloads]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const pause = useCallback(async (id: string) => {
    const result = await pauseDownload(id);
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [pauseDownload, refresh]);

  const resume = useCallback(async (id: string) => {
    const result = await resumeDownload(id);
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [resumeDownload, refresh]);

  const cancel = useCallback(async (id: string) => {
    const result = await cancelDownload(id);
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [cancelDownload, refresh]);

  const retry = useCallback(async (id: string) => {
    const result = await retryDownload(id);
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [retryDownload, refresh]);

  const remove = useCallback(async (id: string) => {
    const result = await removeDownload(id);
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [removeDownload, refresh]);

  const clearCompleted = useCallback(async () => {
    const result = await clearCompletedDownloads();
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [clearCompletedDownloads, refresh]);

  // Stats
  const activeCount = downloads.filter(d => 
    ['pending', 'downloading', 'paused'].includes(d.state)
  ).length;
  const completedCount = downloads.filter(d => d.state === 'completed').length;

  return {
    downloads,
    loading,
    refresh,
    pause,
    resume,
    cancel,
    retry,
    remove,
    clearCompleted,
    activeCount,
    completedCount,
  };
}

/**
 * Hook for managing user settings
 */
export function useSettings() {
  const {
    isReady,
    getSettings,
    getSetting,
    setSetting,
    resetSettings,
  } = useAuroraView();

  const [settings, setSettings] = useState<Record<string, unknown>>({});
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!isReady) return;
    setLoading(true);
    try {
      const result = await getSettings();
      if (result.ok) {
        setSettings(result.settings);
      }
    } finally {
      setLoading(false);
    }
  }, [isReady, getSettings]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const get = useCallback(async (key: string) => {
    return getSetting(key);
  }, [getSetting]);

  const set = useCallback(async (key: string, value: unknown) => {
    const result = await setSetting(key, value);
    if (result.ok) {
      setSettings(prev => ({ ...prev, [key]: value }));
    }
    return result;
  }, [setSetting]);

  const reset = useCallback(async () => {
    const result = await resetSettings();
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [resetSettings, refresh]);

  return {
    settings,
    loading,
    refresh,
    get,
    set,
    reset,
  };
}

/**
 * Hook for notifications
 */
export function useNotifications() {
  const {
    isReady,
    showNotification,
    dismissNotification,
    getNotifications,
  } = useAuroraView();

  const [notifications, setNotifications] = useState<Array<{
    id: string;
    title: string;
    message: string;
    type: string;
  }>>([]);

  const refresh = useCallback(async () => {
    if (!isReady) return;
    const result = await getNotifications();
    if (result.ok) {
      setNotifications(result.notifications);
    }
  }, [isReady, getNotifications]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const show = useCallback(async (
    title: string,
    message: string,
    type: 'info' | 'success' | 'warning' | 'error' = 'info',
    timeout?: number
  ) => {
    const result = await showNotification(title, message, type, timeout);
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [showNotification, refresh]);

  const dismiss = useCallback(async (id: string) => {
    const result = await dismissNotification(id);
    if (result.ok) {
      await refresh();
    }
    return result;
  }, [dismissNotification, refresh]);

  // Convenience methods
  const info = useCallback((title: string, message: string, timeout?: number) => 
    show(title, message, 'info', timeout), [show]);
  const success = useCallback((title: string, message: string, timeout?: number) => 
    show(title, message, 'success', timeout), [show]);
  const warning = useCallback((title: string, message: string, timeout?: number) => 
    show(title, message, 'warning', timeout), [show]);
  const error = useCallback((title: string, message: string, timeout?: number) => 
    show(title, message, 'error', timeout), [show]);

  return {
    notifications,
    refresh,
    show,
    dismiss,
    info,
    success,
    warning,
    error,
  };
}
