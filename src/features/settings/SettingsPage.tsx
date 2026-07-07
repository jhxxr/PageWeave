import { useEffect, useState } from "react";
import {
  Alert,
  Button,
  Card,
  Divider,
  Input,
  InputNumber,
  message,
  Progress,
  Select,
  Space,
  Switch,
  Typography,
} from "antd";
import {
  CloudDownloadOutlined,
  FolderOpenOutlined,
  InboxOutlined,
  ReloadOutlined,
  SyncOutlined,
} from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useSettingsStore } from "../../stores/settingsStore";
import { useProviderStore } from "../../stores/providerStore";
import { useTranslateStore } from "../../stores/translateStore";
import { useUpdateStore, type UpdateStatus } from "../../stores/updateStore";
import { LANGUAGES } from "../../shared/constants";
import { translateApi } from "../../services/api";
import type { AppSettings, OfflineAssetsInfo } from "../../types";

const { Text, Paragraph } = Typography;

export default function SettingsPage() {
  const { t } = useTranslation();
  const [messageApi, contextHolder] = message.useMessage();
  const [offlineInfo, setOfflineInfo] = useState<OfflineAssetsInfo | null>(null);
  const [installingOnline, setInstallingOnline] = useState(false);
  const [installingLocal, setInstallingLocal] = useState(false);
  const s = useSettingsStore();
  const updates = useUpdateStore();
  const providers = useProviderStore((s) => s.providers);

  // Ensure settings are loaded.
  useEffect(() => {
    if (!s.settings) void s.load();
  }, [s]);

  useEffect(() => {
    void refreshOfflineAssetsInfo();
  }, []);

  const cur: AppSettings = s.settings ?? {
    theme: "system",
    language: "zh",
    default_output_dir: "",
    default_lang_in: "en",
    default_lang_out: "zh",
    default_provider_id: "",
    log_retention_days: 7,
    developer_mode: false,
    cache_dir: "",
  };

  async function pickOutputDir() {
    const res = await openDialog({ directory: true });
    if (typeof res === "string") await s.patch({ default_output_dir: res });
  }

  async function refreshOfflineAssetsInfo() {
    try {
      const info = await translateApi.offlineAssetsInfo();
      setOfflineInfo(info);
      useTranslateStore.getState().setBabeldoc(info.installed, info.message);
    } catch (e) {
      messageApi.error((e as Error).message);
    }
  }

  async function installFromRelease() {
    setInstallingOnline(true);
    try {
      const res = await translateApi.installOfflineAssetsFromRelease();
      messageApi.success(res.message);
      await refreshOfflineAssetsInfo();
    } catch (e) {
      messageApi.error((e as Error).message);
    } finally {
      setInstallingOnline(false);
    }
  }

  async function installFromLocalFile() {
    const res = await openDialog({
      multiple: false,
      filters: [{ name: "BabelDOC offline assets", extensions: ["zip"] }],
    });
    if (typeof res !== "string") return;

    setInstallingLocal(true);
    try {
      const result = await translateApi.installOfflineAssetsFromFile(res);
      messageApi.success(result.message);
      await refreshOfflineAssetsInfo();
    } catch (e) {
      messageApi.error((e as Error).message);
    } finally {
      setInstallingLocal(false);
    }
  }

  async function checkForUpdates() {
    await updates.checkForUpdates("manual");
    const next = useUpdateStore.getState();
    if (next.status === "upToDate") {
      messageApi.success(t("settings.updatesUpToDate"));
    } else if (next.status === "readyToInstall") {
      messageApi.success(t("settings.updateReady"));
    } else if (next.status === "error" && next.error) {
      messageApi.error(next.error);
    }
  }

  async function installUpdate() {
    await updates.installAndRestart();
    const next = useUpdateStore.getState();
    if (next.status === "error" && next.error) {
      messageApi.error(next.error);
    }
  }

  const rowStyle: React.CSSProperties = {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    paddingBottom: 12,
    borderBottom: "1px dashed rgba(100, 116, 139, 0.08)",
    gap: 16,
    flexWrap: "wrap",
  };

  return (
    <>
      {contextHolder}
      <Card title={t("settings.title")} className="glass-card">
        <Space direction="vertical" size="large" style={{ width: "100%", maxWidth: 680 }}>
          
          <div style={rowStyle}>
            <Text style={{ fontWeight: 600 }}>{t("settings.theme")}</Text>
            <Select
              style={{ width: 240 }}
              value={cur.theme as "light" | "dark" | "system"}
              onChange={(v: "light" | "dark" | "system") => s.applyTheme(v)}
              options={[
                { value: "light", label: t("settings.themeLight") },
                { value: "dark", label: t("settings.themeDark") },
                { value: "system", label: t("settings.themeSystem") },
              ]}
            />
          </div>

          <div style={rowStyle}>
            <Text style={{ fontWeight: 600 }}>{t("settings.language")}</Text>
            <Select
              style={{ width: 240 }}
              value={cur.language as "zh" | "en"}
              onChange={(v: "zh" | "en") => s.setLanguage(v)}
              options={[
                { value: "zh", label: "中文" },
                { value: "en", label: "English" },
              ]}
            />
          </div>

          <div style={rowStyle}>
            <Text style={{ fontWeight: 600 }}>{t("settings.defaultOutputDir")}</Text>
            <Space.Compact style={{ width: 380 }}>
              <Input
                style={{ width: "100%" }}
                value={cur.default_output_dir}
                onChange={(e) => s.patch({ default_output_dir: e.target.value })}
                placeholder={t("translate.selectOutputDir")}
              />
              <Button icon={<FolderOpenOutlined />} onClick={pickOutputDir} />
            </Space.Compact>
          </div>

          <div style={rowStyle}>
            <Text style={{ fontWeight: 600 }}>{t("settings.defaultLangIn")}</Text>
            <Select
              style={{ width: 240 }}
              value={cur.default_lang_in}
              onChange={(v) => s.patch({ default_lang_in: v })}
              options={LANGUAGES.map((l) => ({ value: l.code, label: l.label }))}
            />
          </div>

          <div style={rowStyle}>
            <Text style={{ fontWeight: 600 }}>{t("settings.defaultLangOut")}</Text>
            <Select
              style={{ width: 240 }}
              value={cur.default_lang_out}
              onChange={(v) => s.patch({ default_lang_out: v })}
              options={LANGUAGES.map((l) => ({ value: l.code, label: l.label }))}
            />
          </div>

          <div style={rowStyle}>
            <Text style={{ fontWeight: 600 }}>{t("settings.defaultProvider")}</Text>
            <Select
              style={{ width: 240 }}
              value={cur.default_provider_id || undefined}
              onChange={(v) => s.patch({ default_provider_id: v })}
              options={providers.map((p) => ({ value: p.id, label: p.name }))}
              allowClear
            />
          </div>

          <div style={rowStyle}>
            <Text style={{ fontWeight: 600 }}>{t("settings.logRetention")}</Text>
            <InputNumber
              min={1}
              max={365}
              style={{ width: 120 }}
              value={cur.log_retention_days}
              onChange={(v) => s.patch({ log_retention_days: Number(v) || 7 })}
            />
          </div>

          <div style={rowStyle}>
            <span>
              <Text style={{ fontWeight: 600, display: "block" }}>
                {t("settings.developerMode")}
              </Text>
              <Text type="secondary" style={{ fontSize: 12 }}>
                {t("settings.developerModeHelp")}
              </Text>
            </span>
            <Switch
              checked={cur.developer_mode}
              onChange={(checked) => s.patch({ developer_mode: checked })}
            />
          </div>

          <div style={{ marginTop: 8 }}>
            <Text strong style={{ fontSize: 16, display: "block", marginBottom: 12 }}>
              {t("settings.offlineAssets")}
            </Text>
            <Alert
              style={{ marginBottom: 16, borderRadius: 10 }}
              type={offlineInfo?.installed ? "success" : "info"}
              showIcon
              message={
                offlineInfo?.installed
                  ? t("settings.offlineAssetsReady")
                  : t("settings.offlineAssetsMissing")
              }
              description={
                offlineInfo
                  ? `${t("settings.cacheDir")}: ${offlineInfo.cache_dir} · ${formatBytes(
                      offlineInfo.size_bytes,
                    )}`
                  : t("settings.loading")
              }
            />
            <Space wrap size="middle">
              <Button
                type="primary"
                icon={<CloudDownloadOutlined />}
                loading={installingOnline}
                onClick={installFromRelease}
                style={{ fontWeight: 550 }}
              >
                {t("settings.installFromRelease")}
              </Button>
              <Button
                icon={<InboxOutlined />}
                loading={installingLocal}
                onClick={installFromLocalFile}
              >
                {t("settings.installFromLocal")}
              </Button>
              <Button onClick={refreshOfflineAssetsInfo}>{t("settings.refresh")}</Button>
            </Space>
          </div>

          <Divider style={{ marginBlock: 12 }} />

          <div>
            <Text strong style={{ fontSize: 16, display: "block", marginBottom: 12 }}>
              {t("settings.about")}
            </Text>
            <Paragraph>
              <div style={{ marginBottom: 8, fontWeight: 500 }}>
                {t("settings.version")}: {updates.appVersion || t("settings.loading")}
              </div>
              <Alert
                style={{ marginBottom: 16, borderRadius: 10 }}
                type={updateAlertType(updates.status)}
                showIcon
                message={updateStatusText(t, updates.status, updates.updateVersion)}
                description={
                  updates.error ||
                  (updates.lastCheckedAt
                    ? `${t("settings.lastChecked")}: ${new Date(
                        updates.lastCheckedAt,
                      ).toLocaleString()}`
                    : undefined)
                }
              />
              {updates.status === "downloading" && (
                <Progress
                  percent={downloadPercent(updates.downloadedBytes, updates.contentLength)}
                  size="small"
                  style={{ marginBottom: 16 }}
                />
              )}
              <Space wrap style={{ marginBottom: 16 }}>
                <Button
                  icon={<SyncOutlined />}
                  loading={updates.status === "checking" || updates.status === "downloading"}
                  onClick={checkForUpdates}
                >
                  {t("settings.checkUpdates")}
                </Button>
                {(updates.status === "readyToInstall" || updates.status === "installing") && (
                  <Button
                    type="primary"
                    icon={<ReloadOutlined />}
                    loading={updates.status === "installing"}
                    onClick={installUpdate}
                    style={{ fontWeight: 550 }}
                  >
                    {t("settings.installAndRestart")}
                  </Button>
                )}
              </Space>
              <div style={{ marginTop: 8 }}>
                {t("settings.license")}: {t("settings.licenseNote")}
              </div>
              <div style={{ marginTop: 12 }}>
                <Text type="secondary" style={{ fontSize: 12 }}>{t("settings.deps")}</Text>
              </div>
              <div style={{ fontSize: 12, opacity: 0.85 }}>{t("settings.depsBody")}</div>
            </Paragraph>
          </div>
        </Space>
      </Card>
    </>
  );

}

