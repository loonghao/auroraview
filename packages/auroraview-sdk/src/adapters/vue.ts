/**
 * AuroraView SDK Vue Adapter
 *
 * Provides Vue 3 composables for the AuroraView bridge API.
 */

import {
  ref,
  onMounted,
  onUnmounted,
  watch,
  type Ref,
} from 'vue';
import { createAuroraView, type AuroraViewClient } from '../core/bridge';
import type {
  EventHandler,
  Unsubscribe,
  ProcessOutput,
  ProcessExit,
} from '../core/types';

/** Singleton client instance */
let clientInstance: AuroraViewClient | null = null;

function getClient(): AuroraViewClient {
  if (!clientInstance) {
    clientInstance = createAuroraView();
  }
  return clientInstance;
}

/**
 * Composable to get the AuroraView client
 *
 * @example
 * ```vue
 * <script setup lang="ts">
 * import { useAuroraView } from '@aspect/auroraview-sdk/vue';
 *
 * const { client, isReady } = useAuroraView();
 *
 * async function handleClick() {
 *   const result = await client.value?.call('api.echo', { message: 'Hello' });
 *   console.log(result);
 * }
 * </script>
 * ```
 */
export function useAuroraView(): {
  client: Ref<AuroraViewClient>;
  isReady: Ref<boolean>;
} {
  const client = ref<AuroraViewClient>(getClient());
  const isReady = ref(false);

  onMounted(async () => {
    if (client.value.isReady()) {
      isReady.value = true;
    } else {
      await client.value.whenReady();
      isReady.value = true;
    }
  });

  return { client: client as Ref<AuroraViewClient>, isReady };
}

/**
 * Composable to subscribe to an event
 *
 * @example
 * ```vue
 * <script setup lang="ts">
 * import { useAuroraEvent } from '@aspect/auroraview-sdk/vue';
 *
 * useAuroraEvent('custom:event', (data) => {
 *   console.log('Received:', data);
 * });
 * </script>
 * ```
 */
export function useAuroraEvent<T = unknown>(
  event: string,
  handler: EventHandler<T>
): void {
  let unsubscribe: Unsubscribe | null = null;

  onMounted(() => {
    const client = getClient();
    unsubscribe = client.on<T>(event, handler);
  });

  onUnmounted(() => {
    unsubscribe?.();
  });
}

/**
 * Composable to subscribe to multiple events
 */
export function useAuroraEvents(
  events: Record<string, EventHandler>
): void {
  const unsubscribers: Unsubscribe[] = [];

  onMounted(() => {
    const client = getClient();
    Object.entries(events).forEach(([event, handler]) => {
      unsubscribers.push(client.on(event, handler));
    });
  });

  onUnmounted(() => {
    unsubscribers.forEach((unsub) => unsub());
  });
}

/**
 * Options for useProcessEvents composable
 */
export interface ProcessEventsOptions {
  onStdout?: (data: ProcessOutput) => void;
  onStderr?: (data: ProcessOutput) => void;
  onExit?: (data: ProcessExit) => void;
  pid?: number;
}

/**
 * Composable to subscribe to process events
 *
 * @example
 * ```vue
 * <script setup lang="ts">
 * import { ref } from 'vue';
 * import { useProcessEvents } from '@aspect/auroraview-sdk/vue';
 *
 * const output = ref<string[]>([]);
 *
 * useProcessEvents({
 *   onStdout: (data) => output.value.push(data.data),
 *   onExit: (data) => console.log(`Exited with code ${data.code}`),
 * });
 * </script>
 * ```
 */
export function useProcessEvents(options: ProcessEventsOptions = {}): void {
  const unsubscribers: Unsubscribe[] = [];

  onMounted(() => {
    const client = getClient();

    if (options.onStdout) {
      unsubscribers.push(
        client.on<ProcessOutput>('process:stdout', (data) => {
          if (options.pid === undefined || options.pid === data.pid) {
            options.onStdout?.(data);
          }
        })
      );
    }

    if (options.onStderr) {
      unsubscribers.push(
        client.on<ProcessOutput>('process:stderr', (data) => {
          if (options.pid === undefined || options.pid === data.pid) {
            options.onStderr?.(data);
          }
        })
      );
    }

    if (options.onExit) {
      unsubscribers.push(
        client.on<ProcessExit>('process:exit', (data) => {
          if (options.pid === undefined || options.pid === data.pid) {
            options.onExit?.(data);
          }
        })
      );
    }
  });

  onUnmounted(() => {
    unsubscribers.forEach((unsub) => unsub());
  });
}

