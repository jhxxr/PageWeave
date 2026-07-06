// Keep in sync with src-tauri/src/translate/args.rs :: build_args
//
// Pure preview of the babeldoc CLI args derived from the advanced-params slice
// + output mode. Used ONLY for display on the params page; the Rust side
// generates the real argv. If you change flag resolution there, mirror it here.
import type { AdvancedParams, OutputMode } from "../../types";

const BASE_FLAGS = [
  "--files <pdf>",
  "--output <dir>",
  "--lang-in",
  "--lang-out",
  "--openai",
  "--openai-model <model>",
  "--openai-base-url <url>",
  "--openai-api-key <key>",
  "--qps <n>",
];

export function previewCliArgs(
  advanced: AdvancedParams,
  outputMode: OutputMode,
): string[] {
  const a = advanced;
  const out: string[] = [...BASE_FLAGS];

  const enhance = a.enhance_compatibility ?? true;
  if (enhance) {
    out.push("--enhance-compatibility");
  } else {
    if (a.skip_clean) out.push("--skip-clean");
    if (a.disable_rich_text_translate) out.push("--disable-rich-text-translate");
    if (
      outputMode !== "mono" &&
      a.dual_translate_first
    ) {
      out.push("--dual-translate-first");
    }
  }

  switch (a.ocr_mode ?? "auto") {
    case "auto":
      out.push("--auto-enable-ocr-workaround");
      break;
    case "off":
      out.push("--skip-scanned-detection");
      break;
    case "force":
      out.push("--ocr-workaround");
      break;
  }

  out.push("--watermark-output-mode", "no_watermark", "--report-interval", "0.1");

  if (outputMode === "mono") out.push("--no-dual");
  else if (outputMode === "dual") out.push("--no-mono");

  if (outputMode !== "mono" && a.use_alternating_pages_dual) {
    out.push("--use-alternating-pages-dual");
  }
  if (a.pages) out.push("--pages", a.pages);
  if (a.min_text_length != null) out.push("--min-text-length", String(a.min_text_length));
  if (a.max_pages_per_part != null) out.push("--max-pages-per-part", String(a.max_pages_per_part));
  if (a.glossary_files?.length) out.push("--glossary-files", a.glossary_files.join(","));
  if (a.no_auto_extract_glossary) out.push("--no-auto-extract-glossary");
  if (a.save_auto_extracted_glossary) out.push("--save-auto-extracted-glossary");
  if (a.primary_font_family && a.primary_font_family !== "auto") {
    out.push("--primary-font-family", a.primary_font_family);
  }
  if (a.translate_table_text) out.push("--translate-table-text");
  if (a.disable_graphic_element_process) out.push("--disable-graphic-element-process");
  if (a.no_merge_alternating_line_numbers) out.push("--no-merge-alternating-line-numbers");
  if (a.disable_same_text_fallback) out.push("--disable-same-text-fallback");
  if (a.ignore_cache) out.push("--ignore-cache");
  if (a.pool_max_workers != null) out.push("--pool-max-workers", String(a.pool_max_workers));
  if (a.term_pool_max_workers != null) out.push("--term-pool-max-workers", String(a.term_pool_max_workers));
  if (a.custom_system_prompt) out.push("--custom-system-prompt", a.custom_system_prompt);
  if (a.no_send_temperature) out.push("--no-send-temperature");
  if (a.enable_json_mode_if_requested) out.push("--enable-json-mode-if-requested");
  if (a.send_dashscope_header) out.push("--send-dashscope-header");
  if (a.openai_reasoning) out.push("--openai-reasoning", a.openai_reasoning);

  return out;
}
