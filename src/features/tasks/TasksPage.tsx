import { Button, Card, Empty, Space, Tag, Tooltip, Typography } from "antd";
import { FilePdfOutlined, FolderOutlined } from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import { useTranslateStore } from "../../stores/translateStore";
import {
  LogStream,
  ProgressOverview,
  cap,
  statusColor,
} from "../translate/ProgressLogPanel";

const { Paragraph, Text } = Typography;

export default function TasksPage() {
  const { t } = useTranslation();
  const st = useTranslateStore();
  const file = st.files[0];
  const latestLogs = st.logs.slice(-8);

  return (
    <Card title={t("tasks.title")} className="glass-card">
      {!file ? (
        <Empty description={t("tasks.noCurrent")} />
      ) : (
        <Space direction="vertical" size="middle" style={{ width: "100%", maxWidth: 760 }}>
          <Space direction="vertical" size={4}>
            <Text type="secondary" style={{ fontSize: 12 }}>{t("tasks.current")}</Text>
            <Space size="middle">
              <FilePdfOutlined style={{ color: "#ef4444", fontSize: 16 }} />
              <Text style={{ wordBreak: "break-all", fontWeight: 600 }}>{file.name}</Text>
              <Tag color={statusColor[st.status]} style={{ borderRadius: 6, fontWeight: 500 }}>{t(`translate.status${cap(st.status)}`)}</Tag>
            </Space>
          </Space>

          <div>
            <Text type="secondary" style={{ display: "block", fontSize: 12, marginBottom: 8 }}>{t("translate.progress")}</Text>
            <ProgressOverview
              percent={st.progress}
              status={st.status}
              stage={st.stage}
              latestLog={st.logs[st.logs.length - 1]?.text}
              stageLabel={t("translate.currentStage")}
              latestLabel={t("tasks.latestLog")}
            />
          </div>

          {st.statusMessage && (
            <Paragraph type={st.status === "error" ? "danger" : undefined} style={{ marginTop: 8 }}>
              <Text type="secondary">{t("tasks.statusMessage")}: </Text>
              {st.statusMessage}
            </Paragraph>
          )}

          {st.outputFiles.length > 0 && (
            <Space direction="vertical" size={8}>
              <Text type="secondary" style={{ fontSize: 12 }}>{t("tasks.outputFiles")}</Text>
              {st.outputFiles.map((f) => (
                <Space key={f} wrap size="middle">
                  <Text style={{ wordBreak: "break-all", fontWeight: 500 }}>{f.split(/[\\/]/).pop() ?? f}</Text>
                  <Button size="small" icon={<FilePdfOutlined style={{ color: "#ef4444" }} />} onClick={() => openPath(f)}>
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
            <Space direction="vertical" size={8} style={{ width: "100%" }}>
              <Text type="secondary" style={{ fontSize: 12 }}>{t("tasks.latestLog")}</Text>
              <LogStream logs={latestLogs} emptyText={t("translate.logEmpty")} height={180} />
            </Space>
          )}
        </Space>
      )}
    </Card>
  );

}
