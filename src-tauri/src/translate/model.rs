use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Mono,
    Dual,
    Both,
}

/// OCR handling strategy. `Auto` is the historical default (emits
/// `--auto-enable-ocr-workaround`); `None` in `AdvancedParams` is interpreted
/// as `Auto` so that an unset request reproduces today's CLI args verbatim.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OcrMode {
    Auto,
    Off,
    Force,
}

/// Advanced BabelDOC parameters. Every field is `Option` so that "unset" can
/// be distinguished from "explicitly false/zero". Two fields have historical
/// defaults that differ from "emit nothing":
/// - `enhance_compatibility`: `None` ⇒ `true` (emit `--enhance-compatibility`)
/// - `ocr_mode`: `None` ⇒ `Auto` (emit `--auto-enable-ocr-workaround`)
/// All other fields: `None` ⇒ do not emit the flag.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct AdvancedParams {
    // Translation scope
    #[serde(default)]
    pub pages: Option<String>,
    #[serde(default)]
    pub min_text_length: Option<u32>,
    #[serde(default)]
    pub max_pages_per_part: Option<u32>,

    // Glossary
    #[serde(default)]
    pub glossary_files: Option<Vec<String>>,
    #[serde(default)]
    pub no_auto_extract_glossary: Option<bool>,
    #[serde(default)]
    pub save_auto_extracted_glossary: Option<bool>,

    // Fonts & layout
    #[serde(default)]
    pub primary_font_family: Option<String>,
    #[serde(default)]
    pub use_alternating_pages_dual: Option<bool>,
    #[serde(default)]
    pub dual_translate_first: Option<bool>,

    // OCR & compatibility
    #[serde(default)]
    pub ocr_mode: Option<OcrMode>,
    #[serde(default)]
    pub enhance_compatibility: Option<bool>,
    #[serde(default)]
    pub skip_clean: Option<bool>,
    #[serde(default)]
    pub disable_rich_text_translate: Option<bool>,
    #[serde(default)]
    pub translate_table_text: Option<bool>,
    #[serde(default)]
    pub disable_graphic_element_process: Option<bool>,
    #[serde(default)]
    pub no_merge_alternating_line_numbers: Option<bool>,
    #[serde(default)]
    pub disable_same_text_fallback: Option<bool>,

    // Cache & pools
    #[serde(default)]
    pub ignore_cache: Option<bool>,
    #[serde(default)]
    pub pool_max_workers: Option<u32>,
    #[serde(default)]
    pub term_pool_max_workers: Option<u32>,

    // OpenAI tuning
    #[serde(default)]
    pub custom_system_prompt: Option<String>,
    #[serde(default)]
    pub no_send_temperature: Option<bool>,
    #[serde(default)]
    pub enable_json_mode_if_requested: Option<bool>,
    #[serde(default)]
    pub send_dashscope_header: Option<bool>,
    #[serde(default)]
    pub openai_reasoning: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TranslateRequest {
    /// Optional client-provided id; otherwise Rust generates one.
    #[serde(default)]
    pub task_id: Option<String>,
    /// Strict MVP: exactly one PDF per request. Batch translation is a later task.
    pub pdf_paths: Vec<String>,
    pub output_dir: String,
    pub lang_in: String,
    pub lang_out: String,
    pub output_mode: OutputMode,
    pub provider: TranslateProvider,
    #[serde(default = "default_qps")]
    pub qps: u32,
    /// Advanced BabelDOC params. `None` (old frontends) or all-`None` fields
    /// reproduce the historical CLI args verbatim.
    #[serde(default)]
    pub advanced: Option<AdvancedParams>,
}

fn default_qps() -> u32 {
    4
}

#[derive(Debug, Clone, Deserialize)]
pub struct TranslateProvider {
    pub base_url: String,
    pub api_key_id: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BabeldocInfo {
    pub installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub hint: String,
}

/// Event payload pushed to the frontend via `translate://progress`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TranslateEvent {
    /// One captured log line from babeldoc's stdout/stderr.
    Log {
        task_id: String,
        line: String,
        stream: String,
    },
    /// Best-effort progress percentage + current stage label.
    Progress {
        task_id: String,
        overall: u32,
        stage: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        part_index: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        total_parts: Option<u32>,
    },
    /// Lifecycle status change.
    Status {
        task_id: String,
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        output_files: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_event_types_match_frontend_contract() {
        let status = serde_json::to_value(TranslateEvent::Status {
            task_id: "task_1".into(),
            status: "running".into(),
            output_files: None,
            message: None,
        })
        .unwrap();
        assert_eq!(status["type"], "status");

        let progress = serde_json::to_value(TranslateEvent::Progress {
            task_id: "task_1".into(),
            overall: 42,
            stage: "Translating".into(),
            part_index: None,
            total_parts: None,
        })
        .unwrap();
        assert_eq!(progress["type"], "progress");

        let log = serde_json::to_value(TranslateEvent::Log {
            task_id: "task_1".into(),
            line: "hello".into(),
            stream: "stderr".into(),
        })
        .unwrap();
        assert_eq!(log["type"], "log");
    }
}
