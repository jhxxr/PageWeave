import { useState } from "react";
import {
  Alert,
  Button,
  Card,
  Collapse,
  Empty,
  Input,
  InputNumber,
  Radio,
  Select,
  Space,
  Switch,
  Tag,
  Tooltip,
  Typography,
} from "antd";
import {
  CodeOutlined,
  DeleteOutlined,
  FileAddOutlined,
  UndoOutlined,
} from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useTranslateStore } from "../../stores/translateStore";
import type { FontFamily, OcrMode } from "../../types";
import { previewCliArgs } from "./cliPreview";

const { Text, Paragraph } = Typography;

/** A label + control row used throughout the page. */
function Row({
  label,
  help,
  htmlFor,
  children,
  extra,
}: {
  label: string;
  help?: string;
  htmlFor?: string;
  children: React.ReactNode;
  extra?: React.ReactNode;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 16,
        margin: "10px 0",
        flexWrap: "wrap",
        paddingBottom: 10,
        borderBottom: "1px dashed rgba(100, 116, 139, 0.08)",
      }}
    >
      <label
        htmlFor={htmlFor}
        style={{
          minWidth: 220,
          display: "block",
        }}
      >
        <Space size={6}>
          <Text style={{ fontWeight: 550 }}>{label}</Text>
          {extra}
        </Space>
        {help && (
          <div style={{ marginTop: 2 }}>
            <Text type="secondary" style={{ fontSize: 12, lineHeight: "1.4", display: "block" }}>
              {help}
            </Text>
          </div>
        )}
      </label>
      <div style={{ flex: 1, minWidth: 240 }}>{children}</div>
    </div>
  );
}

