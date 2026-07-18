use serde::{Deserialize, Serialize};

/// Frontend → backend convert request. Single local file only.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertRequest {
    /// Optional client-provided id; otherwise Rust generates one.
    #[serde(default)]
    pub task_id: Option<String>,
    /// Absolute local path to the input document.
    pub input_path: String,
    /// Absolute directory for the output `.md`.
    pub output_dir: String,
}

/// Probe result for the bundled markitdown sidecar.
#[derive(Debug, Clone, Serialize)]
pub struct MarkitdownInfo {
    pub installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub hint: String,
}

/// Event payload pushed to the frontend via `convert://progress`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ConvertEvent {
    /// One captured log line from markitdown's stdout/stderr.
    Log {
        task_id: String,
        line: String,
        stream: String,
    },
    /// Lifecycle status change. No percentage field (markitdown has no stages).
    Status {
        task_id: String,
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        output_file: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_event_types_match_frontend_contract() {
        let status = serde_json::to_value(ConvertEvent::Status {
            task_id: "c1".into(),
            status: "running".into(),
            output_file: None,
            message: None,
        })
        .unwrap();
        assert_eq!(status["type"], "status");

        let log = serde_json::to_value(ConvertEvent::Log {
            task_id: "c1".into(),
            line: "hello".into(),
            stream: "stderr".into(),
        })
        .unwrap();
        assert_eq!(log["type"], "log");
    }
}
