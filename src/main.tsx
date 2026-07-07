import { StrictMode, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { ConfigProvider, App as AntdApp } from "antd";
import zhCN from "antd/locale/zh_CN";
import enUS from "antd/locale/en_US";
import { useSyncExternalStore } from "react";
import App from "./App";
import { initI18n } from "./i18n";
import { useSettingsStore } from "./stores/settingsStore";
import "./index.css";

function useLang() {
  return useSyncExternalStore(
    (cb) => useSettingsStore.subscribe(cb),
    () => useSettingsStore.getState().language,
    () => "zh" as const,
  );
}

function useThemeDark() {
  return useSyncExternalStore(
    (cb) => useSettingsStore.subscribe(cb),
    () => {
      const t = useSettingsStore.getState().theme;
      if (t === "dark") return true;
      if (t === "light") return false;
      return window.matchMedia("(prefers-color-scheme: dark)").matches;
    },
    () => false,
  );
}

function Root() {
  const lang = useLang();
  const isDark = useThemeDark();

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add("dark-theme");
    } else {
      document.documentElement.classList.remove("dark-theme");
    }
  }, [isDark]);

  return (
    <ConfigProvider
      locale={lang === "en" ? enUS : zhCN}
      theme={{
        algorithm: isDark ? darkAlgorithm : defaultAlgorithm,
        token: {
          colorPrimary: isDark ? "#818cf8" : "#6366f1",
          colorInfo: isDark ? "#22d3ee" : "#06b6d4",
          borderRadius: 12,
          fontFamily: "'Inter', 'Outfit', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
          colorBgLayout: isDark ? "#0f172a" : "#f8fafc",
          colorBgContainer: isDark ? "#1e293b" : "#ffffff",
          colorBorderSecondary: isDark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.04)",
        },
        components: {
          Card: {
            colorBgContainer: isDark ? "rgba(30,41,59,0.4)" : "rgba(255,255,255,0.7)",
            borderRadiusLG: 16,
          },
          Button: {
            borderRadius: 10,
            controlHeight: 36,
          },
          Input: {
            borderRadius: 10,
            controlHeight: 36,
          },
          Select: {
            borderRadius: 10,
            controlHeight: 36,
          },
        }
      }}
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

