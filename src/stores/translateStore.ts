import { create } from "zustand";

export type TaskStatus =
  | "idle"
  | "running"
  | "success"
  | "error"
  | "cancelled";

export interface FileItem {
  /** Absolute path. */
  path: string;
  name: string;
  size: number;
  status: TaskStatus;
}

export interface LogLine {
  id: number;
  text: string;
  stream: string;
}

interface TranslateState {
  files: FileItem[];
  outputDir: string;
  langIn: string;
  langOut: string;
  outputMode: "mono" | "dual" | "both";
  providerId: string;
  model: string;
  qps: number;
  taskId: string | null;
  status: TaskStatus;
  progress: number;
  stage: string;
  logs: LogLine[];
  outputFiles: string[];
  statusMessage: string;
  babeldocInstalled: boolean | null;
  babeldocHint: string;

  setFiles: (f: FileItem[]) => void;
  addFiles: (f: FileItem[]) => void;
  removeFile: (path: string) => void;
  setOutputDir: (d: string) => void;
  setLangIn: (l: string) => void;
  setLangOut: (l: string) => void;
  setOutputMode: (m: "mono" | "dual" | "both") => void;
  setProviderId: (id: string) => void;
  setModel: (m: string) => void;
  setQps: (q: number) => void;
  setTaskId: (id: string | null) => void;
  setStatus: (s: TaskStatus) => void;
  setProgress: (p: number, stage?: string) => void;
  appendLog: (text: string, stream: string) => void;
  clearLogs: () => void;
  setOutputFiles: (f: string[]) => void;
  setStatusMessage: (m: string) => void;
  setBabeldoc: (installed: boolean, hint: string) => void;
  resetTask: () => void;
}

let logId = 0;

export const useTranslateStore = create<TranslateState>((set, get) => ({
  files: [],
  outputDir: "",
  langIn: "en",
  langOut: "zh",
  outputMode: "both",
  providerId: "",
  model: "",
  qps: 4,
  taskId: null,
  status: "idle",
  progress: 0,
  stage: "",
  logs: [],
  outputFiles: [],
  statusMessage: "",
  babeldocInstalled: null,
  babeldocHint: "",

  setFiles: (f) => set({ files: f.slice(0, 1) }),
  addFiles: (f) =>
    set({ files: f.slice(0, 1) }),
  removeFile: (path) =>
    set({ files: get().files.filter((x) => x.path !== path) }),
  setOutputDir: (d) => set({ outputDir: d }),
  setLangIn: (l) => set({ langIn: l }),
  setLangOut: (l) => set({ langOut: l }),
  setOutputMode: (m) => set({ outputMode: m }),
  setProviderId: (id) => set({ providerId: id }),
  setModel: (m) => set({ model: m }),
  setQps: (q) => set({ qps: q }),
  setTaskId: (id) => set({ taskId: id }),
  setStatus: (s) =>
    set((st) => ({
      status: s,
      files: st.files.map((f) => ({ ...f, status: s })),
    })),
  setProgress: (p, stage) =>
    set((st) => ({
      progress: p,
      stage: stage ?? st.stage,
    })),
  appendLog: (text, stream) =>
    set((st) => ({
      logs: [...st.logs, { id: logId++, text, stream }].slice(-500),
    })),
  clearLogs: () => set({ logs: [] }),
  setOutputFiles: (f) => set({ outputFiles: f }),
  setStatusMessage: (m) => set({ statusMessage: m }),
  setBabeldoc: (installed, hint) =>
    set({ babeldocInstalled: installed, babeldocHint: hint }),
  resetTask: () =>
    set({
      taskId: null,
      status: "idle",
      progress: 0,
      stage: "",
      logs: [],
      outputFiles: [],
      statusMessage: "",
      files: get().files.map((f) => ({ ...f, status: "idle" as TaskStatus })),
    }),
}));
