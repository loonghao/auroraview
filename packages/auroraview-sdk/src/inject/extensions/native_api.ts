/**
 * Native API Bridge
 * 
 * Provides communication with AuroraView's native extension host.
 */

declare global {
  interface Window {
    auroraview?: {
      invoke?: (command: string, params: any) => Promise<any>;
      on?: (event: string, handler: (data: any) => void) => void;
    };
  }
}

/**
 * Call native extension API through AuroraView bridge
 */
export async function callNativeApi(
  extensionId: string,
  api: string,
  method: string,
  params: Record<string, any> = {}
): Promise<any> {
  if (typeof window.auroraview?.invoke !== 'function') {
    console.warn('[Chrome API] AuroraView bridge not available');
    return null;
  }
  try {
    return await window.auroraview.invoke('plugin:extensions|api_call', {
      extensionId,
      api,
      method,
      params,
    });
  } catch (e) {
    console.error(`[Chrome API] ${api}.${method} failed:`, e);
    throw e;
  }
}

/**
 * Create a bound native API caller for a specific extension
 */
export function createNativeApiCaller(extensionId: string) {
  return (api: string, method: string, params: Record<string, any> = {}) =>
    callNativeApi(extensionId, api, method, params);
}

/**
 * Promise wrapper for callback-style APIs
 */
export function promisify<T>(
  fn: (...args: any[]) => void,
  getLastError: () => { message: string } | null
): (...args: any[]) => Promise<T> {
  return function (...args: any[]): Promise<T> {
    return new Promise((resolve, reject) => {
      fn(...args, (result: T) => {
        const lastError = getLastError();
        if (lastError) {
          reject(new Error(lastError.message));
        } else {
          resolve(result);
        }
      });
    });
  };
}
