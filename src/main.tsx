import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { ConfigProvider, App as AntdApp } from "antd";
import zhCN from "antd/locale/zh_CN";
import enUS from "antd/locale/en_US";
import { useSyncExternalStore } from "react";
import App from "./App";
import { initI18n } from "./i18n";
import { useSettingsStore } from "./stores/settingsStore";

function useLang() {
  return useSyncExternalStore(
    (cb) => useSettingsStore.subscribe(cb),
    () => useSettingsStore.getState().language,
    () => "zh" as const,
  );
}

function Root() {
  const lang = useLang();
  const isDark =
    useSettingsStore.getState().theme === "dark" ||
    (useSettingsStore.getState().theme === "system" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);
  return (
    <ConfigProvider
      locale={lang === "en" ? enUS : zhCN}
      theme={{ algorithm: isDark ? darkAlgorithm : defaultAlgorithm }}
    >
      <AntdApp>
        <App />
      </AntdApp>
    </ConfigProvider>
  );
}

import { theme as antdTheme } from "antd";
const { darkAlgorithm, defaultAlgorithm } = antdTheme;

initI18n().finally(() => {
  createRoot(document.getElementById("root") as HTMLElement).render(
    <StrictMode>
      <BrowserRouter>
        <Root />
      </BrowserRouter>
    </StrictMode>,
  );
});
