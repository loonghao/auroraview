import { useState, useEffect, useCallback, useRef } from 'react';

export interface RunOptions {
  showConsole?: boolean;
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

export interface ProcessOutput {
  pid: number;
  data: string;
}

export interface ProcessExit {
  pid: number;
  code: number | null;
}

export interface ProcessInfo {
  pid: number;
  sampleId: string;
  title: string;
  startTime: number;
}

export function useAuroraView() {
  const [isReady, setIsReady] = useState(false);

  useEffect(() => {
    const checkReady = () => {
      if (window.auroraview) {
        setIsReady(true);
      }
    };

    // Check immediately
    checkReady();

    // Listen for ready event
    const handleReady = () => setIsReady(true);
    window.addEventListener('auroraviewready', handleReady);

    return () => {
      window.removeEventListener('auroraviewready', handleReady);
    };
  }, []);

  const getSource = useCallback(async (sampleId: string): Promise<string> => {
    if (!window.auroraview) {
      throw new Error('AuroraView not ready');
    }
    return window.auroraview.api.get_source({ sample_id: sampleId });
  }, []);

  const runSample = useCallback(async (sampleId: string, options?: RunOptions) => {
    if (!window.auroraview) {
      throw new Error('AuroraView not ready');
    }
    return window.auroraview.api.run_sample({
      sample_id: sampleId,
      show_console: options?.showConsole ?? false,
    });
  }, []);

  const getSamples = useCallback(async (): Promise<Sample[]> => {
    if (!window.auroraview) {
      throw new Error('AuroraView not ready');
    }
    return window.auroraview.api.get_samples();
  }, []);

  const getCategories = useCallback(async (): Promise<Record<string, Category>> => {
    if (!window.auroraview) {
      throw new Error('AuroraView not ready');
    }
    return window.auroraview.api.get_categories();
  }, []);

  const openUrl = useCallback(async (url: string) => {
    if (!window.auroraview) {
      throw new Error('AuroraView not ready');
    }
    return window.auroraview.api.open_url({ url });
  }, []);

  const openInWebView = useCallback((url: string, title?: string) => {
    // Use native window.open() - WebView2 will handle creating a new browser window
    // This is enabled by allow_new_window=True in the Python WebView config
    const windowName = title ?? 'AuroraView';
    const features = 'width=1024,height=768,menubar=no,toolbar=no,location=yes,status=no';
    window.open(url, windowName, features);
  }, []);

  const killProcess = useCallback(async (pid: number) => {
    if (!window.auroraview) {
      throw new Error('AuroraView not ready');
    }
    return window.auroraview.api.kill_process({ pid });
  }, []);

  const sendToProcess = useCallback(async (pid: number, data: string) => {
    if (!window.auroraview) {
      throw new Error('AuroraView not ready');
    }
    return window.auroraview.api.send_to_process({ pid, data });
  }, []);

  const listProcesses = useCallback(async () => {
    if (!window.auroraview) {
      throw new Error('AuroraView not ready');
    }
    return window.auroraview.api.list_processes();
  }, []);

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
 * Hook to subscribe to process events (stdout, stderr, exit)
 */
export function useProcessEvents(options?: {
  onStdout?: (data: ProcessOutput) => void;
  onStderr?: (data: ProcessOutput) => void;
  onExit?: (data: ProcessExit) => void;
}) {
  const [isSubscribed, setIsSubscribed] = useState(false);
  const optionsRef = useRef(options);
  optionsRef.current = options;

  useEffect(() => {
    const subscribe = () => {
      if (!window.auroraview || isSubscribed) return;

      const unsubscribers: (() => void)[] = [];

      // Always subscribe to all events
      const unsubStdout = window.auroraview.on('process:stdout', (data: unknown) => {
        optionsRef.current?.onStdout?.(data as ProcessOutput);
      });
      if (unsubStdout) unsubscribers.push(unsubStdout);

      const unsubStderr = window.auroraview.on('process:stderr', (data: unknown) => {
        optionsRef.current?.onStderr?.(data as ProcessOutput);
      });
      if (unsubStderr) unsubscribers.push(unsubStderr);

      const unsubExit = window.auroraview.on('process:exit', (data: unknown) => {
        optionsRef.current?.onExit?.(data as ProcessExit);
      });
      if (unsubExit) unsubscribers.push(unsubExit);

      setIsSubscribed(true);

      return () => {
        unsubscribers.forEach(unsub => unsub());
        setIsSubscribed(false);
      };
    };

    // Try to subscribe immediately
    const cleanup = subscribe();

    // Also listen for ready event in case auroraview isn't ready yet
    const handleReady = () => {
      subscribe();
    };
    window.addEventListener('auroraviewready', handleReady);

    return () => {
      window.removeEventListener('auroraviewready', handleReady);
      cleanup?.();
    };
  }, [isSubscribed]);
}
