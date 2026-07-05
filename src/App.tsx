import { useEffect } from "react";
import { Layout, Menu, theme as antdTheme } from "antd";
import {
  FileTextOutlined,
  ApiOutlined,
  ControlOutlined,
  HistoryOutlined,
  SettingOutlined,
} from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { useLocation, useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useSettingsStore } from "./stores/settingsStore";
import { useProviderStore } from "./stores/providerStore";
import { useTranslateStore } from "./stores/translateStore";
import { useUpdateStore } from "./stores/updateStore";
import { translateApi } from "./services/api";
import type { TranslateEvent } from "./types";
import AppRoutes from "./app/routes";

const { Sider, Content } = Layout;

export default function App() {
  const { t } = useTranslation();
  const nav = useNavigate();
  const loc = useLocation();
  const loadSettings = useSettingsStore((s) => s.load);
  const loadProviders = useProviderStore((s) => s.load);
  const initUpdates = useUpdateStore((s) => s.init);
  const { token } = antdTheme.useToken();

  // Bootstrap settings + providers, and subscribe to translate events once.
  useEffect(() => {
    void loadSettings();
    void loadProviders();
    void initUpdates();
  }, [loadSettings, loadProviders, initUpdates]);

  useEffect(() => {
    // Translation requires the offline BabelDOC resource package.
    translateApi
      .offlineAssetsInfo()
      .then((info) => {
        useTranslateStore.getState().setBabeldoc(info.installed, info.message);
      })
      .catch(() => {
        useTranslateStore
          .getState()
          .setBabeldoc(false, "未检测到 BabelDOC 离线资源包，请在设置页安装。");
      });

    let unlisten: UnlistenFn | undefined;
    (async () => {
      unlisten = await listen<TranslateEvent>("translate://progress", (e) => {
        const ev = e.payload;
        const st = useTranslateStore.getState();
        if (ev.task_id && st.taskId && ev.task_id !== st.taskId) return;
        switch (ev.type) {
          case "log":
            st.appendLog(ev.line, ev.stream);
            break;
          case "progress":
            st.setProgress(ev.overall, ev.stage);
            break;
          case "status":
            st.setStatus(ev.status as never);
            if (ev.output_files) st.setOutputFiles(ev.output_files);
            if (ev.message) st.setStatusMessage(ev.message);
            break;
        }
      });
    })();
    return () => {
      void unlisten?.();
    };
  }, []);

  const items = [
    { key: "/translate", icon: <FileTextOutlined />, label: t("app.nav.translate") },
    { key: "/provider", icon: <ApiOutlined />, label: t("app.nav.provider") },
    { key: "/params", icon: <ControlOutlined />, label: t("app.nav.params") },
    { key: "/tasks", icon: <HistoryOutlined />, label: t("app.nav.tasks") },
    { key: "/settings", icon: <SettingOutlined />, label: t("app.nav.settings") },
  ];

  const selected =
    items.find((i) => loc.pathname.startsWith(i.key))?.key ?? "/translate";

  return (
    <Layout style={{ height: "100vh" }}>
      <Sider
        width={184}
        style={{
          background: token.colorBgContainer,
          borderRight: `1px solid ${token.colorBorderSecondary}`,
        }}
      >
        <div
          style={{
            height: 48,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontWeight: 700,
            fontSize: 18,
            color: token.colorPrimary,
          }}
        >
          {t("app.name")}
        </div>
        <Menu
          mode="inline"
          selectedKeys={[selected]}
          items={items}
          onClick={({ key }) => nav(key)}
        />
      </Sider>
      <Content
        style={{ overflow: "auto", padding: 20, background: token.colorBgLayout }}
      >
        <AppRoutes />
      </Content>
    </Layout>
  );
}
