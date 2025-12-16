/**
 * AuroraView SDK React Adapter
 *
 * Provides React hooks for the AuroraView bridge API.
 */

import { useEffect, useState, useRef, useCallback, useMemo } from 'react';
import { createAuroraView, type AuroraViewClient } from '../core/bridge';
import type {
  EventHandler,
  Unsubscribe,
  ProcessOutput,
  ProcessExit,
} from '../core/types';

/**
 * Hook to get the AuroraView client
 *
 * @example
 * ```tsx
 * function App() {
 *   const { client, isReady } = useAuroraView();
 *
 *   const handleClick = async () => {
 *     const result = await client.call('api.echo', { message: 'Hello' });
 *     console.log(result);
 *   };
 *
 *   return <button onClick={handleClick} disabled={!isReady}>Call API</button>;
 * }
 * ```
 */
export function useAuroraView(): {
  client: AuroraViewClient;
  isReady: boolean;
} {
  const [isReady, setIsReady] = useState(false);
  const client = useMemo(() => createAuroraView(), []);

  useEffect(() => {
    // Check initial state
    if (client.isReady()) {
      setIsReady(true);
    } else {
      client.whenReady().then(() => setIsReady(true));
    }
  }, [client]);

  return { client, isReady };
}

/**
 * Hook to subscribe to an event
 *
 * @param event - Event name to subscribe to
 * @param handler - Event handler function
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   useAuroraEvent('custom:event', (data) => {
 *     console.log('Received:', data);
 *   });
 *
 *   return <div>Listening for events...</div>;
 * }
 * ```
 */
export function useAuroraEvent<T = unknown>(
  event: string,
  handler: EventHandler<T>
): void {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    const client = createAuroraView();

    const unsubscribe = client.on<T>(event, (data) => {
      handlerRef.current(data);
    });

    return unsubscribe;
  }, [event]);
}

/**
 * Hook to subscribe to multiple events
 *
 * @param events - Map of event names to handlers
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   useAuroraEvents({
 *     'user:login': (data) => console.log('Login:', data),
 *     'user:logout': () => console.log('Logged out'),
 *   });
 *
 *   return <div>Listening...</div>;
 * }
 * ```
 */
export function useAuroraEvents(
  events: Record<string, EventHandler>
): void {
  const eventsRef = useRef(events);
  eventsRef.current = events;

  useEffect(() => {
    const client = createAuroraView();
    const unsubscribers: Unsubscribe[] = [];

    Object.entries(eventsRef.current).forEach(([event]) => {
      const unsub = client.on(event, (data) => {
        eventsRef.current[event]?.(data);
      });
      unsubscribers.push(unsub);
    });

    return () => {
      unsubscribers.forEach((unsub) => unsub());
    };
  }, [Object.keys(events).join(',')]);
}

/**
 * Options for useProcessEvents hook
 */
export interface ProcessEventsOptions {
  /** Handler for stdout data */
  onStdout?: (data: ProcessOutput) => void;
  /** Handler for stderr data */
  onStderr?: (data: ProcessOutput) => void;
  /** Handler for process exit */
  onExit?: (data: ProcessExit) => void;
  /** Filter by specific process ID */
  pid?: number;
}

/**
 * Hook to subscribe to process events
 *
 * @example
 * ```tsx
 * function ProcessMonitor() {
 *   const [output, setOutput] = useState<string[]>([]);
 *
 *   useProcessEvents({
 *     onStdout: (data) => setOutput(prev => [...prev, data.data]),
 *     onStderr: (data) => setOutput(prev => [...prev, `[ERR] ${data.data}`]),
 *     onExit: (data) => console.log(`Process ${data.pid} exited with code ${data.code}`),
 *   });
 *
 *   return <pre>{output.join('')}</pre>;
 * }
 * ```
 */
export function useProcessEvents(options: ProcessEventsOptions = {}): void {
  const optionsRef = useRef(options);
  optionsRef.current = options;

  useEffect(() => {
    const client = createAuroraView();
    const unsubscribers: Unsubscribe[] = [];

    if (optionsRef.current.onStdout) {
      const unsub = client.on<ProcessOutput>('process:stdout', (data) => {
        if (
          optionsRef.current.pid === undefined ||
          optionsRef.current.pid === data.pid
        ) {
          optionsRef.current.onStdout?.(data);
        }
      });
      unsubscribers.push(unsub);
    }

    if (optionsRef.current.onStderr) {
      const unsub = client.on<ProcessOutput>('process:stderr', (data) => {
        if (
          optionsRef.current.pid === undefined ||
          optionsRef.current.pid === data.pid
        ) {
          optionsRef.current.onStderr?.(data);
        }
      });
      unsubscribers.push(unsub);
    }

    if (optionsRef.current.onExit) {
      const unsub = client.on<ProcessExit>('process:exit', (data) => {
        if (
          optionsRef.current.pid === undefined ||
          optionsRef.current.pid === data.pid
        ) {
          optionsRef.current.onExit?.(data);
        }
      });
      unsubscribers.push(unsub);
    }

    return () => {
      unsubscribers.forEach((unsub) => unsub());
    };
  }, []);
}

