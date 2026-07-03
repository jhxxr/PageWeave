import { useEffect, useState } from "react";
import {
  Alert,
  Button,
  Card,
  Divider,
  Input,
  InputNumber,
  message,
  Select,
  Space,
  Typography,
} from "antd";
import { CloudDownloadOutlined, FolderOpenOutlined, InboxOutlined } from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useSettingsStore } from "../../stores/settingsStore";
import { useProviderStore } from "../../stores/providerStore";
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
    cache_dir: "",
  };

  async function pickOutputDir() {
    const res = await openDialog({ directory: true });
    if (typeof res === "string") await s.patch({ default_output_dir: res });
  }

  async function refreshOfflineAssetsInfo() {
    try {
      setOfflineInfo(await translateApi.offlineAssetsInfo());
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

  return (
    <>
      {contextHolder}
      <Card title={t("settings.title")} variant="borderless">
      <Space direction="vertical" size="middle" style={{ width: 520 }}>
        <div>
          <Text type="secondary">{t("settings.theme")}</Text>
          <Select
            style={{ width: 200, marginLeft: 12 }}
            value={cur.theme as "light" | "dark" | "system"}
            onChange={(v: "light" | "dark" | "system") => s.applyTheme(v)}
            options={[
              { value: "light", label: t("settings.themeLight") },
              { value: "dark", label: t("settings.themeDark") },
              { value: "system", label: t("settings.themeSystem") },
            ]}
          />
        </div>
        <div>
          <Text type="secondary">{t("settings.language")}</Text>
          <Select
            style={{ width: 200, marginLeft: 12 }}
            value={cur.language as "zh" | "en"}
            onChange={(v: "zh" | "en") => s.setLanguage(v)}
            options={[
              { value: "zh", label: "中文" },
              { value: "en", label: "English" },
            ]}
          />
        </div>
        <div>
          <Text type="secondary">{t("settings.defaultOutputDir")}</Text>
          <Input
            style={{ width: 360, marginLeft: 12 }}
            value={cur.default_output_dir}
            onChange={(e) => s.patch({ default_output_dir: e.target.value })}
            placeholder={t("translate.selectOutputDir")}
          />
          <Button icon={<FolderOpenOutlined />} onClick={pickOutputDir} style={{ marginLeft: 8 }} />
        </div>
        <div>
          <Text type="secondary">{t("settings.defaultLangIn")}</Text>
          <Select
            style={{ width: 200, marginLeft: 12 }}
            value={cur.default_lang_in}
            onChange={(v) => s.patch({ default_lang_in: v })}
            options={LANGUAGES.map((l) => ({ value: l.code, label: l.label }))}
          />
        </div>
        <div>
          <Text type="secondary">{t("settings.defaultLangOut")}</Text>
          <Select
            style={{ width: 200, marginLeft: 12 }}
            value={cur.default_lang_out}
            onChange={(v) => s.patch({ default_lang_out: v })}
            options={LANGUAGES.map((l) => ({ value: l.code, label: l.label }))}
          />
        </div>
        <div>
          <Text type="secondary">{t("settings.defaultProvider")}</Text>
          <Select
            style={{ width: 320, marginLeft: 12 }}
            value={cur.default_provider_id || undefined}
            onChange={(v) => s.patch({ default_provider_id: v })}
            options={providers.map((p) => ({ value: p.id, label: p.name }))}
            allowClear
          />
        </div>
        <div>
          <Text type="secondary">{t("settings.logRetention")}</Text>
          <InputNumber
            min={1}
            max={365}
            style={{ width: 120, marginLeft: 12 }}
            value={cur.log_retention_days}
            onChange={(v) => s.patch({ log_retention_days: Number(v) || 7 })}
          />
        </div>

        <Divider />
        <div>
          <Text strong>{t("settings.offlineAssets")}</Text>
          <Alert
            style={{ marginTop: 8, marginBottom: 12 }}
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
          <Space wrap>
            <Button
              icon={<CloudDownloadOutlined />}
              loading={installingOnline}
              onClick={installFromRelease}
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

        <Divider />
        <div>
          <Text strong>{t("settings.about")}</Text>
          <Paragraph style={{ marginTop: 8 }}>
            <div>
              {t("settings.version")}: 0.1.0
            </div>
            <div>
              {t("settings.license")}: {t("settings.licenseNote")}
            </div>
            <div style={{ marginTop: 8 }}>
              <Text type="secondary">{t("settings.deps")}</Text>
            </div>
            <div>{t("settings.depsBody")}</div>
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
