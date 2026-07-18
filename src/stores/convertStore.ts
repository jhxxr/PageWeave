import { create } from "zustand";

export type ConvertTaskStatus =
  | "idle"
  | "running"
  | "success"
  | "error"
  | "cancelled";

export interface ConvertLogLine {
  id: number;
  text: string;
  stream: string;
}

interface ConvertState {
  inputPath: string;
  inputName: string;
  outputDir: string;
  taskId: string | null;
  status: ConvertTaskStatus;
  logs: ConvertLogLine[];
  outputFile: string;
  statusMessage: string;
  markitdownInstalled: boolean | null;
  markitdownHint: string;

  setInput: (path: string, name: string) => void;
  clearInput: () => void;
  setOutputDir: (d: string) => void;
  setTaskId: (id: string | null) => void;
  setStatus: (s: ConvertTaskStatus) => void;
  appendLog: (text: string, stream: string) => void;
  clearLogs: () => void;
  setOutputFile: (f: string) => void;
  setStatusMessage: (m: string) => void;
  setMarkitdown: (installed: boolean, hint: string) => void;
  resetTask: () => void;
}

let logId = 0;

export const useConvertStore = create<ConvertState>((set, get) => ({
  inputPath: "",
  inputName: "",
  outputDir: "",
  taskId: null,
  status: "idle",
  logs: [],
  outputFile: "",
  statusMessage: "",
  markitdownInstalled: null,
  markitdownHint: "",

  setInput: (path, name) => set({ inputPath: path, inputName: name }),
  clearInput: () => set({ inputPath: "", inputName: "" }),
  setOutputDir: (d) => set({ outputDir: d }),
  setTaskId: (id) => set({ taskId: id }),
  setStatus: (s) => set({ status: s }),
  appendLog: (text, stream) =>
    set((st) => ({
      logs: [...st.logs, { id: logId++, text, stream }].slice(-500),
    })),
  clearLogs: () => set({ logs: [] }),
  setOutputFile: (f) => set({ outputFile: f }),
  setStatusMessage: (m) => set({ statusMessage: m }),
  setMarkitdown: (installed, hint) =>
    set({ markitdownInstalled: installed, markitdownHint: hint }),
  resetTask: () =>
    set({
      taskId: null,
      status: "idle",
      logs: [],
      outputFile: "",
      statusMessage: "",
      // keep inputPath / outputDir
      inputPath: get().inputPath,
      inputName: get().inputName,
    }),
}));