/**
 * Hook to call an API method
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { execute, loading, error, data } = useAuroraCall<string>('api.greet');
 *
 *   return (
 *     <div>
 *       <button onClick={() => execute({ name: 'World' })} disabled={loading}>
 *         Greet
 *       </button>
 *       {error && <p>Error: {error.message}</p>}
 *       {data && <p>Result: {data}</p>}
 *     </div>
 *   );
 * }
 * ```
 */
export function useAuroraCall<T = unknown>(method: string): {
  execute: (params?: unknown) => Promise<T>;
  loading: boolean;
  error: Error | null;
  data: T | null;
  reset: () => void;
} {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [data, setData] = useState<T | null>(null);

  const execute = useCallback(
    async (params?: unknown): Promise<T> => {
      const client = createAuroraView();
      setLoading(true);
      setError(null);

      try {
        const result = await client.call<T>(method, params);
        setData(result);
        return result;
      } catch (e) {
        const err = e instanceof Error ? e : new Error(String(e));
        setError(err);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [method]
  );

  const reset = useCallback(() => {
    setLoading(false);
    setError(null);
    setData(null);
  }, []);

  return { execute, loading, error, data, reset };
}

/**
 * Hook to invoke a plugin command
 *
 * @example
 * ```tsx
 * function FileReader() {
 *   const { execute, loading, data } = useAuroraInvoke<string>('plugin:fs|read_file');
 *
 *   return (
 *     <div>
 *       <button onClick={() => execute({ path: '/tmp/test.txt' })}>
 *         Read File
 *       </button>
 *       {data && <pre>{data}</pre>}
 *     </div>
 *   );
 * }
 * ```
 */
export function useAuroraInvoke<T = unknown>(cmd: string): {
  execute: (args?: Record<string, unknown>) => Promise<T>;
  loading: boolean;
  error: Error | null;
  data: T | null;
  reset: () => void;
} {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [data, setData] = useState<T | null>(null);

  const execute = useCallback(
    async (args?: Record<string, unknown>): Promise<T> => {
      const client = createAuroraView();
      setLoading(true);
      setError(null);

      try {
        const result = await client.invoke<T>(cmd, args);
        setData(result);
        return result;
      } catch (e) {
        const err = e instanceof Error ? e : new Error(String(e));
        setError(err);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [cmd]
  );

  const reset = useCallback(() => {
    setLoading(false);
    setError(null);
    setData(null);
  }, []);

  return { execute, loading, error, data, reset };
}

/**
 * Hook to access shared state
 *
 * @example
 * ```tsx
 * function ThemeToggle() {
 *   const [theme, setTheme] = useAuroraState<string>('theme', 'light');
 *
 *   return (
 *     <button onClick={() => setTheme(theme === 'light' ? 'dark' : 'light')}>
 *       Current: {theme}
 *     </button>
 *   );
 * }
 * ```
 */
export function useAuroraState<T>(
  key: string,
  defaultValue?: T
): [T | undefined, (value: T) => void] {
  const client = useMemo(() => createAuroraView(), []);
  const [value, setValue] = useState<T | undefined>(() => {
    const state = client.state;
    return state ? (state[key] as T) ?? defaultValue : defaultValue;
  });

  useEffect(() => {
    const state = client.state;
    if (!state) return;

    // Initial value
    if (state[key] !== undefined) {
      setValue(state[key] as T);
    }

    // Subscribe to changes
    const unsubscribe = state.onChange((changedKey: string, newValue: unknown) => {
      if (changedKey === key) {
        setValue(newValue as T);
      }
    });

    return unsubscribe;
  }, [client, key]);

  const setStateValue = useCallback(
    (newValue: T) => {
      const state = client.state;
      if (state) {
        state[key] = newValue;
      }
      setValue(newValue);
    },
    [client, key]
  );

  return [value, setStateValue];
}

// Re-export types for convenience
export type { ProcessOutput, ProcessExit } from '../core/types';
export type { AuroraViewClient } from '../core/bridge';
