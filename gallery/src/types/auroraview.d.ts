// AuroraView JavaScript Bridge Types

interface RunSampleParams {
  sample_id: string;
  show_console?: boolean;
}

interface OpenInWebViewParams {
  url: string;
  title?: string;
}

interface SendToProcessParams {
  pid: number;
  data: string;
}

interface KillProcessParams {
  pid: number;
}

interface OpenUrlParams {
  url: string;
}

interface GetSourceParams {
  sample_id: string;
}

interface AuroraViewAPI {
  get_source: (params: GetSourceParams) => Promise<string>;
  run_sample: (params: RunSampleParams) => Promise<{ ok: boolean; message?: string; error?: string; pid?: number }>;
  get_samples: () => Promise<Sample[]>;
  get_categories: () => Promise<Record<string, Category>>;
  open_url: (params: OpenUrlParams) => Promise<{ ok: boolean; error?: string }>;
  open_in_webview: (params: OpenInWebViewParams) => Promise<{ ok: boolean; message?: string; error?: string }>;
  kill_process: (params: KillProcessParams) => Promise<{ ok: boolean; error?: string }>;
  send_to_process: (params: SendToProcessParams) => Promise<{ ok: boolean; error?: string }>;
  list_processes: () => Promise<{ ok: boolean; processes: number[] }>;
}

interface AuroraView {
  call: (method: string, params?: unknown) => Promise<unknown>;
  on: (event: string, handler: (payload: unknown) => void) => () => void;
  off: (event: string, handler: (payload: unknown) => void) => void;
  api: AuroraViewAPI;
  trigger: (event: string, data: unknown) => void;
}

interface Sample {
  id: string;
  title: string;
  category: string;
  description: string;
  icon: string;
  source_file: string;
  tags?: string[];
}

interface Category {
  title: string;
  icon: string;
  description: string;
}

declare global {
  interface Window {
    auroraview?: AuroraView;
  }
}

export {};