function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 MB";
  const mb = bytes / 1024 / 1024;
  if (mb < 1024) return `${mb.toFixed(1)} MB`;
  return `${(mb / 1024).toFixed(2)} GB`;
}

function downloadPercent(downloaded: number, total?: number) {
  if (!total || total <= 0) return 0;
  return Math.min(100, Math.round((downloaded / total) * 100));
}

function updateAlertType(status: UpdateStatus) {
  if (status === "readyToInstall") return "success";
  if (status === "error") return "error";
  if (status === "upToDate") return "success";
  if (status === "checking" || status === "downloading" || status === "available") {
    return "info";
  }
  return "info";
}

function updateStatusText(
  t: (key: string, options?: Record<string, string>) => string,
  status: UpdateStatus,
  version?: string,
) {
  switch (status) {
    case "checking":
      return t("settings.updatesChecking");
    case "available":
      return t("settings.updateAvailable", { version: version ?? "" });
    case "downloading":
      return t("settings.updateDownloading", { version: version ?? "" });
    case "readyToInstall":
      return t("settings.updateReady", { version: version ?? "" });
    case "installing":
      return t("settings.updateInstalling");
    case "upToDate":
      return t("settings.updatesUpToDate");
    case "error":
      return t("settings.updateFailed");
    case "idle":
      return t("settings.updatesIdle");
  }
}
