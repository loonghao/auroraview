import { useState, useEffect, useCallback, useRef } from 'react';
import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

interface ExtensionContentProps {
  extensionId: string;
  extensionName: string;
  extensionPath: string;
  sidePanelPath: string;
  onError?: (error: string) => void;
  onOpenInBrowser?: (url: string) => void;
}

interface ExtensionManifest {
  manifest_version?: number;
  name?: string;
  version?: string;
  description?: string;
  permissions?: string[];
  side_panel?: {
    default_path?: string;
  };
}

/**
 * Extension Content Component
 * 
 * Renders extension side panel content. Supports two modes:
 * 1. Embedded mode: Uses iframe within the side panel (default)
 * 2. Window mode: Opens in a separate browser window with independent DevTools
 * 
 * The window mode is more aligned with Chrome's extension architecture,
 * providing better isolation and debugging capabilities.
 */
export function ExtensionContent({
  extensionId,
  extensionName,
  extensionPath,
  sidePanelPath,
  onError,
  onOpenInBrowser,
}: ExtensionContentProps) {
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [htmlContent, setHtmlContent] = useState<string | null>(null);
  const [polyfill, setPolyfill] = useState<string | null>(null);
  const [wxtShim, setWxtShim] = useState<string | null>(null);
  const [isDevMode, setIsDevMode] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);
  const iframeRef = useRef<HTMLIFrameElement | null>(null);
  const [extensionInfo, setExtensionInfo] = useState<{
    manifest?: ExtensionManifest;
    version?: string;
    description?: string;
    permissions?: string[];
  } | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  // Load extension content and polyfill using Rust native API
  useEffect(() => {
    const loadExtension = async () => {
      setIsLoading(true);
      setError(null);

      try {
        // Check if auroraview is available
        if (!window.auroraview) {
          throw new Error('AuroraView bridge not available');
        }

        // Try to get the polyfill script from Rust plugin
        let polyfillScript = '';
        let wxtShimScript = '';
        
        try {
          const polyfillResult = await window.auroraview.invoke?.('plugin:extensions|get_polyfill', {
            extensionId,
          }) as { polyfill?: string; wxtShim?: string } | undefined;

          if (polyfillResult?.polyfill) {
            polyfillScript = polyfillResult.polyfill;
            wxtShimScript = polyfillResult.wxtShim || '';
          }
        } catch (e) {
          console.warn('[ExtensionContent] Polyfill not available, using minimal shim:', e);
          // Provide minimal chrome API shim
          polyfillScript = `
            window.chrome = window.chrome || {};
            window.chrome.runtime = window.chrome.runtime || {
              id: '${extensionId}',
              getManifest: () => ({}),
              sendMessage: () => Promise.resolve(),
              onMessage: { addListener: () => {}, removeListener: () => {} }
            };
            window.chrome.storage = window.chrome.storage || {
              local: {
                get: () => Promise.resolve({}),
                set: () => Promise.resolve(),
                remove: () => Promise.resolve()
              },
              sync: {
                get: () => Promise.resolve({}),
                set: () => Promise.resolve(),
                remove: () => Promise.resolve()
              }
            };
          `;
        }
        
        setPolyfill(polyfillScript);
        setWxtShim(wxtShimScript);

        // Try to load manifest.json
        const manifestPath = `${extensionPath}/manifest.json`.replace(/\\/g, '/');
        try {
          const manifestResult = await window.auroraview.invoke?.('plugin:fs|read_file', {
            path: manifestPath,
            encoding: 'utf-8',
          });

          // Handle both string and object formats
          let manifestContent: string | null = null;
          if (typeof manifestResult === 'string') {
            manifestContent = manifestResult;
          } else if (manifestResult && typeof manifestResult === 'object' && 'content' in manifestResult) {
            manifestContent = (manifestResult as { content: string }).content;
          }

          if (manifestContent) {
            const manifest = JSON.parse(manifestContent) as ExtensionManifest;
            setExtensionInfo({
              manifest,
              version: manifest.version,
              description: manifest.description,
              permissions: manifest.permissions,
            });
          }
        } catch (e) {
          console.warn('[ExtensionContent] Could not load manifest:', e);
        }

        // Load the side panel HTML
        const htmlPath = `${extensionPath}/${sidePanelPath}`.replace(/\\/g, '/');
        console.log('[ExtensionContent] Loading HTML from:', htmlPath);
        
        // Try plugin:fs|read_file first
        let htmlContent: string | null = null;
        try {
          // fs plugin returns the file content directly as a string
          const htmlResult = await window.auroraview.invoke?.('plugin:fs|read_file', {
            path: htmlPath,
            encoding: 'utf-8',
          });
          console.log('[ExtensionContent] plugin:fs|read_file result type:', typeof htmlResult, htmlResult ? 'has value' : 'empty');
          
          // Handle both string (direct content) and object ({ content: string }) formats
          if (typeof htmlResult === 'string') {
            htmlContent = htmlResult;
          } else if (htmlResult && typeof htmlResult === 'object' && 'content' in htmlResult) {
            htmlContent = (htmlResult as { content: string }).content;
          }
        } catch (e) {
          console.warn('[ExtensionContent] plugin:fs|read_file failed:', e);
        }

        // Fallback: try auroraview.call with fs.read_file
        if (!htmlContent) {
          try {
            const result = await window.auroraview.call?.('fs.read_file', { path: htmlPath }) as string | { content?: string } | undefined;
            if (typeof result === 'string') {
              htmlContent = result;
            } else if (result && typeof result === 'object' && 'content' in result && result.content) {
              htmlContent = result.content;
            }
          } catch (e) {
            console.warn('[ExtensionContent] fs.read_file failed:', e);
          }
        }

        // Fallback: try fetch with file:// URL
        if (!htmlContent) {
          try {
            const fileUrl = htmlPath.startsWith('/') ? `file://${htmlPath}` : `file:///${htmlPath}`;
            const response = await fetch(fileUrl);
            if (response.ok) {
              htmlContent = await response.text();
            }
          } catch (e) {
            console.warn('[ExtensionContent] fetch file:// failed:', e);
          }
        }

        if (htmlContent) {
          setHtmlContent(htmlContent);
        } else {
          throw new Error(`Failed to load side panel HTML from: ${htmlPath}\n\nThe extension's side panel file could not be loaded. Make sure the file exists and the path is correct.`);
        }
      } catch (e) {
        const errorMsg = e instanceof Error ? e.message : `Failed to load extension: ${e}`;
        console.error('[ExtensionContent]', errorMsg);
        setError(errorMsg);
        onError?.(errorMsg);
      } finally {
        setIsLoading(false);
      }
    };

    loadExtension();
  }, [extensionId, extensionPath, sidePanelPath, onError, reloadKey]);

  // Reload extension
  const handleReload = useCallback(() => {
    // Increment reload key to trigger useEffect re-run
    setReloadKey(k => k + 1);
    setIsLoading(true);
    setError(null);
    setHtmlContent(null);
    setPolyfill(null);
    setWxtShim(null);
  }, []);

  // Open in external browser
  const handleOpenExternal = useCallback(async () => {
    const normalizedPath = extensionPath.replace(/\\/g, '/');
    const fileUrl = `file:///${normalizedPath}/${sidePanelPath}`;
    
    if (onOpenInBrowser) {
      onOpenInBrowser(fileUrl);
    } else {
      // Fallback: try to use shell plugin or window.open
      try {
        await window.auroraview?.invoke?.('plugin:shell|open', { path: fileUrl });
      } catch (e) {
        console.warn('[ExtensionContent] shell plugin failed, using window.open:', e);
        window.open(fileUrl, '_blank');
      }
    }
  }, [extensionPath, sidePanelPath, onOpenInBrowser]);

  // Open DevTools for the iframe
  const handleOpenDevTools = useCallback(async () => {
    try {
      // Try to call Rust plugin to open DevTools
      await window.auroraview?.invoke?.('plugin:devtools|open', {
        targetId: `extension-${extensionId}`,
      });
    } catch (e) {
      console.warn('[ExtensionContent] DevTools plugin not available:', e);
      // Show a message to user
      alert('DevTools is not available in this environment. Try right-clicking and selecting "Inspect" in the extension content area.');
    }
  }, [extensionId]);

  // Open extension folder in file explorer
  const handleOpenFolder = useCallback(async () => {
    try {
      await window.auroraview?.invoke?.('plugin:shell|open', { path: extensionPath });
    } catch (e) {
      console.error('[ExtensionContent] Failed to open folder:', e);
    }
  }, [extensionPath]);

  // Process and inject HTML content
  useEffect(() => {
    if (!htmlContent || polyfill === null || !contentRef.current) return;

    const container = contentRef.current;
    const baseUrl = `https://auroraview.localhost/extension/${extensionId}/`;
    
    // Parse HTML
    const parser = new DOMParser();
    const doc = parser.parseFromString(htmlContent, 'text/html');

    // Check for WXT/Vite development mode scripts (localhost dev server references)
    const devServerPattern = /localhost:\d+/;
    let detectedDevMode = false;
    let removedDevScripts = 0;
    
    // Remove development server scripts and detect dev mode
    doc.querySelectorAll('script[src]').forEach((script) => {
      const src = script.getAttribute('src');
      if (src && devServerPattern.test(src)) {
        detectedDevMode = true;
        removedDevScripts++;
        script.remove();
        console.warn(`[ExtensionContent] Removed dev server script: ${src}`);
      }
    });
    
    // Also check for inline scripts with dev server references
    doc.querySelectorAll('script:not([src])').forEach((script) => {
      const content = script.textContent || '';
      if (devServerPattern.test(content)) {
        detectedDevMode = true;
      }
    });

    if (detectedDevMode) {
      console.warn(`[ExtensionContent] Detected WXT/Vite development mode extension. Removed ${removedDevScripts} dev server scripts.`);
      console.warn('[ExtensionContent] For production use, please build the extension with: wxt build');
      setIsDevMode(true);
    } else {
      setIsDevMode(false);
    }

    // Create polyfill script that will be injected first
    const polyfillScript = doc.createElement('script');
    polyfillScript.textContent = `
      // AuroraView Extension Environment
      (function() {
        'use strict';
        
        // Mark this as AuroraView environment
        window.__AURORAVIEW__ = true;
        window.__EXTENSION_ID__ = '${extensionId}';
        window.__EXTENSION_PATH__ = '${extensionPath.replace(/\\/g, '/')}';
        window.__EXTENSION_DEV_MODE__ = ${detectedDevMode};
        
        // Forward auroraview from parent
        if (window.parent && window.parent.auroraview) {
          window.auroraview = window.parent.auroraview;
        }
        
        // WXT Shim (must be before polyfill)
        ${wxtShim || ''}
        
        // Chrome API Polyfill
        ${polyfill}
        
        // Console wrapper for debugging
        const originalConsole = { ...console };
        ['log', 'warn', 'error', 'info', 'debug'].forEach(method => {
          const original = console[method];
          console[method] = function(...args) {
            original.apply(console, ['[Extension:${extensionId}]', ...args]);
          };
        });
        
        console.log('Extension environment initialized' + (${detectedDevMode} ? ' (DEV MODE - some features may not work)' : ''));
      })();
    `;
    
    // Insert polyfill at the beginning of head
    const head = doc.head || doc.documentElement;
    head.insertBefore(polyfillScript, head.firstChild);

    // Rewrite relative URLs in scripts (skip already processed or removed)
    doc.querySelectorAll('script[src]').forEach((script) => {
      const src = script.getAttribute('src');
      if (src && !src.startsWith('http') && !src.startsWith('//') && !src.startsWith('data:')) {
        const cleanSrc = src.replace(/^\.\//, '').replace(/^\//, '');
        script.setAttribute('src', `${baseUrl}${cleanSrc}`);
      }
    });

    // Rewrite relative URLs in stylesheets
    doc.querySelectorAll('link[rel="stylesheet"]').forEach((link) => {
      const href = link.getAttribute('href');
      if (href && !href.startsWith('http') && !href.startsWith('//') && !href.startsWith('data:')) {
        const cleanHref = href.replace(/^\.\//, '').replace(/^\//, '');
        link.setAttribute('href', `${baseUrl}${cleanHref}`);
      }
    });

    // Rewrite relative URLs in images
    doc.querySelectorAll('img[src]').forEach((img) => {
      const src = img.getAttribute('src');
      if (src && !src.startsWith('http') && !src.startsWith('//') && !src.startsWith('data:')) {
        const cleanSrc = src.replace(/^\.\//, '').replace(/^\//, '');
        img.setAttribute('src', `${baseUrl}${cleanSrc}`);
      }
    });

    // Get the processed HTML (body content only)
    const processedHead = doc.head.innerHTML;
    const processedBody = doc.body.innerHTML;

    // Clear container and render
    container.innerHTML = '';
    
    // Create an iframe for proper isolation
    const iframe = document.createElement('iframe');
    iframe.style.width = '100%';
    iframe.style.height = '100%';
    iframe.style.border = 'none';
    iframe.style.backgroundColor = 'transparent';
    iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-forms allow-popups');
    
    // Store iframe reference
    iframeRef.current = iframe;
    
    // Build the complete HTML document
    const fullHtml = `
      <!DOCTYPE html>
      <html>
        <head>
          <meta charset="UTF-8">
          <meta name="viewport" content="width=device-width, initial-scale=1.0">
          <base href="${baseUrl}">
          <style>
            /* Reset and base styles */
            *, *::before, *::after { box-sizing: border-box; }
            html, body { 
              margin: 0; 
              padding: 0;
              height: 100%;
              font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', sans-serif;
              font-size: 14px;
              line-height: 1.5;
              color: inherit;
              background: transparent;
            }
            /* Scrollbar styling */
            ::-webkit-scrollbar { width: 8px; height: 8px; }
            ::-webkit-scrollbar-track { background: transparent; }
            ::-webkit-scrollbar-thumb { background: rgba(128, 128, 128, 0.3); border-radius: 4px; }
            ::-webkit-scrollbar-thumb:hover { background: rgba(128, 128, 128, 0.5); }
            /* Dark mode support */
            @media (prefers-color-scheme: dark) {
              body { color: #e5e5e5; }
            }
          </style>
          ${processedHead}
        </head>
        <body>
          ${processedBody}
        </body>
      </html>
    `;

    // Use srcdoc for same-origin content
    iframe.srcdoc = fullHtml;
    container.appendChild(iframe);

    // Handle iframe load
    iframe.onload = () => {
      try {
        // Ensure auroraview is available in iframe
        if (iframe.contentWindow && window.auroraview) {
          (iframe.contentWindow as Window & { auroraview?: typeof window.auroraview }).auroraview = window.auroraview;
        }
        console.log('[ExtensionContent] Extension loaded successfully:', extensionId);
      } catch (e) {
        console.warn('[ExtensionContent] Could not inject auroraview into iframe:', e);
      }
    };

    iframe.onerror = (e) => {
      console.error('[ExtensionContent] Iframe error:', e);
    };

    return () => {
      container.innerHTML = '';
      iframeRef.current = null;
    };
  }, [htmlContent, polyfill, wxtShim, extensionId, extensionPath]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="flex flex-col items-center gap-3">
          <Icons.Loader2 className="w-8 h-8 animate-spin text-primary" />
          <span className="text-sm text-muted-foreground">Loading extension...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col h-full">
        {/* Error Header */}
        <div className="p-4 border-b border-border">
          <div className="flex items-center gap-2 text-destructive">
            <Icons.AlertCircle className="w-5 h-5" />
            <span className="font-medium">Extension Error</span>
          </div>
        </div>

        {/* Error Content */}
        <div className="flex-1 p-4 overflow-auto">
          <div className="p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
            <p className="text-sm text-destructive break-words">{error}</p>
          </div>

          {/* Extension Info */}
          <div className="mt-4 p-4 bg-muted/30 rounded-lg">
            <h4 className="text-sm font-medium mb-2">Extension Details</h4>
            <dl className="text-xs space-y-1">
              <div className="flex">
                <dt className="w-20 text-muted-foreground">Name:</dt>
                <dd>{extensionName}</dd>
              </div>
              <div className="flex">
                <dt className="w-20 text-muted-foreground">ID:</dt>
                <dd className="font-mono">{extensionId}</dd>
              </div>
              <div className="flex">
                <dt className="w-20 text-muted-foreground">Path:</dt>
                <dd className="font-mono truncate">{sidePanelPath}</dd>
              </div>
            </dl>
          </div>
        </div>

        {/* Actions */}
        <div className="p-4 border-t border-border space-y-2">
          <button
            onClick={handleReload}
            className={cn(
              "w-full flex items-center justify-center gap-2 px-4 py-2",
              "bg-primary text-primary-foreground rounded-lg",
              "hover:bg-primary/90 transition-colors",
              "text-sm font-medium"
            )}
          >
            <Icons.RefreshCw className="w-4 h-4" />
            Retry
          </button>
          <button
            onClick={handleOpenExternal}
            className={cn(
              "w-full flex items-center justify-center gap-2 px-4 py-2",
              "bg-muted text-foreground rounded-lg",
              "hover:bg-muted/80 transition-colors",
              "text-sm"
            )}
          >
            <Icons.ExternalLink className="w-4 h-4" />
            Open in Browser
          </button>
        </div>
      </div>
    );
  }

  // Render extension content
  return (
    <div className="flex flex-col h-full">
      {/* Dev Mode Warning Banner */}
      {isDevMode && (
        <div className="px-3 py-2 bg-amber-500/10 border-b border-amber-500/20 text-amber-600 dark:text-amber-400">
          <div className="flex items-center gap-2 text-xs">
            <Icons.AlertTriangle className="w-3.5 h-3.5 flex-shrink-0" />
            <span>
              Development mode extension detected. Some features may not work. 
              Build with <code className="px-1 py-0.5 bg-amber-500/20 rounded text-[10px]">wxt build</code> for production.
            </span>
          </div>
        </div>
      )}
      
      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-border bg-muted/20">
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Icons.Puzzle className="w-3.5 h-3.5" />
          <span className="truncate max-w-[150px]">{extensionName}</span>
          {extensionInfo?.version && (
            <span className="px-1.5 py-0.5 bg-muted rounded text-[10px]">
              v{extensionInfo.version}
            </span>
          )}
          {isDevMode && (
            <span className="px-1.5 py-0.5 bg-amber-500/20 text-amber-600 dark:text-amber-400 rounded text-[10px] font-medium">
              DEV
            </span>
          )}
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={handleOpenFolder}
            className="p-1 rounded hover:bg-muted transition-colors"
            title="Open extension folder"
          >
            <Icons.FolderOpen className="w-3.5 h-3.5 text-muted-foreground" />
          </button>
          <button
            onClick={handleOpenDevTools}
            className="p-1 rounded hover:bg-muted transition-colors"
            title="Open DevTools"
          >
            <Icons.Bug className="w-3.5 h-3.5 text-muted-foreground" />
          </button>
          <button
            onClick={handleReload}
            className="p-1 rounded hover:bg-muted transition-colors"
            title="Reload extension"
          >
            <Icons.RefreshCw className="w-3.5 h-3.5 text-muted-foreground" />
          </button>
          <button
            onClick={handleOpenExternal}
            className="p-1 rounded hover:bg-muted transition-colors"
            title="Open in browser"
          >
            <Icons.ExternalLink className="w-3.5 h-3.5 text-muted-foreground" />
          </button>
        </div>
      </div>

      {/* Extension Content Container */}
      <div 
        ref={contentRef}
        className="flex-1 overflow-hidden bg-background"
        style={{ minHeight: 0 }}
      />
    </div>
  );
}
