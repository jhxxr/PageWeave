import { create } from "zustand";
import i18n from "../i18n";
import type { AppSettings } from "../types";
import { settingsApi } from "../services/api";

type Theme = "light" | "dark" | "system";

interface SettingsState {
  theme: Theme;
  language: "zh" | "en";
  settings: AppSettings | null;
  load: () => Promise<void>;
  applyTheme: (t: Theme) => void;
  setLanguage: (l: "zh" | "en") => Promise<void>;
  patch: (p: Partial<AppSettings>) => Promise<void>;
}

function applyHtmlTheme(t: Theme) {
  const sys = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const dark = t === "dark" || (t === "system" && sys);
  document.documentElement.setAttribute(
    "data-theme",
    dark ? "dark" : "light",
  );
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  theme: "system",
  language: "zh",
  settings: null,
  async load() {
    const s = await settingsApi.get();
    set({ settings: s, theme: s.theme as Theme, language: s.language as "zh" | "en" });
    applyHtmlTheme(s.theme as Theme);
    await i18n.changeLanguage(s.language);
  },
  applyTheme(t) {
    applyHtmlTheme(t);
    set({ theme: t });
  },
  async setLanguage(l) {
    await i18n.changeLanguage(l);
    set({ language: l });
    await get().patch({ language: l });
  },
  async patch(p) {
    const cur = get().settings ?? (await settingsApi.get());
    const next: AppSettings = { ...cur, ...p };
    const saved = await settingsApi.save(next);
    set({ settings: saved });
    if (p.theme) {
      applyHtmlTheme(p.theme as Theme);
      set({ theme: p.theme as Theme });
    }
    if (p.language) set({ language: p.language as "zh" | "en" });
  },
}));
