import { useEffect } from "react";
import { Button, Layout, Menu, theme as antdTheme } from "antd";
import {
  CloseOutlined,
  FileTextOutlined,
  ApiOutlined,
  ControlOutlined,
  HistoryOutlined,
  MinusOutlined,
  SettingOutlined,
  BorderOutlined,
} from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { useLocation, useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettingsStore } from "./stores/settingsStore";
import { useProviderStore } from "./stores/providerStore";
import { useTranslateStore } from "./stores/translateStore";
import { useUpdateStore } from "./stores/updateStore";
import { translateApi } from "./services/api";
import type { TranslateEvent } from "./types";
import AppRoutes from "./app/routes";

const { Sider, Content } = Layout;

function AppTitleBar() {
  const windowControls = [
    {
      key: "minimize",
      label: "最小化",
      icon: <MinusOutlined />,
      onClick: () => void getCurrentWindow().minimize(),
    },
    {
      key: "maximize",
      label: "最大化",
      icon: <BorderOutlined />,
      onClick: () => void getCurrentWindow().toggleMaximize(),
    },
    {
      key: "close",
      label: "关闭",
      icon: <CloseOutlined />,
      onClick: () => void getCurrentWindow().close(),
    },
  ];

  return (
    <div className="app-titlebar">
      <div
        className="app-titlebar-drag-region"
        data-tauri-drag-region
        onDoubleClick={() => void getCurrentWindow().toggleMaximize()}
      />
      <div className="app-window-controls">
        {windowControls.map((control) => (
          <Button
            key={control.key}
            type="text"
            aria-label={control.label}
            title={control.label}
            icon={control.icon}
            className={
              control.key === "close"
                ? "app-window-control app-window-control-close"
                : "app-window-control"
            }
            onClick={control.onClick}
          />
        ))}
      </div>
    </div>
  );
}

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
            st.setProgress(ev.overall, ev.stage || undefined);
            break;
          case "status":
            st.setStatus(ev.status as never);
            if (ev.output_files) st.setOutputFiles(ev.output_files);
            if (ev.message) st.setStatusMessage(ev.message);
            if (ev.status === "success") st.setProgress(100);
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
    <div className="app-shell" style={{ background: token.colorBgLayout }}>
      <AppTitleBar />
      <Layout className="app-main-layout" style={{ background: "transparent" }}>
        <Sider
          width={200}
          className="floating-sider"
          style={{
            background: token.colorBgContainer,
          }}
        >
          <div
            style={{
              height: 60,
              display: "flex",
              alignItems: "center",
              padding: "0 16px",
              fontWeight: 800,
              fontSize: 18,
              color: token.colorTextHeading,
            }}
          >
            <img
              src="/logo.png"
              style={{
                height: 28,
                width: 28,
                marginRight: 10,
                borderRadius: 8,
                boxShadow: "0 4px 10px rgba(99, 102, 241, 0.2)"
              }}
              alt="logo"
            />
            <span className="brand-title" style={{ letterSpacing: "-0.5px", background: "linear-gradient(135deg, #6366f1 0%, #a855f7 100%)", WebkitBackgroundClip: "text", WebkitTextFillColor: "transparent" }}>
              {t("app.name")}
            </span>
          </div>
          <Menu
            mode="inline"
            selectedKeys={[selected]}
            items={items}
            style={{ borderInlineEnd: "none", background: "transparent" }}
            onClick={({ key }) => nav(key)}
          />
        </Sider>
        <Content
          className="page-fade-in"
          style={{
            overflow: "auto",
            padding: "12px 16px 12px 4px",
            background: "transparent",
          }}
        >
          <AppRoutes />
        </Content>
      </Layout>
    </div>
  );

}
