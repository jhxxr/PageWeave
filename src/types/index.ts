// Types mirrored from src-tauri Rust structs. Keep field names in sync.

export type ProviderCategory =
  | "openai"
  | "deepseek"
  | "siliconflow"
  | "qwen"
  | "moonshot"
  | "zhipu"
  | "ollama"
  | "custom";

export interface ProviderRecord {
  id: string;
  name: string;
  category: ProviderCategory;
  base_url: string;
  api_key_id: string;
  has_api_key: boolean;
  models: string[];
  default_model: string;
  is_enabled: boolean;
  is_applied: boolean;
  sort_index: number;
  notes: string;
  extra: unknown;
  created_at: string;
  updated_at: string;
}

export interface ProviderPayload {
  name: string;
  category: ProviderCategory;
  base_url: string;
  /** Empty string = keep existing key; non-empty = write/replace. */
  api_key: string;
  models: string[];
  default_model: string;
  is_enabled: boolean;
  notes: string;
  extra?: unknown;
}

export interface ProviderPreset {
  category: ProviderCategory;
  label: string;
  base_url: string;
  models: string[];
}

export interface ConnectivityRequest {
  api_key_id: string;
  /** Plaintext key for unsaved provider forms; never persisted by test/fetch calls. */
  api_key?: string;
  base_url: string;
  model?: string;
}

export interface ConnectionTestResult {
  ok: boolean;
  message: string;
  latency_ms?: number;
}

export interface ModelFetchResult {
  ok: boolean;
  models: string[];
  message: string;
}

export interface AppError {
  kind: string;
  message: string;
}

export type OutputMode = "mono" | "dual" | "both";

export interface TranslateRequest {
  task_id?: string;
  pdf_paths: string[];
  output_dir: string;
  lang_in: string;
  lang_out: string;
  output_mode: OutputMode;
  provider: { base_url: string; api_key_id: string; model: string };
  qps: number;
}

export interface BabeldocInfo {
  installed: boolean;
  version?: string;
  path?: string;
  hint: string;
}

export interface OfflineAssetsInfo {
  installed: boolean;
  cache_dir: string;
  size_bytes: number;
  message: string;
}

export interface OfflineAssetsInstallResult {
  ok: boolean;
  cache_dir: string;
  asset_name?: string;
  message: string;
}

export type TranslateEvent =
  | { task_id: string; type: "log"; line: string; stream: string }
  | {
      task_id: string;
      type: "progress";
      overall: number;
      stage: string;
      part_index?: number;
      total_parts?: number;
    }
  | {
      task_id: string;
      type: "status";
      status: string;
      output_files?: string[];
      message?: string;
    };

export interface AppSettings {
  theme: string; // light | dark | system
  language: string; // zh | en
  default_output_dir: string;
  default_lang_in: string;
  default_lang_out: string;
  default_provider_id: string;
  log_retention_days: number;
  cache_dir: string;
}
