use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Mono,
    Dual,
    Both,
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
#[serde(tag = "type")]
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
