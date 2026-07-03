import { Card, Empty } from "antd";
import { useTranslation } from "react-i18next";

export default function ParamsPage() {
  const { t } = useTranslation();
  return (
    <Card title={t("params.title")} variant="borderless">
      <Empty description={t("params.placeholder")} />
    </Card>
  );
}
