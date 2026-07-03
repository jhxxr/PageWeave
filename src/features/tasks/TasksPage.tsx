import { Button, Card, Empty, Progress, Space, Tag, Tooltip, Typography } from "antd";
import { FilePdfOutlined, FolderOutlined } from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import { useTranslateStore } from "../../stores/translateStore";

const { Paragraph, Text } = Typography;

const statusColor: Record<string, string> = {
  idle: "default",
  running: "processing",
  success: "success",
  error: "error",
  cancelled: "warning",
};

export default function TasksPage() {
  const { t } = useTranslation();
  const st = useTranslateStore();
  const file = st.files[0];
  const latestLogs = st.logs.slice(-8);

  return (
    <Card title={t("tasks.title")} variant="borderless">
      {!file ? (
        <Empty description={t("tasks.noCurrent")} />
      ) : (
        <Space direction="vertical" size="middle" style={{ width: "100%", maxWidth: 760 }}>
          <Space direction="vertical" size={4}>
            <Text type="secondary">{t("tasks.current")}</Text>
            <Space>
              <FilePdfOutlined />
              <Text style={{ wordBreak: "break-all" }}>{file.name}</Text>
              <Tag color={statusColor[st.status]}>{t(`translate.status${cap(st.status)}`)}</Tag>
            </Space>
          </Space>

          <div>
            <Text type="secondary">{t("translate.progress")}</Text>
            <Progress
              percent={st.progress}
              status={st.status === "error" ? "exception" : undefined}
              style={{ marginTop: 4 }}
            />
            {st.stage && <Text type="secondary">{st.stage}</Text>}
          </div>

          {st.statusMessage && (
            <Paragraph type={st.status === "error" ? "danger" : undefined}>
              <Text type="secondary">{t("tasks.statusMessage")}: </Text>
              {st.statusMessage}
            </Paragraph>
          )}

          {st.outputFiles.length > 0 && (
            <Space direction="vertical" size={6}>
              <Text type="secondary">{t("tasks.outputFiles")}</Text>
              {st.outputFiles.map((f) => (
                <Space key={f} wrap>
                  <Text style={{ wordBreak: "break-all" }}>{f}</Text>
                  <Button size="small" icon={<FilePdfOutlined />} onClick={() => openPath(f)}>
                    {t("translate.openFile")}
                  </Button>
                  <Tooltip title={f}>
                    <Button size="small" icon={<FolderOutlined />} onClick={() => revealItemInDir(f)}>
                      {t("translate.openFolder")}
                    </Button>
                  </Tooltip>
                </Space>
              ))}
            </Space>
          )}

          {latestLogs.length > 0 && (
            <Space direction="vertical" size={6} style={{ width: "100%" }}>
              <Text type="secondary">{t("tasks.latestLog")}</Text>
              <div
                style={{
                  maxHeight: 180,
                  overflow: "auto",
                  background: "rgba(0,0,0,0.04)",
                  padding: 8,
                  borderRadius: 6,
                  fontFamily: "ui-monospace, monospace",
                  fontSize: 12,
                }}
              >
                {latestLogs.map((l) => (
                  <div key={l.id}>
                    <span style={{ color: l.stream === "stderr" ? "#c0392b" : undefined }}>
                      {l.text}
                    </span>
                  </div>
                ))}
              </div>
            </Space>
          )}
        </Space>
      )}
    </Card>
  );
}

function cap(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
