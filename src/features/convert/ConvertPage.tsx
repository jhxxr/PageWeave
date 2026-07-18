import { useEffect, useRef, useState } from "react";
import {
  Alert,
  Button,
  Card,
  Input,
  Progress,
  Space,
  Tag,
  Tooltip,
  Typography,
  message,
} from "antd";
import {
  FileMarkdownOutlined,
  FolderOpenOutlined,
  FolderOutlined,
  InboxOutlined,
  PlayCircleOutlined,
  StopOutlined,
  DeleteOutlined,
  CopyOutlined,
} from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useConvertStore } from "../../stores/convertStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { convertApi } from "../../services/api";
import { openFilePath, revealFilePath } from "../../shared/openers";
import {
  CONVERT_ALLOWED_EXTENSIONS,
  type ConvertRequest,
} from "../../types";
import {
  LogStream,
  statusColor,
  cap,
} from "../translate/ProgressLogPanel";
import type { LogLine } from "../../stores/translateStore";

const { Text, Paragraph } = Typography;

function isAllowedPath(path: string): boolean {
  const lower = path.toLowerCase();
  return CONVERT_ALLOWED_EXTENSIONS.some((ext) => lower.endsWith(`.${ext}`));
}

function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export default function ConvertPage() {
  const { t } = useTranslation();
  const st = useConvertStore();
  const settings = useSettingsStore((s) => s.settings);
  const [busy, setBusy] = useState(false);
  const logRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (settings && !st.outputDir) st.setOutputDir(settings.default_output_dir);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [settings]);

  useEffect(() => {
    convertApi
      .markitdownInfo()
      .then((info) => {
        useConvertStore.getState().setMarkitdown(info.installed, info.hint);
      })
      .catch(() => {
        useConvertStore
          .getState()
          .setMarkitdown(false, t("convert.noSidecar"));
      });
  }, [t]);

  useEffect(() => {
    if (logRef.current) logRef.current.scrollTop = logRef.current.scrollHeight;
  }, [st.logs]);

  // Drag & drop via Tauri native event (only when this page is mounted).
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    (async () => {
      const app = getCurrentWebview();
      const fn = await app.onDragDropEvent(async (e) => {
        if (e.payload.type !== "drop") return;
        // Ignore drops while a conversion is running.
        if (useConvertStore.getState().status === "running") return;
        const paths = e.payload.paths.filter(isAllowedPath);
        if (!paths.length) {
          const any = e.payload.paths.length > 0;
          if (any) message.warning(t("convert.badExtension"));
          return;
        }
        if (paths.length > 1) {
          message.info(t("convert.oneFileOnly"));
        }
        const path = paths[0];
        useConvertStore.getState().setInput(path, fileName(path));
      });
      if (cancelled) {
        fn();
        return;
      }
      unlisten = fn;
    })();
    return () => {
      cancelled = true;
      void unlisten?.();
    };
  }, [t]);

  async function pickFile() {
    const res = await openDialog({
      multiple: false,
      filters: [
        {
          name: "Documents",
          extensions: [...CONVERT_ALLOWED_EXTENSIONS],
        },
      ],
    });
    const path = Array.isArray(res) ? res[0] : res;
    if (!path || typeof path !== "string") return;
    if (!isAllowedPath(path)) {
      message.warning(t("convert.badExtension"));
      return;
    }
    st.setInput(path, fileName(path));
  }

  async function pickOutputDir() {
    const res = await openDialog({ directory: true });
    if (typeof res === "string") st.setOutputDir(res);
  }

  const canStart =
    !!st.inputPath &&
    !!st.outputDir.trim() &&
    st.markitdownInstalled !== false &&
    st.status !== "running";

  async function start() {
    if (!st.inputPath) {
      message.warning(t("convert.noFile"));
      return;
    }
    if (!isAllowedPath(st.inputPath)) {
      message.warning(t("convert.badExtension"));
      return;
    }
    if (!st.outputDir.trim()) {
      message.warning(t("convert.noOutputDir"));
      return;
    }
    if (st.markitdownInstalled === false) {
      message.error(st.markitdownHint || t("convert.noSidecar"));
      return;
    }
    setBusy(true);
    st.clearLogs();
    st.setOutputFile("");
    st.setStatusMessage("");
    st.setTaskId(null);
    st.setStatus("running");
    const req: ConvertRequest = {
      input_path: st.inputPath,
      output_dir: st.outputDir,
    };
    try {
      const taskId = await convertApi.start(req);
      st.setTaskId(taskId);
    } catch (e) {
      st.setStatus("error");
      st.setStatusMessage(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function cancel() {
    if (st.taskId) await convertApi.cancel(st.taskId);
  }

  return (
    <Space direction="vertical" style={{ width: "100%" }} size={16}>
      {st.markitdownInstalled === false && (
        <Alert
          type="warning"
          showIcon
          message={st.markitdownHint || t("convert.noSidecar")}
        />
      )}

      <Card title={t("convert.title")} className="glass-card">
        <Alert
          type="info"
          showIcon
          style={{ marginBottom: 16 }}
          message={t("convert.disclaimer")}
        />

        <div
          onDragOver={(e) => e.preventDefault()}
          onClick={pickFile}
          className={`drag-drop-zone ${st.inputPath ? "has-files" : ""}`}
          style={{ marginBottom: 16 }}
        >
          <InboxOutlined className="drag-drop-icon" />
          <div style={{ marginTop: 12 }}>
            <Text strong style={{ fontSize: 15 }}>
              {t("convert.dropHere")}
            </Text>
          </div>
          <div style={{ marginTop: 4 }}>
            <Text type="secondary" style={{ fontSize: 13 }}>
              {t("convert.formatsHint")}
            </Text>
          </div>
        </div>

        {st.inputPath ? (
          <Space style={{ width: "100%", justifyContent: "space-between" }}>
            <Space>
              <FileMarkdownOutlined style={{ color: "#6366f1" }} />
              <Text style={{ wordBreak: "break-all", fontWeight: 500 }}>
                {st.inputName || st.inputPath}
              </Text>
            </Space>
            <Button
              size="small"
              type="text"
              danger
              icon={<DeleteOutlined />}
              disabled={st.status === "running"}
              onClick={() => st.clearInput()}
            >
              {t("convert.remove")}
            </Button>
          </Space>
        ) : null}
      </Card>

      <Card className="glass-card">
        <Space style={{ width: "100%" }} direction="vertical" size={16}>
          <div
            style={{
              display: "flex",
              gap: 12,
              alignItems: "center",
              flexWrap: "wrap",
            }}
          >
            <span style={{ flex: 1, minWidth: 280 }}>
              <Text
                type="secondary"
                style={{ display: "block", fontSize: 12, marginBottom: 4 }}
              >
                {t("convert.outputDir")}
              </Text>
              <Input
                style={{ width: "100%" }}
                value={st.outputDir}
                onChange={(e) => st.setOutputDir(e.target.value)}
                placeholder={t("convert.selectOutputDir")}
              />
            </span>
            <Button
              icon={<FolderOpenOutlined />}
              onClick={pickOutputDir}
              style={{ marginTop: 20 }}
            >
              {t("convert.selectOutputDir")}
            </Button>
          </div>
          <Space size="middle">
            <Button
              type="primary"
              icon={<PlayCircleOutlined />}
              disabled={!canStart || busy}
              loading={busy}
              onClick={start}
              style={{ height: 38, paddingInline: 24, fontWeight: 600 }}
            >
              {t("convert.start")}
            </Button>
            <Button
              danger
              icon={<StopOutlined />}
              disabled={st.status !== "running"}
              onClick={cancel}
              style={{ height: 38, paddingInline: 20 }}
            >
              {t("convert.cancel")}
            </Button>
          </Space>
        </Space>
      </Card>

      <Card
        title={t("convert.status")}
        className="glass-card"
        extra={
          st.status !== "idle" && (
            <Tag
              color={statusColor[st.status]}
              style={{ borderRadius: 6, fontWeight: 550 }}
            >
              {t(`convert.status${cap(st.status)}`)}
            </Tag>
          )
        }
      >
        {st.status === "running" && (
          <Progress percent={100} status="active" showInfo={false} />
        )}
        {st.statusMessage && (
          <Paragraph
            type={st.status === "error" ? "danger" : undefined}
            style={{ marginTop: 12, marginBottom: 0 }}
          >
            {st.statusMessage}
          </Paragraph>
        )}
        {st.outputFile && (
          <Space style={{ marginTop: 12 }}>
            <Button
              size="small"
              icon={<FileMarkdownOutlined style={{ color: "#6366f1" }} />}
              onClick={() => void openFilePath(st.outputFile)}
            >
              {t("convert.openFile")}
            </Button>
            <Tooltip title={st.outputFile}>
              <Button
                size="small"
                icon={<FolderOutlined />}
                onClick={() => void revealFilePath(st.outputFile)}
              >
                {t("convert.openFolder")}
              </Button>
            </Tooltip>
            <Text type="secondary" style={{ fontSize: 12, wordBreak: "break-all" }}>
              {st.outputFile}
            </Text>
          </Space>
        )}
      </Card>

      <Card
        title={
          <Space>
            <span>{t("convert.log")}</span>
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
        className="glass-card"
      >
        <LogStream
          logs={st.logs as LogLine[]}
          emptyText={t("convert.logEmpty")}
          containerRef={logRef}
        />
      </Card>
    </Space>
  );
}

