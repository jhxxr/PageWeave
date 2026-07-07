import { useEffect, useState } from "react";
import { Button, Card, Empty, Progress, Space, Table, Tag, Tooltip, Typography, message } from "antd";
import { DeleteOutlined, FilePdfOutlined, FolderOutlined, ReloadOutlined } from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { translateApi } from "../../services/api";
import { openFilePath, revealFilePath } from "../../shared/openers";
import type { TaskRecord } from "../../types";
import { cap, statusColor } from "../translate/ProgressLogPanel";

const { Text } = Typography;

export default function TasksPage() {
  const { t } = useTranslation();
  const [records, setRecords] = useState<TaskRecord[]>([]);
  const [loading, setLoading] = useState(false);

  async function load() {
    setLoading(true);
    try {
      setRecords(await translateApi.listTaskRecords());
    } catch (e) {
      message.error(`加载历史记录失败：${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setLoading(false);
    }
  }

  async function remove(id: string) {
    try {
      await translateApi.deleteTaskRecord(id);
      setRecords((items) => items.filter((item) => item.id !== id));
    } catch (e) {
      message.error(`删除历史记录失败：${e instanceof Error ? e.message : String(e)}`);
    }
  }

  useEffect(() => {
    void load();
  }, []);

  return (
    <Card
      title={t("tasks.title")}
      className="glass-card"
      extra={
        <Button size="small" icon={<ReloadOutlined />} loading={loading} onClick={() => void load()}>
          {t("settings.refresh")}
        </Button>
      }
    >
      <Table
        rowKey="id"
        loading={loading}
        dataSource={records}
        pagination={{ pageSize: 8 }}
        locale={{ emptyText: <Empty description={t("tasks.noCurrent")} /> }}
        expandable={{
          expandedRowRender: (record) => (
            <Space direction="vertical" size={8} style={{ width: "100%" }}>
              <Text type="secondary">{record.message || record.stage || record.output_dir}</Text>
              {record.output_files.map((f) => (
                <Space key={f} wrap>
                  <Text style={{ wordBreak: "break-all", fontWeight: 500 }}>{f.split(/[\\/]/).pop() ?? f}</Text>
                  <Button size="small" icon={<FilePdfOutlined style={{ color: "#ef4444" }} />} onClick={() => void openFilePath(f)}>
                    {t("translate.openFile")}
                  </Button>
                  <Tooltip title={f}>
                    <Button size="small" icon={<FolderOutlined />} onClick={() => void revealFilePath(f)}>
                      {t("translate.openFolder")}
                    </Button>
                  </Tooltip>
                </Space>
              ))}
            </Space>
          ),
        }}
        columns={[
          {
            title: t("translate.fileCol"),
            dataIndex: "pdf_paths",
            render: (paths: string[]) => {
              const path = paths[0] ?? "";
              return (
                <Space>
                  <FilePdfOutlined style={{ color: "#ef4444" }} />
                  <Tooltip title={path}>
                    <Text style={{ wordBreak: "break-all", fontWeight: 600 }}>{path.split(/[\\/]/).pop() ?? path}</Text>
                  </Tooltip>
                </Space>
              );
            },
          },
          {
            title: t("translate.statusCol"),
            dataIndex: "status",
            width: 110,
            render: (status: string) => (
              <Tag color={statusColor[status]} style={{ borderRadius: 6, fontWeight: 500 }}>
                {t(`translate.status${cap(status)}`)}
              </Tag>
            ),
          },
          {
            title: t("translate.progress"),
            dataIndex: "progress",
            width: 140,
            render: (progress: number, record) => (
              <Progress percent={record.status === "success" ? 100 : progress} size="small" />
            ),
          },
          {
            title: t("translate.model"),
            dataIndex: "model",
            width: 180,
            ellipsis: true,
          },
          {
            title: t("tasks.createdAt"),
            dataIndex: "created_at",
            width: 180,
            render: (v: string) => new Date(v).toLocaleString(),
          },
          {
            title: t("translate.actionCol"),
            width: 96,
            render: (_: unknown, record) => (
              <Space>
                {record.output_files[0] && (
                  <Tooltip title={record.output_files[0]}>
                    <Button size="small" icon={<FilePdfOutlined />} onClick={() => void openFilePath(record.output_files[0])} />
                  </Tooltip>
                )}
                <Button danger size="small" icon={<DeleteOutlined />} onClick={() => void remove(record.id)} />
              </Space>
            ),
          },
        ]}
      />
    </Card>
  );

}
