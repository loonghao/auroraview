/**
 * AuroraView Gallery API Type Definitions
 *
 * This file defines the contract between frontend and backend.
 * Both sides must adhere to these types to avoid parameter mismatches.
 *
 * IMPORTANT: When modifying these types, ensure the Python backend
 * (gallery/main.py) is updated accordingly.
 */

// =============================================================================
// Request Types (Frontend -> Backend)
// =============================================================================

/** Parameters for get_source API */
export interface GetSourceParams {
  sample_id: string;
}

/** Parameters for run_sample API */
export interface RunSampleParams {
  sample_id: string;
  show_console?: boolean;
}

/** Parameters for kill_process API */
export interface KillProcessParams {
  pid: number;
}

/** Parameters for send_to_process API */
export interface SendToProcessParams {
  pid: number;
  data: string;
}

/** Parameters for open_url API */
export interface OpenUrlParams {
  url: string;
}

// =============================================================================
// Response Types (Backend -> Frontend)
// =============================================================================

/** Base response for operations that can fail */
export interface ApiResponse {
  ok: boolean;
  error?: string;
}

/** Response from run_sample */
export interface RunSampleResponse extends ApiResponse {
  pid?: number;
  message?: string;
}

/** Response from list_processes */
export interface ListProcessesResponse extends ApiResponse {
  processes?: number[];
}

/** Sample definition */
export interface Sample {
  id: string;
  title: string;
  category: string;
  description: string;
  icon: string;
  source_file: string;
  tags: string[];
}

/** Category definition */
export interface Category {
  title: string;
  icon: string;
  description: string;
}

// =============================================================================
// Event Types (Backend -> Frontend)
// =============================================================================

/** Process stdout event data */
export interface ProcessStdoutEvent {
  pid: number;
  data: string;
}

/** Process stderr event data */
export interface ProcessStderrEvent {
  pid: number;
  data: string;
}

/** Process exit event data */
export interface ProcessExitEvent {
  pid: number;
  code: number;
}

// =============================================================================
// Type-safe API wrapper
// =============================================================================

/**
 * Type-safe wrapper for AuroraView API calls.
 *
 * Usage:
 *   const api = createTypedApi();
 *   const result = await api.killProcess({ pid: 12345 });
 */
export interface TypedGalleryApi {
  getSource(params: GetSourceParams): Promise<string>;
  runSample(params: RunSampleParams): Promise<RunSampleResponse>;
  killProcess(params: KillProcessParams): Promise<ApiResponse>;
  sendToProcess(params: SendToProcessParams): Promise<ApiResponse>;
  listProcesses(): Promise<ListProcessesResponse>;
  openUrl(params: OpenUrlParams): Promise<ApiResponse>;
  getSamples(): Promise<Sample[]>;
  getCategories(): Promise<Record<string, Category>>;
}

/**
 * Create a type-safe API wrapper.
 *
 * This ensures all API calls use the correct parameter format.
 */
export function createTypedApi(): TypedGalleryApi {
  const auroraview = (window as any).auroraview;

  if (!auroraview) {
    throw new Error("AuroraView not initialized");
  }

  return {
    async getSource(params: GetSourceParams): Promise<string> {
      return auroraview.api.get_source(params);
    },

    async runSample(params: RunSampleParams): Promise<RunSampleResponse> {
      return auroraview.api.run_sample(params);
    },

    async killProcess(params: KillProcessParams): Promise<ApiResponse> {
      // CRITICAL: Pass object with pid property, not just pid
      return auroraview.api.kill_process(params);
    },

    async sendToProcess(params: SendToProcessParams): Promise<ApiResponse> {
      return auroraview.api.send_to_process(params);
    },

    async listProcesses(): Promise<ListProcessesResponse> {
      return auroraview.api.list_processes();
    },

    async openUrl(params: OpenUrlParams): Promise<ApiResponse> {
      return auroraview.api.open_url(params);
    },

    async getSamples(): Promise<Sample[]> {
      return auroraview.api.get_samples();
    },

    async getCategories(): Promise<Record<string, Category>> {
      return auroraview.api.get_categories();
    },
  };
}

// =============================================================================
// Event subscription helpers
// =============================================================================

export type ProcessEventHandler<T> = (data: T) => void;

export interface TypedEventSubscriptions {
  onProcessStdout(handler: ProcessEventHandler<ProcessStdoutEvent>): () => void;
  onProcessStderr(handler: ProcessEventHandler<ProcessStderrEvent>): () => void;
  onProcessExit(handler: ProcessEventHandler<ProcessExitEvent>): () => void;
}

export function createTypedEvents(): TypedEventSubscriptions {
  const auroraview = (window as any).auroraview;

  if (!auroraview) {
    throw new Error("AuroraView not initialized");
  }

  return {
    onProcessStdout(handler: ProcessEventHandler<ProcessStdoutEvent>) {
      return auroraview.on("process:stdout", handler);
    },

    onProcessStderr(handler: ProcessEventHandler<ProcessStderrEvent>) {
      return auroraview.on("process:stderr", handler);
    },

    onProcessExit(handler: ProcessEventHandler<ProcessExitEvent>) {
      return auroraview.on("process:exit", handler);
    },
  };
}