export default function ParamsPage() {
  const { t } = useTranslation();
  const st = useTranslateStore();
  const a = st.advanced;
  const [previewOpen, setPreviewOpen] = useState(false);

  const outputMode = st.outputMode;
  const enhanceOn = a.enhance_compatibility !== false;
  const dualOnlyDisabled = outputMode === "mono";

  async function addGlossaryFiles() {
    const res = await openDialog({
      multiple: true,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    const paths = Array.isArray(res) ? res : res ? [res] : [];
    if (!paths.length) return;
    const existing = a.glossary_files ?? [];
    const merged = Array.from(new Set([...existing, ...paths]));
    st.setAdvanced({ glossary_files: merged });
  }

  const cliPreview = previewOpen
    ? previewCliArgs(a, outputMode).join(" \\\n  ")
    : "";

  return (
    <Card
      title={t("params.title")}
      className="glass-card"
      extra={
        <Space>
          <Tooltip title={t("params.resetHint")}>
            <Button icon={<UndoOutlined />} onClick={st.resetAdvanced}>
              {t("params.reset")}
            </Button>
          </Tooltip>
          <Tooltip title={t("params.previewHint")}>
            <Button
              icon={<CodeOutlined />}
              type={previewOpen ? "primary" : "default"}
              onClick={() => setPreviewOpen((v) => !v)}
            >
              {t("params.preview")}
            </Button>
          </Tooltip>
        </Space>
      }
    >
      <Alert
        type="info"
        showIcon
        message={t("params.intro")}
        style={{ marginBottom: 16 }}
      />

      <Collapse
        defaultActiveKey={["scope"]}
        items={[
          {
            key: "scope",
            label: t("params.scope.header"),
            children: (
              <>
                <Row
                  label={t("params.scope.pages")}
                  help={t("params.scope.pagesHelp")}
                  htmlFor="pages"
                >
                  <Input
                    id="pages"
                    placeholder={t("params.scope.pagesPlaceholder")}
                    value={a.pages ?? ""}
                    onChange={(e) => st.setAdvanced({ pages: e.target.value })}
                    allowClear
                  />
                </Row>
                <Row
                  label={t("params.scope.minTextLength")}
                  help={t("params.scope.minTextLengthHelp")}
                >
                  <InputNumber
                    min={1}
                    max={1000}
                    placeholder="5"
                    style={{ width: 120 }}
                    value={a.min_text_length ?? null}
                    onChange={(v) =>
                      st.setAdvanced({
                        min_text_length: v == null ? undefined : Number(v),
                      })
                    }
                  />
                </Row>
                <Row
                  label={t("params.scope.maxPagesPerPart")}
                  help={t("params.scope.maxPagesPerPartHelp")}
                >
                  <InputNumber
                    min={1}
                    max={1000}
                    style={{ width: 120 }}
                    value={a.max_pages_per_part ?? null}
                    onChange={(v) =>
                      st.setAdvanced({
                        max_pages_per_part: v == null ? undefined : Number(v),
                      })
                    }
                  />
                </Row>
              </>
            ),
          },
          {
            key: "glossary",
            label: t("params.glossary.header"),
            children: (
              <>
                <Row
                  label={t("params.glossary.files")}
                  help={t("params.glossary.filesHelp")}
                >
                  <Space direction="vertical" style={{ width: "100%" }}>
                    {!(a.glossary_files?.length ?? 0) && (
                      <Empty
                        image={Empty.PRESENTED_IMAGE_SIMPLE}
                        description={t("params.glossary.empty")}
                        style={{ margin: 0 }}
                      />
                    )}
                    {(a.glossary_files ?? []).map((f) => (
                      <Space
                        key={f}
                        style={{ width: "100%", justifyContent: "space-between" }}
                      >
                        <Text
                          style={{
                            wordBreak: "break-all",
                            maxWidth: 460,
                          }}
                          ellipsis={{ tooltip: f }}
                        >
                          {f}
                        </Text>
                        <Button
                          size="small"
                          type="text"
                          danger
                          icon={<DeleteOutlined />}
                          onClick={() =>
                            st.setAdvanced({
                              glossary_files: (a.glossary_files ?? []).filter(
                                (x) => x !== f,
                              ),
                            })
                          }
                        >
                          {t("params.glossary.removeFile")}
                        </Button>
                      </Space>
                    ))}
                    <Button
                      icon={<FileAddOutlined />}
                      onClick={addGlossaryFiles}
                    >
                      {t("params.glossary.addFiles")}
                    </Button>
                  </Space>
                </Row>
                <Row
                  label={t("params.glossary.noAutoExtract")}
                  help={t("params.glossary.noAutoExtractHelp")}
                >
                  <Switch
                    checked={a.no_auto_extract_glossary ?? false}
                    onChange={(v) =>
                      st.setAdvanced({ no_auto_extract_glossary: v })
                    }
                  />
                </Row>
                <Row
                  label={t("params.glossary.saveAutoExtracted")}
                  help={t("params.glossary.saveAutoExtractedHelp")}
                >
                  <Switch
                    checked={a.save_auto_extracted_glossary ?? false}
                    onChange={(v) =>
                      st.setAdvanced({ save_auto_extracted_glossary: v })
                    }
                  />
                </Row>
              </>
            ),
          },
          {
            key: "layout",
            label: t("params.layout.header"),
            children: (
              <>
                <Row label={t("params.layout.fontFamily")}>
                  <Select
                    style={{ width: 200 }}
                    value={a.primary_font_family ?? "auto"}
                    onChange={(v: FontFamily) =>
                      st.setAdvanced({
                        primary_font_family: v === "auto" ? undefined : v,
                      })
                    }
                    options={[
                      { value: "auto", label: t("params.layout.fontAuto") },
                      { value: "serif", label: t("params.layout.fontSerif") },
                      {
                        value: "sans-serif",
                        label: t("params.layout.fontSansSerif"),
                      },
                      {
                        value: "script",
                        label: t("params.layout.fontScript"),
                      },
                    ]}
                  />
                </Row>
                <Row
                  label={t("params.layout.useAlternatingPagesDual")}
                  help={t("params.layout.useAlternatingPagesDualHelp")}
                  extra={
                    dualOnlyDisabled ? (
                      <Text type="secondary" style={{ fontSize: 12 }}>
                        {t("params.layout.dualOnlyDisabled")}
                      </Text>
                    ) : null
                  }
                >
                  <Switch
                    checked={a.use_alternating_pages_dual ?? false}
                    disabled={dualOnlyDisabled}
                    onChange={(v) =>
                      st.setAdvanced({ use_alternating_pages_dual: v })
                    }
                  />
                </Row>
                <Row
                  label={t("params.layout.dualTranslateFirst")}
                  help={t("params.layout.dualTranslateFirstHelp")}
                  extra={
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      {dualOnlyDisabled
                        ? t("params.layout.dualOnlyDisabled")
                        : enhanceOn
                          ? t("params.layout.bundledDisabled")
                          : null}
                    </Text>
                  }
                >
                  <Tooltip
                    title={
                      dualOnlyDisabled || enhanceOn
                        ? dualOnlyDisabled
                          ? t("params.layout.dualOnlyDisabled")
                          : t("params.layout.bundledDisabled")
                        : ""
                    }
                  >
                    <span>
                      <Switch
                        checked={a.dual_translate_first ?? false}
                        disabled={dualOnlyDisabled || enhanceOn}
                        onChange={(v) =>
                          st.setAdvanced({ dual_translate_first: v })
                        }
                      />
                    </span>
                  </Tooltip>
                </Row>
              </>
            ),
          },
          {
            key: "ocr",
            label: t("params.ocr.header"),
            children: (
              <>
                <Row label={t("params.ocr.mode")}>
                  <Radio.Group
                    value={a.ocr_mode ?? "auto"}
                    onChange={(e) => {
                      const v = e.target.value as OcrMode;
                      st.setAdvanced({
                        ocr_mode: v === "auto" ? undefined : v,
                      });
                    }}
                  >
                    <Space direction="vertical">
                      <Radio value="auto">
                        <Tooltip title={t("params.ocr.modeAutoHelp")}>
                          <span>{t("params.ocr.modeAuto")}</span>
                        </Tooltip>
                      </Radio>
                      <Radio value="off">
                        <Tooltip title={t("params.ocr.modeOffHelp")}>
                          <span>{t("params.ocr.modeOff")}</span>
                        </Tooltip>
                      </Radio>
                      <Radio value="force">
                        <Tooltip title={t("params.ocr.modeForceHelp")}>
                          <span>{t("params.ocr.modeForce")}</span>
                        </Tooltip>
                      </Radio>
                    </Space>
                  </Radio.Group>
                </Row>
                <Row
                  label={t("params.ocr.enhanceCompatibility")}
                  help={t("params.ocr.enhanceCompatibilityHelp")}
                >
                  <Switch
                    checked={enhanceOn}
                    onChange={(v) =>
                      st.setAdvanced({ enhance_compatibility: v })
                    }
                  />
                </Row>
                <Row
                  label={t("params.ocr.skipClean")}
                  help={t("params.ocr.skipCleanHelp")}
                  extra={
                    enhanceOn ? (
                      <Tag>{t("params.ocr.bundled")}</Tag>
                    ) : null
                  }
                >
                  <Switch
                    checked={a.skip_clean ?? false}
                    disabled={enhanceOn}
                    onChange={(v) => st.setAdvanced({ skip_clean: v })}
                  />
                </Row>
                <Row
                  label={t("params.ocr.disableRichText")}
                  help={t("params.ocr.disableRichTextHelp")}
                  extra={
                    enhanceOn ? <Tag>{t("params.ocr.bundled")}</Tag> : null
                  }
                >
                  <Switch
                    checked={a.disable_rich_text_translate ?? false}
                    disabled={enhanceOn}
                    onChange={(v) =>
                      st.setAdvanced({ disable_rich_text_translate: v })
                    }
                  />
                </Row>
                <Row
                  label={t("params.ocr.translateTableText")}
                  help={t("params.ocr.translateTableTextHelp")}
                  extra={<Tag color="orange">{t("params.ocr.experimental")}</Tag>}
                >
                  <Switch
                    checked={a.translate_table_text ?? false}
                    onChange={(v) =>
                      st.setAdvanced({ translate_table_text: v })
                    }
                  />
                </Row>
                <Row
                  label={t("params.ocr.disableGraphicElement")}
                  help={t("params.ocr.disableGraphicElementHelp")}
                >
                  <Switch
                    checked={a.disable_graphic_element_process ?? false}
                    onChange={(v) =>
                      st.setAdvanced({ disable_graphic_element_process: v })
                    }
                  />
                </Row>
                <Row
                  label={t("params.ocr.noMergeAlternatingLineNumbers")}
                  help={t("params.ocr.noMergeAlternatingLineNumbersHelp")}
                >
                  <Switch
                    checked={a.no_merge_alternating_line_numbers ?? false}
                    onChange={(v) =>
                      st.setAdvanced({
                        no_merge_alternating_line_numbers: v,
                      })
                    }
                  />
                </Row>
                <Row
                  label={t("params.ocr.disableSameTextFallback")}
                  help={t("params.ocr.disableSameTextFallbackHelp")}
                >
                  <Switch
                    checked={a.disable_same_text_fallback ?? false}
                    onChange={(v) =>
                      st.setAdvanced({ disable_same_text_fallback: v })
                    }
                  />
                </Row>
              </>
            ),
          },
          {
            key: "cache",
            label: t("params.cache.header"),
            children: (
              <>
                <Row
                  label={t("params.cache.ignoreCache")}
                  help={t("params.cache.ignoreCacheHelp")}
                >
                  <Switch
                    checked={a.ignore_cache ?? false}
                    onChange={(v) => st.setAdvanced({ ignore_cache: v })}
                  />
                </Row>
                <Row
                  label={t("params.cache.poolMaxWorkers")}
                  help={t("params.cache.poolMaxWorkersHelp")}
                >
                  <InputNumber
                    min={1}
                    max={64}
                    style={{ width: 120 }}
                    placeholder="QPS"
                    value={a.pool_max_workers ?? null}
                    onChange={(v) =>
                      st.setAdvanced({
                        pool_max_workers: v == null ? undefined : Number(v),
                      })
                    }
                  />
                </Row>
                <Row
                  label={t("params.cache.termPoolMaxWorkers")}
                  help={t("params.cache.termPoolMaxWorkersHelp")}
                >
                  <InputNumber
                    min={1}
                    max={64}
                    style={{ width: 120 }}
                    placeholder="pool"
                    value={a.term_pool_max_workers ?? null}
                    onChange={(v) =>
                      st.setAdvanced({
                        term_pool_max_workers: v == null ? undefined : Number(v),
                      })
                    }
                  />
                </Row>
              </>
            ),
          },
          {
            key: "openai",
            label: t("params.openai.header"),
            children: (
              <>
                <Row
                  label={t("params.openai.customSystemPrompt")}
                  help={t("params.openai.customSystemPromptHelp")}
                >
                  <Input.TextArea
                    rows={4}
                    value={a.custom_system_prompt ?? ""}
                    onChange={(e) =>
                      st.setAdvanced({
                        custom_system_prompt: e.target.value || undefined,
                      })
                    }
                    maxLength={8000}
                    showCount
                  />
                </Row>
                <Row
                  label={t("params.openai.noSendTemperature")}
                  help={t("params.openai.noSendTemperatureHelp")}
                >
                  <Switch
                    checked={a.no_send_temperature ?? false}
                    onChange={(v) =>
                      st.setAdvanced({ no_send_temperature: v })
                    }
                  />
                </Row>
                <Row
                  label={t("params.openai.enableJsonMode")}
                  help={t("params.openai.enableJsonModeHelp")}
                >
                  <Switch
                    checked={a.enable_json_mode_if_requested ?? false}
                    onChange={(v) =>
                      st.setAdvanced({ enable_json_mode_if_requested: v })
                    }
                  />
                </Row>
                <Row
                  label={t("params.openai.sendDashscopeHeader")}
                  help={t("params.openai.sendDashscopeHeaderHelp")}
                >
                  <Switch
                    checked={a.send_dashscope_header ?? false}
                    onChange={(v) =>
                      st.setAdvanced({ send_dashscope_header: v })
                    }
                  />
                </Row>
                <Row
                  label={t("params.openai.reasoning")}
                  help={t("params.openai.reasoningHelp")}
                >
                  <Input
                    style={{ width: 240 }}
                    placeholder="auto / low / high"
                    value={a.openai_reasoning ?? ""}
                    onChange={(e) =>
                      st.setAdvanced({
                        openai_reasoning: e.target.value || undefined,
                      })
                    }
                    allowClear
                  />
                </Row>
              </>
            ),
          },
        ]}
      />

      {previewOpen && (
        <Card
          size="small"
          type="inner"
          title={t("params.previewTitle")}
          style={{ marginTop: 16 }}
        >
          <Paragraph>
            <pre
              style={{
                background: "rgba(0,0,0,0.03)",
                padding: 12,
                borderRadius: 6,
                margin: 0,
                whiteSpace: "pre-wrap",
                wordBreak: "break-all",
                fontSize: 12,
              }}
            >
              {cliPreview}
            </pre>
          </Paragraph>
          <Text type="secondary" style={{ fontSize: 12 }}>
            {t("params.previewDisclaimer")}
          </Text>
        </Card>
      )}
    </Card>
  );
}
