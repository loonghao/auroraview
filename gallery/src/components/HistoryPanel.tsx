/**
 * HistoryPanel - Browsing history display with search and filtering
 */

import { useState, useMemo, useCallback } from 'react';
import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

export interface HistoryEntry {
  id: string;
  url: string;
  title: string;
  visitCount: number;
  lastVisit: string; // ISO date string
  favicon?: string;
}

interface HistoryPanelProps {
  entries: HistoryEntry[];
  onEntryClick: (entry: HistoryEntry) => void;
  onDeleteEntry: (id: string) => void;
  onClearAll: () => void;
  onClose: () => void;
  className?: string;
}

type TimeFilter = 'today' | 'yesterday' | 'week' | 'month' | 'all';

export function HistoryPanel({
  entries,
  onEntryClick,
  onDeleteEntry,
  onClearAll,
  onClose,
  className,
}: HistoryPanelProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [timeFilter, setTimeFilter] = useState<TimeFilter>('all');
  const [confirmClear, setConfirmClear] = useState(false);

  // Filter entries by time
  const filterByTime = useCallback((entry: HistoryEntry, filter: TimeFilter): boolean => {
    const entryDate = new Date(entry.lastVisit);
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterday = new Date(today.getTime() - 24 * 60 * 60 * 1000);
    const weekAgo = new Date(today.getTime() - 7 * 24 * 60 * 60 * 1000);
    const monthAgo = new Date(today.getTime() - 30 * 24 * 60 * 60 * 1000);

    switch (filter) {
      case 'today':
        return entryDate >= today;
      case 'yesterday':
        return entryDate >= yesterday && entryDate < today;
      case 'week':
        return entryDate >= weekAgo;
      case 'month':
        return entryDate >= monthAgo;
      case 'all':
      default:
        return true;
    }
  }, []);

  // Filter and group entries
  const filteredEntries = useMemo(() => {
    let result = entries;

    // Apply time filter
    result = result.filter(entry => filterByTime(entry, timeFilter));

    // Apply search filter
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      result = result.filter(
        entry =>
          entry.title.toLowerCase().includes(query) ||
          entry.url.toLowerCase().includes(query)
      );
    }

    // Sort by last visit (most recent first)
    return [...result].sort(
      (a, b) => new Date(b.lastVisit).getTime() - new Date(a.lastVisit).getTime()
    );
  }, [entries, searchQuery, timeFilter, filterByTime]);

  // Group entries by date
  const groupedEntries = useMemo(() => {
    const groups: { [key: string]: HistoryEntry[] } = {};
    
    for (const entry of filteredEntries) {
      const date = new Date(entry.lastVisit);
      const today = new Date();
      const yesterday = new Date(today.getTime() - 24 * 60 * 60 * 1000);
      
      let groupKey: string;
      if (date.toDateString() === today.toDateString()) {
        groupKey = 'Today';
      } else if (date.toDateString() === yesterday.toDateString()) {
        groupKey = 'Yesterday';
      } else {
        groupKey = date.toLocaleDateString('en-US', {
          weekday: 'long',
          month: 'short',
          day: 'numeric',
        });
      }

      if (!groups[groupKey]) {
        groups[groupKey] = [];
      }
      groups[groupKey].push(entry);
    }

    return groups;
  }, [filteredEntries]);

  const formatTime = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    });
  };

  const handleClearAll = () => {
    if (confirmClear) {
      onClearAll();
      setConfirmClear(false);
    } else {
      setConfirmClear(true);
      // Reset confirm state after 3 seconds
      setTimeout(() => setConfirmClear(false), 3000);
    }
  };

  return (
    <div className={cn("flex flex-col h-full bg-card", className)}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-border">
        <div className="flex items-center gap-2">
          <Icons.History className="w-5 h-5 text-primary" />
          <h2 className="text-lg font-semibold">History</h2>
        </div>
        <button
          onClick={onClose}
          className="p-2 hover:bg-muted rounded-lg transition-colors"
        >
          <Icons.X className="w-5 h-5" />
        </button>
      </div>

      {/* Search and filters */}
      <div className="p-3 space-y-3 border-b border-border">
        {/* Search */}
        <div className="relative">
          <Icons.Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search history..."
            className="w-full pl-9 pr-3 py-2 text-sm bg-muted/50 border border-border rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50"
          />
        </div>

        {/* Time filters */}
        <div className="flex items-center gap-1 overflow-x-auto">
          {(['today', 'yesterday', 'week', 'month', 'all'] as TimeFilter[]).map((filter) => (
            <button
              key={filter}
              onClick={() => setTimeFilter(filter)}
              className={cn(
                "px-3 py-1 text-xs rounded-full whitespace-nowrap transition-colors",
                timeFilter === filter
                  ? "bg-primary text-primary-foreground"
                  : "bg-muted hover:bg-muted/80"
              )}
            >
              {filter.charAt(0).toUpperCase() + filter.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* History list */}
      <div className="flex-1 overflow-y-auto">
        {Object.keys(groupedEntries).length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <Icons.History className="w-12 h-12 mb-3 opacity-50" />
            <p>No history found</p>
            {searchQuery && (
              <p className="text-sm mt-1">Try a different search term</p>
            )}
          </div>
        ) : (
          Object.entries(groupedEntries).map(([dateGroup, groupEntries]) => (
            <div key={dateGroup}>
              {/* Date header */}
              <div className="sticky top-0 px-4 py-2 text-xs font-medium text-muted-foreground bg-muted/50 backdrop-blur-sm">
                {dateGroup}
              </div>
              
              {/* Entries */}
              {groupEntries.map((entry) => (
                <div
                  key={entry.id}
                  className="group flex items-center gap-3 px-4 py-2 hover:bg-muted/50 cursor-pointer"
                  onClick={() => onEntryClick(entry)}
                >
                  {/* Favicon */}
                  <div className="w-8 h-8 flex items-center justify-center flex-shrink-0">
                    {entry.favicon ? (
                      <img
                        src={entry.favicon}
                        alt=""
                        className="w-5 h-5 rounded"
                      />
                    ) : (
                      <Icons.Globe className="w-5 h-5 text-muted-foreground" />
                    )}
                  </div>

                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium truncate">
                      {entry.title || entry.url}
                    </div>
                    <div className="text-xs text-muted-foreground truncate">
                      {entry.url}
                    </div>
                  </div>

                  {/* Time and actions */}
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-muted-foreground">
                      {formatTime(entry.lastVisit)}
                    </span>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onDeleteEntry(entry.id);
                      }}
                      className="p-1 rounded hover:bg-destructive/20 opacity-0 group-hover:opacity-100 transition-opacity"
                      title="Remove from history"
                    >
                      <Icons.X className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          ))
        )}
      </div>

      {/* Footer with clear all */}
      {entries.length > 0 && (
        <div className="p-3 border-t border-border">
          <button
            onClick={handleClearAll}
            className={cn(
              "w-full py-2 text-sm rounded-lg transition-colors",
              confirmClear
                ? "bg-destructive text-destructive-foreground"
                : "bg-muted hover:bg-muted/80"
            )}
          >
            {confirmClear ? "Click again to confirm" : "Clear all history"}
          </button>
        </div>
      )}
    </div>
  );
}
