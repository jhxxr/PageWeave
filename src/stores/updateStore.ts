import { getVersion } from "@tauri-apps/api/app";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { create } from "zustand";

export type UpdateStatus =
  | "idle"
  | "checking"
  | "upToDate"
  | "available"
  | "downloading"
  | "readyToInstall"
  | "installing"
  | "error";

interface UpdateState {
  appVersion: string;
  status: UpdateStatus;
  updateVersion?: string;
  releaseNotes?: string;
  downloadedBytes: number;
  contentLength?: number;
  lastCheckedAt?: string;
  error?: string;
  init: () => Promise<void>;
  checkForUpdates: (mode?: "background" | "manual") => Promise<void>;
  installAndRestart: () => Promise<void>;
}

let pendingUpdate: Update | null = null;
let bootstrapped = false;
let checking = false;

function errorMessage(e: unknown) {
  return e instanceof Error ? e.message : String(e);
}

function progressState(event: DownloadEvent) {
  const current = useUpdateStore.getState();
  switch (event.event) {
    case "Started":
      return {
        contentLength: event.data.contentLength,
        downloadedBytes: 0,
      };
    case "Progress":
      return {
        downloadedBytes: current.downloadedBytes + event.data.chunkLength,
      };
    case "Finished":
      return {};
  }
}

export const useUpdateStore = create<UpdateState>((set, get) => ({
  appVersion: "",
  status: "idle",
  downloadedBytes: 0,

  async init() {
    if (bootstrapped) return;
    bootstrapped = true;

    try {
      set({ appVersion: await getVersion() });
    } catch {
      set({ appVersion: "" });
    }

    void get().checkForUpdates("background");
  },

  async checkForUpdates(mode = "manual") {
    if (checking || get().status === "installing") return;
    if (pendingUpdate && get().status === "readyToInstall") return;

    checking = true;
    set({
      status: "checking",
      error: undefined,
      updateVersion: undefined,
      releaseNotes: undefined,
      downloadedBytes: 0,
      contentLength: undefined,
    });

    try {
      const update = await check();
      const checkedAt = new Date().toISOString();
      if (!update) {
        pendingUpdate = null;
        set({ status: "upToDate", lastCheckedAt: checkedAt });
        return;
      }

      pendingUpdate = update;
      set({
        status: "available",
        updateVersion: update.version,
        releaseNotes: update.body,
        lastCheckedAt: checkedAt,
      });

      set({ status: "downloading" });
      await update.download((event) => {
        set(progressState(event));
      });
      set({ status: "readyToInstall" });
    } catch (e) {
      pendingUpdate = null;
      set({
        status: mode === "background" ? "idle" : "error",
        error: mode === "background" ? undefined : errorMessage(e),
      });
    } finally {
      checking = false;
    }
  },

  async installAndRestart() {
    if (!pendingUpdate || get().status !== "readyToInstall") return;

    set({ status: "installing", error: undefined });
    try {
      await pendingUpdate.install();
      pendingUpdate = null;
      await relaunch();
    } catch (e) {
      set({ status: "error", error: errorMessage(e) });
    }
  },
}));
