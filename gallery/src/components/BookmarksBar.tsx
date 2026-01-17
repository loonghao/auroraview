/**
 * BookmarksBar - Browser-style bookmarks bar component
 * 
 * Features:
 * - Display bookmarks horizontally
 * - Drag and drop reordering
 * - Add/remove bookmarks
 * - Folder support
 */

import { useState, useCallback } from 'react';
import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

export interface Bookmark {
  id: string;
  title: string;
  url: string;
  favicon?: string;
  folderId?: string;
}

export interface BookmarkFolder {
  id: string;
  name: string;
  bookmarks: Bookmark[];
}

interface BookmarksBarProps {
  bookmarks: Bookmark[];
  folders: BookmarkFolder[];
  onBookmarkClick: (bookmark: Bookmark) => void;
  onAddBookmark: (url: string, title: string) => void;
  onRemoveBookmark: (id: string) => void;
  onFolderClick?: (folder: BookmarkFolder) => void;
  className?: string;
  visible?: boolean;
}

export function BookmarksBar({
  bookmarks,
  folders,
  onBookmarkClick,
  onAddBookmark,
  onRemoveBookmark,
  onFolderClick: _onFolderClick,
  className,
  visible = true,
}: BookmarksBarProps) {
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [newUrl, setNewUrl] = useState('');
  const [newTitle, setNewTitle] = useState('');
  const [openFolderId, setOpenFolderId] = useState<string | null>(null);

  const handleAddBookmark = useCallback(() => {
    if (newUrl && newTitle) {
      onAddBookmark(newUrl, newTitle);
      setNewUrl('');
      setNewTitle('');
      setShowAddDialog(false);
    }
  }, [newUrl, newTitle, onAddBookmark]);

  if (!visible) return null;

  return (
    <div
      className={cn(
        "flex items-center gap-1 px-2 py-1 bg-muted/30 border-b border-border overflow-x-auto",
        className
      )}
    >
      {/* Bookmarks */}
      {bookmarks.filter(b => !b.folderId).map((bookmark) => (
        <button
          key={bookmark.id}
          onClick={() => onBookmarkClick(bookmark)}
          className={cn(
            "flex items-center gap-1.5 px-2 py-1 rounded text-sm",
            "hover:bg-muted transition-colors group max-w-[150px]"
          )}
          title={bookmark.url}
        >
          {bookmark.favicon ? (
            <img src={bookmark.favicon} alt="" className="w-4 h-4 rounded" />
          ) : (
            <Icons.Globe className="w-4 h-4 text-muted-foreground" />
          )}
          <span className="truncate">{bookmark.title}</span>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onRemoveBookmark(bookmark.id);
            }}
            className="hidden group-hover:block p-0.5 hover:bg-destructive/20 rounded"
          >
            <Icons.X className="w-3 h-3" />
          </button>
        </button>
      ))}

      {/* Folders */}
      {folders.map((folder) => (
        <div key={folder.id} className="relative">
          <button
            onClick={() => setOpenFolderId(openFolderId === folder.id ? null : folder.id)}
            className={cn(
              "flex items-center gap-1.5 px-2 py-1 rounded text-sm",
              "hover:bg-muted transition-colors",
              openFolderId === folder.id && "bg-muted"
            )}
          >
            <Icons.Folder className="w-4 h-4 text-muted-foreground" />
            <span className="truncate max-w-[100px]">{folder.name}</span>
            <Icons.ChevronDown className="w-3 h-3 text-muted-foreground" />
          </button>
          
          {/* Folder dropdown */}
          {openFolderId === folder.id && (
            <div className="absolute top-full left-0 mt-1 w-48 bg-popover border border-border rounded-lg shadow-lg z-50">
              {folder.bookmarks.length === 0 ? (
                <div className="px-3 py-2 text-sm text-muted-foreground">
                  No bookmarks in folder
                </div>
              ) : (
                folder.bookmarks.map((bookmark) => (
                  <button
                    key={bookmark.id}
                    onClick={() => {
                      onBookmarkClick(bookmark);
                      setOpenFolderId(null);
                    }}
                    className="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-muted text-left"
                  >
                    {bookmark.favicon ? (
                      <img src={bookmark.favicon} alt="" className="w-4 h-4 rounded" />
                    ) : (
                      <Icons.Globe className="w-4 h-4 text-muted-foreground" />
                    )}
                    <span className="truncate">{bookmark.title}</span>
                  </button>
                ))
              )}
            </div>
          )}
        </div>
      ))}

      {/* Add bookmark button */}
      <button
        onClick={() => setShowAddDialog(true)}
        className="flex items-center gap-1 px-2 py-1 rounded text-sm text-muted-foreground hover:bg-muted transition-colors"
      >
        <Icons.Plus className="w-4 h-4" />
      </button>

      {/* Add bookmark dialog */}
      {showAddDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-card border border-border rounded-lg p-4 w-80 shadow-xl">
            <h3 className="font-semibold mb-3">Add Bookmark</h3>
            <div className="space-y-3">
              <div>
                <label className="text-sm text-muted-foreground">Title</label>
                <input
                  type="text"
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  className="w-full px-3 py-2 border border-border rounded-lg bg-background text-sm"
                  placeholder="Bookmark title"
                />
              </div>
              <div>
                <label className="text-sm text-muted-foreground">URL</label>
                <input
                  type="url"
                  value={newUrl}
                  onChange={(e) => setNewUrl(e.target.value)}
                  className="w-full px-3 py-2 border border-border rounded-lg bg-background text-sm"
                  placeholder="https://example.com"
                />
              </div>
              <div className="flex justify-end gap-2">
                <button
                  onClick={() => setShowAddDialog(false)}
                  className="px-3 py-1.5 text-sm rounded-lg hover:bg-muted"
                >
                  Cancel
                </button>
                <button
                  onClick={handleAddBookmark}
                  disabled={!newUrl || !newTitle}
                  className={cn(
                    "px-3 py-1.5 text-sm rounded-lg bg-primary text-primary-foreground",
                    (!newUrl || !newTitle) && "opacity-50 cursor-not-allowed"
                  )}
                >
                  Add
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
