import { invoke } from "@tauri-apps/api/core";
import type {
  AppError,
  BabeldocInfo,
  ConnectionTestResult,
  ConnectivityRequest,
  ConvertRequest,
  MarkitdownInfo,
  ModelFetchResult,
  OfflineAssetsInfo,
  OfflineAssetsInstallResult,
  ProviderPayload,
  ProviderPreset,
  ProviderRecord,
  TaskRecord,
  TranslateRequest,
} from "../types";

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    const err = e as AppError | string;
    if (typeof err === "string") throw new Error(err);
    throw new Error(err.message || err.kind || "unknown error");
  }
}

// ---- provider ----
export const providerApi = {
  listPresets: () => call<ProviderPreset[]>("list_provider_presets"),
  list: () => call<ProviderRecord[]>("list_providers"),
  get: (id: string) => call<ProviderRecord | null>("get_provider", { id }),
  create: (payload: ProviderPayload) =>
    call<ProviderRecord>("create_provider", { payload }),
  update: (id: string, payload: ProviderPayload) =>
    call<ProviderRecord>("update_provider", { id, payload }),
  remove: (id: string) => call<void>("delete_provider", { id }),
  setDefault: (id: string) => call<void>("set_default_provider", { id }),
  revealKey: (api_key_id: string) =>
    call<string | null>("reveal_api_key", { apiKeyId: api_key_id }),
  testConnection: (req: ConnectivityRequest) =>
    call<ConnectionTestResult>("test_provider_connection", { req }),
  fetchModels: (req: ConnectivityRequest) =>
    call<ModelFetchResult>("fetch_provider_models", { req }),
};

// ---- translate ----
export const translateApi = {
  start: (req: TranslateRequest) => call<string>("start_translate", { req }),
  cancel: (task_id: string) => call<boolean>("cancel_translate", { taskId: task_id }),
  fileSize: (path: string) => call<number>("get_file_size", { path }),
  openFilePath: (path: string) => call<void>("open_file_path", { path }),
  revealFilePath: (path: string) => call<void>("reveal_file_path", { path }),
  babeldocInfo: () => call<BabeldocInfo>("get_babeldoc_info"),
  offlineAssetsInfo: () => call<OfflineAssetsInfo>("get_offline_assets_info"),
  installOfflineAssetsFromRelease: () =>
    call<OfflineAssetsInstallResult>("install_offline_assets_from_release"),
  installOfflineAssetsFromFile: (path: string) =>
    call<OfflineAssetsInstallResult>("install_offline_assets_from_file", { path }),
  listTaskRecords: () => call<TaskRecord[]>("list_task_records"),
  deleteTaskRecord: (id: string) => call<boolean>("delete_task_record", { id }),
};

// ---- convert (markitdown; peel-off module) ----
export const convertApi = {
  start: (req: ConvertRequest) => call<string>("start_convert", { req }),
  cancel: (task_id: string) => call<boolean>("cancel_convert", { taskId: task_id }),
  markitdownInfo: () => call<MarkitdownInfo>("get_markitdown_info"),
};

// ---- settings ----
export const settingsApi = {
  get: () => call<import("../types").AppSettings>("get_settings"),
  save: (settings: import("../types").AppSettings) =>
    call<import("../types").AppSettings>("save_settings", { settings }),
};
