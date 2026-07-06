import { useEffect, useMemo, useRef, useState } from "react";
import {
  Alert,
  AutoComplete,
  Button,
  Card,
  Empty,
  Input,
  Radio,
  Select,
  Space,
  Table,
  Tag,
  Tooltip,
  Typography,
  message,
} from "antd";
import {
  FolderOpenOutlined,
  InboxOutlined,
  PlayCircleOutlined,
  StopOutlined,
  CopyOutlined,
  FolderOutlined,
  FilePdfOutlined,
} from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import { useNavigate } from "react-router-dom";
import { useTranslateStore } from "../../stores/translateStore";
import { useProviderStore } from "../../stores/providerStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { translateApi } from "../../services/api";
import { LANGUAGES, formatBytes } from "../../shared/constants";
import type { FileItem } from "../../stores/translateStore";
import type { TranslateRequest } from "../../types";
import {
  LogStream,
  ProgressOverview,
  cap,
  statusColor,
} from "./ProgressLogPanel";

const { Text, Paragraph } = Typography;

export default function TranslatePage() {
  const { t } = useTranslation();
  const nav = useNavigate();
  const st = useTranslateStore();
  const providers = useProviderStore((s) => s.providers);
  const defaultId = useProviderStore((s) => s.defaultId);
  const settings = useSettingsStore((s) => s.settings);
  const [busy, setBusy] = useState(false);
  const logRef = useRef<HTMLDivElement>(null);

  // Apply default provider / languages from settings on first load.
  useEffect(() => {
    if (!st.providerId) {
      const def =
        providers.find((p) => p.id === defaultId) ??
        providers.find((p) => p.is_enabled);
      if (def) {
        st.setProviderId(def.id);
        st.setModel(def.default_model);
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [providers, defaultId]);

  useEffect(() => {
    if (settings && !st.outputDir) st.setOutputDir(settings.default_output_dir);
    if (settings && (!st.langIn || st.langIn === "en"))
      st.setLangIn(settings.default_lang_in || "en");
    if (settings && (!st.langOut || st.langOut === "zh"))
      st.setLangOut(settings.default_lang_out || "zh");
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [settings]);

  // Auto-scroll log panel.
  useEffect(() => {
    if (logRef.current) logRef.current.scrollTop = logRef.current.scrollHeight;
  }, [st.logs]);

  // Drag & drop via Tauri native event.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      const app = getCurrentWebview();
      unlisten = await app.onDragDropEvent(async (e) => {
        if (e.payload.type !== "drop") return;
        const paths = e.payload.paths.filter((p) =>
          p.toLowerCase().endsWith(".pdf"),
        );
        if (!paths.length) return;
        if (paths.length > 1) {
          message.info(t("translate.onePdfOnly"));
        }
        const items: FileItem[] = await Promise.all(
          paths.slice(0, 1).map(createFileItem),
        );
        st.addFiles(items);
      });
    })();
    return () => void unlisten?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function pickFiles() {
    const res = await openDialog({
      multiple: false,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    const paths = Array.isArray(res) ? res : res ? [res] : [];
    if (!paths.length) return;
    const items = await Promise.all(paths.map(createFileItem));
    st.addFiles(items);
  }

  async function pickOutputDir() {
    const res = await openDialog({ directory: true });
    if (typeof res === "string") st.setOutputDir(res);
  }

  const selectedProvider = useMemo(
    () => providers.find((p) => p.id === st.providerId),
    [providers, st.providerId],
  );

  const canStart =
    st.files.length > 0 &&
    !!st.providerId &&
    !!(st.model || selectedProvider?.default_model) &&
    !!st.outputDir.trim() &&
    st.babeldocInstalled !== false &&
    st.status !== "running";

  async function start() {
    const first = st.files[0];
    if (!first) {
      message.warning(t("translate.noPdf"));
      return;
    }
    if (!selectedProvider) {
      message.warning(t("translate.noProvider"));
      return;
    }
    if (!selectedProvider.has_api_key) {
      message.warning(t("translate.noApiKey"));
      return;
    }
    const model = st.model || selectedProvider.default_model;
    if (!model) {
      message.warning(t("translate.noModel"));
      return;
    }
    if (!st.outputDir.trim()) {
      message.warning(t("translate.noOutputDir"));
      return;
    }
    if (st.babeldocInstalled === false) {
      message.error(st.babeldocHint || t("settings.offlineAssetsMissing"));
      return;
    }
    setBusy(true);
    st.clearLogs();
    st.setOutputFiles([]);
    st.setStatusMessage("");
    st.setProgress(0, "");
    st.setTaskId(null);
    st.setStatus("running");
    const req: TranslateRequest = {
      pdf_paths: [first.path],
      output_dir: st.outputDir,
      lang_in: st.langIn,
      lang_out: st.langOut,
      output_mode: st.outputMode,
      provider: {
        base_url: selectedProvider.base_url,
        api_key_id: selectedProvider.api_key_id,
        model,
      },
      qps: st.qps,
    };
    try {
      const taskId = await translateApi.start(req);
      st.setTaskId(taskId);
    } catch (e) {
      st.setStatus("error");
      st.setStatusMessage(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function cancel() {
    if (st.taskId) await translateApi.cancel(st.taskId);
  }

  return (
    <Space direction="vertical" style={{ width: "100%" }} size={16}>
      {st.babeldocInstalled === false && (
        <Alert
          type="warning"
          showIcon
          message={st.babeldocHint || t("settings.offlineAssetsMissing")}
          action={
            <Button
              size="small"
              onClick={() => nav("/settings")}
            >
              {t("app.nav.settings")}
            </Button>
          }
        />
      )}

      <Card title={t("translate.title")} variant="borderless">
        <div
          onDragOver={(e) => e.preventDefault()}
          onClick={pickFiles}
          style={{
            border: `2px dashed ${
              st.files.length ? "#1677ff" : "#d9d9d9"
            }`,
            borderRadius: 8,
            padding: 24,
            textAlign: "center",
            cursor: "pointer",
            marginBottom: 16,
          }}
        >
          <InboxOutlined style={{ fontSize: 32, color: "#1677ff" }} />
          <div style={{ marginTop: 8 }}>
            <Text>{t("translate.dropHere")}</Text>
          </div>
          <div style={{ marginTop: 4 }}>
            <Text type="secondary">{t("translate.onePdfHint")}</Text>
          </div>
        </div>

        <Table
          size="small"
          dataSource={st.files}
          rowKey="path"
          pagination={false}
          locale={{
            emptyText: <Empty description={t("translate.dropHere")} />,
          }}
          columns={[
            {
              title: t("translate.fileCol"),
              dataIndex: "name",
              render: (v: string) => (
                <Space>
                  <FilePdfOutlined />
                  <Text style={{ wordBreak: "break-all" }}>{v}</Text>
                </Space>
              ),
            },
            {
              title: t("translate.sizeCol"),
              dataIndex: "size",
              width: 100,
              render: (v: number | null) => (v == null ? "-" : formatBytes(v)),
            },
            {
              title: t("translate.statusCol"),
              dataIndex: "status",
              width: 100,
              render: (v: string) => (
                <Tag color={statusColor[v]}>
                  {t(`translate.status${cap(v)}`)}
                </Tag>
              ),
            },
            {
              title: t("translate.actionCol"),
              width: 80,
              render: (_: unknown, row: FileItem) => (
                <Button
                  size="small"
                  type="text"
                  danger
                  onClick={() => st.removeFile(row.path)}
                >
                  {t("translate.remove")}
                </Button>
              ),
            },
          ]}
        />
      </Card>

      <Card variant="borderless">
        <Space wrap size="middle">
          <span>
            <Text type="secondary">{t("translate.langIn")}</Text>
            <Select
              style={{ width: 140, marginLeft: 8 }}
              value={st.langIn}
              onChange={st.setLangIn}
              options={LANGUAGES.map((l) => ({ value: l.code, label: l.label }))}
            />
          </span>
            <Text type="secondary">→</Text>
          <span>
            <Text type="secondary">{t("translate.langOut")}</Text>
            <Select
              style={{ width: 140, marginLeft: 8 }}
              value={st.langOut}
              onChange={st.setLangOut}
              options={LANGUAGES.map((l) => ({ value: l.code, label: l.label }))}
            />
          </span>
          <span>
            <Text type="secondary">{t("translate.provider")}</Text>
            <Select
              style={{ width: 200, marginLeft: 8 }}
              value={st.providerId || undefined}
              placeholder={t("translate.noProvider")}
              onChange={(id) => {
                st.setProviderId(id);
                const p = providers.find((x) => x.id === id);
                if (p) st.setModel(p.default_model);
              }}
              options={providers.map((p) => ({
                value: p.id,
                label: p.name,
              }))}
            />
          </span>
          <span>
            <Text type="secondary">{t("translate.model")}</Text>
            <AutoComplete
              style={{ width: 220, marginLeft: 8 }}
              value={st.model}
              onChange={st.setModel}
              options={(selectedProvider?.models ?? []).map((m) => ({
                value: m,
                label: m,
              }))}
            />
          </span>
          <span>
            <Text type="secondary">{t("translate.outputMode")}</Text>
            <Radio.Group
              style={{ marginLeft: 8 }}
              value={st.outputMode}
              onChange={(e) => st.setOutputMode(e.target.value)}
            >
              <Radio.Button value="mono">{t("translate.modeMono")}</Radio.Button>
              <Radio.Button value="dual">{t("translate.modeDual")}</Radio.Button>
              <Radio.Button value="both">{t("translate.modeBoth")}</Radio.Button>
            </Radio.Group>
          </span>
          <span>
            <Text type="secondary">{t("translate.qps")}</Text>
            <Input
              type="number"
              min={1}
              max={32}
              style={{ width: 80, marginLeft: 8 }}
              value={st.qps}
              onChange={(e) => st.setQps(Number(e.target.value) || 1)}
            />
          </span>
        </Space>
      </Card>

      <Card variant="borderless">
        <Space style={{ width: "100%" }} direction="vertical">
          <Space>
            <Text type="secondary">{t("translate.outputDir")}</Text>
            <Input
              style={{ width: 480 }}
              value={st.outputDir}
              onChange={(e) => st.setOutputDir(e.target.value)}
              placeholder={t("translate.selectOutputDir")}
            />
            <Button icon={<FolderOpenOutlined />} onClick={pickOutputDir}>
              {t("translate.selectOutputDir")}
            </Button>
          </Space>
          <Space>
            <Button
              type="primary"
              icon={<PlayCircleOutlined />}
              disabled={!canStart || busy}
              loading={busy}
              onClick={start}
            >
              {t("translate.start")}
            </Button>
            <Button
              danger
              icon={<StopOutlined />}
              disabled={st.status !== "running"}
              onClick={cancel}
            >
              {t("translate.cancel")}
            </Button>
          </Space>
        </Space>
      </Card>

      <Card
        title={t("translate.progress")}
        variant="borderless"
        extra={
          st.status !== "idle" && (
            <Tag color={statusColor[st.status]}>
              {t(`translate.status${cap(st.status)}`)}
            </Tag>
          )
        }
      >
        <ProgressOverview
          percent={st.progress}
          status={st.status}
          stage={st.stage}
          latestLog={st.logs[st.logs.length - 1]?.text}
          stageLabel={t("translate.currentStage")}
          latestLabel={t("tasks.latestLog")}
        />
        {st.statusMessage && (
          <Paragraph type={st.status === "error" ? "danger" : undefined}>
            {st.statusMessage}
          </Paragraph>
        )}
        {st.outputFiles.length > 0 && (
          <Space>
            {st.outputFiles.map((f) => (
              <Space key={f}>
                <Button
                  size="small"
                  icon={<FilePdfOutlined />}
                  onClick={() => openPath(f)}
                >
                  {t("translate.openFile")}
                </Button>
                <Tooltip title={f}>
                  <Button
                    size="small"
                    icon={<FolderOutlined />}
                    onClick={() => revealItemInDir(f)}
                  >
                    {t("translate.openFolder")}
                  </Button>
                </Tooltip>
              </Space>
            ))}
          </Space>
        )}
      </Card>

      <Card
        title={
          <Space>
            <span>{t("translate.log")}</span>
            <Tooltip title={t("common.copy")}>
              <Button
                size="small"
                icon={<CopyOutlined />}
                onClick={() => {
                  const txt = st.logs.map((l) => l.text).join("\n");
                  navigator.clipboard.writeText(txt).then(() =>
                    message.success(t("common.copied")),
                  );
                }}
              />
            </Tooltip>
          </Space>
        }
        variant="borderless"
      >
        <LogStream
          logs={st.logs}
          emptyText={t("translate.logEmpty")}
          containerRef={logRef}
        />
      </Card>
    </Space>
  );
}

async function createFileItem(path: string): Promise<FileItem> {
  const name = path.split(/[\\/]/).pop() ?? path;
  try {
    const size = await translateApi.fileSize(path);
    return { path, name, size, status: "idle" };
  } catch {
    return { path, name, size: null, status: "idle" };
  }
}
