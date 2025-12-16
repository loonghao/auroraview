/**
 * AuroraView React Hooks for Gallery
 *
 * This file re-exports hooks from @auroraview/sdk and adds Gallery-specific
 * API wrappers for sample management.
 */

import { useState, useEffect, useCallback } from 'react';
import {
  useAuroraView as useAuroraViewBase,
  useProcessEvents as useProcessEventsBase,
  type ProcessOutput,
  type ProcessExit,
} from '@auroraview/sdk/react';

// Re-export types from SDK
export type { ProcessOutput, ProcessExit } from '@auroraview/sdk/react';

// Gallery-specific types
export interface RunOptions {
  showConsole?: boolean;
}

export interface ApiResult {
  ok: boolean;
  error?: string;
  pid?: number;
}

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

export interface ProcessInfo {
  pid: number;
  sampleId: string;
  title: string;
  startTime: number;
}

/**
 * Gallery-specific hook that wraps SDK's useAuroraView with Gallery API methods
 */
export function useAuroraView() {
  const { client, isReady } = useAuroraViewBase();

  const getSource = useCallback(async (sampleId: string): Promise<string> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<string>('api.get_source', { sample_id: sampleId });
  }, [client]);

  const runSample = useCallback(async (sampleId: string, options?: RunOptions): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<ApiResult>('api.run_sample', {
      sample_id: sampleId,
      show_console: options?.showConsole ?? false,
    });
  }, [client]);

  const getSamples = useCallback(async (): Promise<Sample[]> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<Sample[]>('api.get_samples');
  }, [client]);

  const getCategories = useCallback(async (): Promise<Record<string, Category>> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<Record<string, Category>>('api.get_categories');
  }, [client]);

  const openUrl = useCallback(async (url: string): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<ApiResult>('api.open_url', { url });
  }, [client]);

  const openInWebView = useCallback((url: string, title?: string) => {
    // Use native window.open() - WebView2 will handle creating a new browser window
    const windowName = title ?? 'AuroraView';
    const features = 'width=1024,height=768,menubar=no,toolbar=no,location=yes,status=no';
    window.open(url, windowName, features);
  }, []);

  const killProcess = useCallback(async (pid: number): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<ApiResult>('api.kill_process', { pid });
  }, [client]);

  const sendToProcess = useCallback(async (pid: number, data: string) => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('api.send_to_process', { pid, data });
  }, [client]);

  const listProcesses = useCallback(async () => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('api.list_processes');
  }, [client]);

  return {
    isReady,
    getSource,
    runSample,
    getSamples,
    getCategories,
    openUrl,
    openInWebView,
    killProcess,
    sendToProcess,
    listProcesses,
  };
}

/**
 * Re-export useProcessEvents from SDK with Gallery-compatible interface
 */
export function useProcessEvents(options?: {
  onStdout?: (data: ProcessOutput) => void;
  onStderr?: (data: ProcessOutput) => void;
  onExit?: (data: ProcessExit) => void;
}) {
  const [isSubscribed, setIsSubscribed] = useState(false);

  // Use SDK's useProcessEvents
  useProcessEventsBase({
    onStdout: options?.onStdout,
    onStderr: options?.onStderr,
    onExit: options?.onExit,
  });

  useEffect(() => {
    setIsSubscribed(true);
    return () => setIsSubscribed(false);
  }, []);

  return { isSubscribed };
}
