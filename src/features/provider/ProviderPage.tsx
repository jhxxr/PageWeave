import { useEffect, useState } from "react";
import {
  AutoComplete,
  Button,
  Card,
  Empty,
  Form,
  Input,
  Modal,
  Popconfirm,
  Select,
  Space,
  Switch,
  Table,
  Tag,
  Tooltip,
  message,
} from "antd";
import {
  PlusOutlined,
  ThunderboltOutlined,
  CloudDownloadOutlined,
  EditOutlined,
  DeleteOutlined,
  StarOutlined,
  StarFilled,
  EyeInvisibleOutlined,
  EyeOutlined,
} from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { useProviderStore } from "../../stores/providerStore";
import { providerApi } from "../../services/api";
import type {
  ProviderCategory,
  ProviderPayload,
  ProviderPreset,
  ProviderRecord,
} from "../../types";

export default function ProviderPage() {
  const { t } = useTranslation();
  const { providers, load } = useProviderStore();
  const [presets, setPresets] = useState<ProviderPreset[]>([]);
  const [editing, setEditing] = useState<ProviderRecord | null>(null);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    providerApi.listPresets().then(setPresets).catch(() => {});
  }, []);

  async function refresh() {
    await load();
  }

  function onAdd() {
    setEditing(null);
    setOpen(true);
  }
  function onEdit(rec: ProviderRecord) {
    setEditing(rec);
    setOpen(true);
  }

  async function onDelete(rec: ProviderRecord) {
    await providerApi.remove(rec.id);
    useProviderStore.getState().removeLocal(rec.id);
    message.success(t("common.delete"));
  }

  async function onSetDefault(rec: ProviderRecord) {
    await providerApi.setDefault(rec.id);
    useProviderStore.getState().setDefaultLocal(rec.id);
  }

  return (
    <Space direction="vertical" style={{ width: "100%" }} size={16}>
      <Card
        title={t("provider.title")}
        variant="borderless"
        extra={
          <Space>
            <Button icon={<PlusOutlined />} type="primary" onClick={onAdd}>
              {t("provider.add")}
            </Button>
            <ExportButton />
          </Space>
        }
      >
        {providers.length === 0 ? (
          <Empty description={t("provider.empty")} />
        ) : (
          <Table
            rowKey="id"
            dataSource={providers}
            pagination={false}
            columns={[
              {
                title: t("provider.name"),
                dataIndex: "name",
                render: (v: string, r: ProviderRecord) => (
                  <Space>
                    <span>{v}</span>
                    {r.is_applied && (
                      <Tag color="gold">
                        <StarFilled /> {t("provider.isDefault")}
                      </Tag>
                    )}
                    {r.has_api_key ? (
                      <Tag color="green">{t("provider.keySet")}</Tag>
                    ) : (
                      <Tag>{t("provider.keyNotSet")}</Tag>
                    )}
                  </Space>
                ),
              },
              {
                title: t("provider.category"),
                dataIndex: "category",
                width: 120,
              },
              {
                title: t("provider.baseUrl"),
                dataIndex: "base_url",
                render: (v: string) => (
                  <span style={{ wordBreak: "break-all" }}>{v}</span>
                ),
              },
              {
                title: t("provider.model"),
                dataIndex: "default_model",
                width: 180,
              },
              {
                title: t("provider.enabled"),
                dataIndex: "is_enabled",
                width: 80,
                render: (v: boolean) => (v ? <Tag color="blue">✓</Tag> : "—"),
              },
              {
                title: t("common.ok"),
                width: 280,
                render: (_: unknown, r: ProviderRecord) => (
                  <Space size="small">
                    <Button
                      size="small"
                      onClick={() => onSetDefault(r)}
                      disabled={r.is_applied}
                      icon={r.is_applied ? <StarFilled /> : <StarOutlined />}
                    >
                      {t("provider.setDefault")}
                    </Button>
                    <Button size="small" icon={<EditOutlined />} onClick={() => onEdit(r)}>
                      {t("provider.edit")}
                    </Button>
                    <Popconfirm
                      title={t("provider.confirmDelete")}
                      onConfirm={() => onDelete(r)}
                    >
                      <Button size="small" danger icon={<DeleteOutlined />}>
                        {t("provider.delete")}
                      </Button>
                    </Popconfirm>
                  </Space>
                ),
              },
            ]}
          />
        )}
      </Card>

      <ProviderFormModal
        open={open}
        editing={editing}
        presets={presets}
        onClose={() => setOpen(false)}
        onSaved={() => {
          setOpen(false);
          void refresh();
        }}
      />
    </Space>
  );
}

