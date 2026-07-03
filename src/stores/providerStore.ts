import { create } from "zustand";
import type { ProviderRecord } from "../types";
import { providerApi } from "../services/api";

interface ProviderState {
  providers: ProviderRecord[];
  defaultId: string;
  loading: boolean;
  load: () => Promise<void>;
  upsertLocal: (rec: ProviderRecord) => void;
  removeLocal: (id: string) => void;
  setDefaultLocal: (id: string) => void;
}

export const useProviderStore = create<ProviderState>((set, get) => ({
  providers: [],
  defaultId: "",
  loading: false,
  async load() {
    set({ loading: true });
    const list = await providerApi.list();
    const def = list.find((p) => p.is_applied);
    set({ providers: list, defaultId: def?.id ?? "", loading: false });
  },
  upsertLocal(rec) {
    const list = get().providers.filter((p) => p.id !== rec.id);
    list.push(rec);
    list.sort((a, b) => a.sort_index - b.sort_index);
    set({
      providers: list,
      defaultId: rec.is_applied ? rec.id : get().defaultId,
    });
  },
  removeLocal(id) {
    const list = get().providers.filter((p) => p.id !== id);
    set({
      providers: list,
      defaultId: get().defaultId === id ? "" : get().defaultId,
    });
  },
  setDefaultLocal(id) {
    set({
      defaultId: id,
      providers: get().providers.map((p) => ({
        ...p,
        is_applied: p.id === id,
      })),
    });
  },
}));
