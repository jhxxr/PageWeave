import { Card, Empty, Tag, Typography } from "antd";
import { useTranslation } from "react-i18next";
import { useTranslateStore } from "../../stores/translateStore";

const { Text } = Typography;

export default function TasksPage() {
  const { t } = useTranslation();
  const st = useTranslateStore();
  const file = st.files[0];
  return (
    <Card title={t("tasks.title")} variant="borderless">
      {!file ? (
        <Empty description={t("tasks.placeholder")} />
      ) : (
        <div style={{ maxWidth: 600 }}>
          <p>
            <Text type="secondary">{t("translate.fileCol")}: </Text>
            <Text>{file.name}</Text>
          </p>
          <p>
            <Text type="secondary">{t("translate.statusCol")}: </Text>
            <Tag>{t(`translate.status${cap(st.status)}`)}</Tag>
          </p>
          <p>
            <Text type="secondary">{t("translate.progress")}: </Text>
            <Text>{st.progress}%</Text>
          </p>
          {st.outputFiles.map((f) => (
            <p key={f}>
              <Text type="secondary">{t("translate.openFile")}: </Text>
              <Text style={{ wordBreak: "break-all" }}>{f}</Text>
            </p>
          ))}
        </div>
      )}
    </Card>
  );
}

function cap(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