function ExportButton() {
  const { t } = useTranslation();
  async function doExport() {
    const { invoke } = await import("@tauri-apps/api/core");
    const data = await invoke("export_providers");
    const blob = new Blob([JSON.stringify(data, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "pageweave-providers.json";
    a.click();
    URL.revokeObjectURL(url);
    message.success(t("provider.exportNoKey"));
  }
  return (
    <Tooltip title={t("provider.exportNoKey")}>
      <Button onClick={doExport}>{t("provider.export")}</Button>
    </Tooltip>
  );
}

interface FormProps {
  open: boolean;
  editing: ProviderRecord | null;
  presets: ProviderPreset[];
  onClose: () => void;
  onSaved: () => void;
}

function ProviderFormModal({ open, editing, presets, onClose, onSaved }: FormProps) {
  const { t } = useTranslation();
  const [form] = Form.useForm<ProviderPayload>();
  const [showKey, setShowKey] = useState(false);
  const [testing, setTesting] = useState(false);
  const [fetching, setFetching] = useState(false);
  const [models, setModels] = useState<string[]>([]);

  useEffect(() => {
    if (!open) return;
    if (editing) {
      form.setFieldsValue({
        name: editing.name,
        category: editing.category,
        base_url: editing.base_url,
        api_key: "",
        models: editing.models,
        default_model: editing.default_model,
        is_enabled: editing.is_enabled,
        notes: editing.notes,
        extra: (editing.extra as Record<string, unknown>) ?? {},
      });
      setModels(editing.models);
    } else {
      form.resetFields();
      form.setFieldsValue({
        category: "openai",
        is_enabled: true,
        api_key: "",
        models: [],
        extra: {},
      });
      setModels([]);
    }
    setShowKey(false);
  }, [open, editing, form]);

  async function applyPreset(cat: ProviderCategory) {
    const p = presets.find((x) => x.category === cat);
    if (!p) return;
    const cur = form.getFieldValue("base_url");
    if (!cur) form.setFieldValue("base_url", p.base_url);
    if (p.models.length && !form.getFieldValue("default_model")) {
      form.setFieldValue("default_model", p.models[0]);
      setModels(p.models);
      form.setFieldValue("models", p.models);
    }
  }

  async function doTest() {
    const v = form.getFieldsValue();
    if (!editing && !v.api_key) {
      message.warning(t("provider.keyNotSet"));
      return;
    }
    const id = editing?.api_key_id ?? "";
    if (!id && !v.api_key) return;
    setTesting(true);
    try {
      // If editing and key unchanged, use stored id; the backend reads keyring.
      const res = await providerApi.testConnection({
        api_key_id: id,
        base_url: v.base_url,
        model: v.default_model,
      });
      if (res.ok) message.success(`${t("provider.testOk")} (${res.latency_ms ?? 0}ms)`);
      else message.error(`${t("provider.testFail")}: ${res.message}`);
    } catch (e) {
      message.error(`${t("provider.testFail")}: ${e}`);
    } finally {
      setTesting(false);
    }
  }

  async function doFetchModels() {
    const v = form.getFieldsValue();
    if (!editing && !v.api_key) {
      message.warning(t("provider.keyNotSet"));
      return;
    }
    setFetching(true);
    try {
      const res = await providerApi.fetchModels({
        api_key_id: editing?.api_key_id ?? "",
        base_url: v.base_url,
      });
      if (res.ok && res.models.length) {
        setModels(res.models);
        form.setFieldValue("models", res.models);
        if (!form.getFieldValue("default_model"))
          form.setFieldValue("default_model", res.models[0]);
        message.success(t("provider.fetchOk", { count: res.models.length }));
      } else {
        message.warning(`${t("provider.fetchFail")}: ${res.message}`);
      }
    } catch (e) {
      message.warning(`${t("provider.fetchFail")}: ${e}`);
    } finally {
      setFetching(false);
    }
  }

  async function save() {
    const v = await form.validateFields();
    let rec: ProviderRecord;
    if (editing) {
      rec = await providerApi.update(editing.id, v);
    } else {
      rec = await providerApi.create(v);
    }
    useProviderStore.getState().upsertLocal(rec);
    message.success(t("common.save"));
    onSaved();
  }

  return (
    <Modal
      open={open}
      title={editing ? t("provider.edit") : t("provider.add")}
      onCancel={onClose}
      onOk={save}
      width={620}
      okText={t("provider.save")}
      cancelText={t("provider.cancel")}
    >
      <Form form={form} layout="vertical">
        <Form.Item name="category" label={t("provider.category")}>
          <Select
            options={presets.map((p) => ({
              value: p.category,
              label: p.label,
            }))}
            onChange={(v: ProviderCategory) => applyPreset(v)}
          />
        </Form.Item>
        <Form.Item name="name" label={t("provider.name")} rules={[{ required: true }]}>
          <Input />
        </Form.Item>
        <Form.Item name="base_url" label={t("provider.baseUrl")} rules={[{ required: true }]}>
          <Input placeholder="https://api.openai.com/v1" />
        </Form.Item>
        <Form.Item
          name="api_key"
          label={
            <Space>
              <span>{t("provider.apiKey")}</span>
              {editing?.has_api_key && <Tag color="green">{t("provider.keySet")}</Tag>}
            </Space>
          }
          extra={editing ? "留空表示不修改已有 Key" : undefined}
        >
          <Input
            type={showKey ? "text" : "password"}
            placeholder="sk-..."
            addonAfter={
              <span onClick={() => setShowKey((s) => !s)} style={{ cursor: "pointer" }}>
                {showKey ? <EyeInvisibleOutlined /> : <EyeOutlined />}
                {showKey ? ` ${t("provider.hideKey")}` : ` ${t("provider.revealKey")}`}
              </span>
            }
          />
        </Form.Item>
        <Form.Item label={t("provider.model")}>
          <Space.Compact style={{ width: "100%" }}>
            <Form.Item name="default_model" noStyle rules={[{ required: true }]}>
              <AutoComplete
                style={{ width: "100%" }}
                options={models.map((m) => ({ value: m, label: m }))}
              />
            </Form.Item>
            <Tooltip title={t("provider.fetchHint")}>
              <Button
                icon={<CloudDownloadOutlined />}
                loading={fetching}
                onClick={doFetchModels}
              >
                {t("provider.fetchModels")}
              </Button>
            </Tooltip>
            <Tooltip title={t("provider.testConnection")}>
              <Button icon={<ThunderboltOutlined />} loading={testing} onClick={doTest}>
                {testing ? t("provider.testing") : t("provider.testConnection")}
              </Button>
            </Tooltip>
          </Space.Compact>
        </Form.Item>
        <Form.Item name="models" hidden>
          <Input />
        </Form.Item>
        <Form.Item name="is_enabled" label={t("provider.enabled")} valuePropName="checked">
          <Switch />
        </Form.Item>
        <Form.Item name="notes" label={t("provider.notes")}>
          <Input.TextArea rows={2} />
        </Form.Item>
      </Form>
    </Modal>
  );
}