/**
 * Composable to call an API method
 *
 * @example
 * ```vue
 * <script setup lang="ts">
 * import { useAuroraCall } from '@aspect/auroraview-sdk/vue';
 *
 * const { execute, loading, error, data } = useAuroraCall<string>('api.greet');
 * </script>
 *
 * <template>
 *   <button @click="execute({ name: 'World' })" :disabled="loading">Greet</button>
 *   <p v-if="data">{{ data }}</p>
 * </template>
 * ```
 */
export function useAuroraCall<T = unknown>(method: string): {
  execute: (params?: unknown) => Promise<T>;
  loading: Ref<boolean>;
  error: Ref<Error | null>;
  data: Ref<T | null>;
  reset: () => void;
} {
  const loading = ref(false);
  const error = ref<Error | null>(null);
  const data = ref<T | null>(null) as Ref<T | null>;

  const execute = async (params?: unknown): Promise<T> => {
    const client = getClient();
    loading.value = true;
    error.value = null;

    try {
      const result = await client.call<T>(method, params);
      data.value = result;
      return result;
    } catch (e) {
      const err = e instanceof Error ? e : new Error(String(e));
      error.value = err;
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const reset = () => {
    loading.value = false;
    error.value = null;
    data.value = null;
  };

  return { execute, loading, error, data, reset };
}

/**
 * Composable to invoke a plugin command
 */
export function useAuroraInvoke<T = unknown>(cmd: string): {
  execute: (args?: Record<string, unknown>) => Promise<T>;
  loading: Ref<boolean>;
  error: Ref<Error | null>;
  data: Ref<T | null>;
  reset: () => void;
} {
  const loading = ref(false);
  const error = ref<Error | null>(null);
  const data = ref<T | null>(null) as Ref<T | null>;

  const execute = async (args?: Record<string, unknown>): Promise<T> => {
    const client = getClient();
    loading.value = true;
    error.value = null;

    try {
      const result = await client.invoke<T>(cmd, args);
      data.value = result;
      return result;
    } catch (e) {
      const err = e instanceof Error ? e : new Error(String(e));
      error.value = err;
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const reset = () => {
    loading.value = false;
    error.value = null;
    data.value = null;
  };

  return { execute, loading, error, data, reset };
}

/**
 * Composable to access shared state
 *
 * @example
 * ```vue
 * <script setup lang="ts">
 * import { useAuroraState } from '@aspect/auroraview-sdk/vue';
 *
 * const theme = useAuroraState<string>('theme', 'light');
 * </script>
 *
 * <template>
 *   <button @click="theme = theme === 'light' ? 'dark' : 'light'">
 *     Current: {{ theme }}
 *   </button>
 * </template>
 * ```
 */
export function useAuroraState<T>(
  key: string,
  defaultValue?: T
): Ref<T | undefined> {
  const client = getClient();
  const value = ref<T | undefined>(defaultValue) as Ref<T | undefined>;

  onMounted(() => {
    const state = client.state;
    if (!state) return;

    // Get initial value
    if (state[key] !== undefined) {
      value.value = state[key] as T;
    }

    // Subscribe to changes from Python
    const unsubscribe = state.onChange((changedKey, newValue) => {
      if (changedKey === key) {
        value.value = newValue as T;
      }
    });

    onUnmounted(unsubscribe);
  });

  // Watch for local changes and sync to state
  watch(value, (newValue) => {
    const state = client.state;
    if (state && state[key] !== newValue) {
      state[key] = newValue;
    }
  });

  return value;
}

// Re-export types
export type { ProcessOutput, ProcessExit } from '../core/types';
export type { AuroraViewClient } from '../core/bridge';
