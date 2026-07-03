use serde::{Deserialize, Serialize};

/// The category drives the default base_url + common models in the UI (see `presets.rs`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderCategory {
    Openai,
    Deepseek,
    Siliconflow,
    Qwen,
    Moonshot,
    Zhipu,
    Ollama,
    Custom,
}

impl ProviderCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderCategory::Openai => "openai",
            ProviderCategory::Deepseek => "deepseek",
            ProviderCategory::Siliconflow => "siliconflow",
            ProviderCategory::Qwen => "qwen",
            ProviderCategory::Moonshot => "moonshot",
            ProviderCategory::Zhipu => "zhipu",
            ProviderCategory::Ollama => "ollama",
            ProviderCategory::Custom => "custom",
        }
    }
}

/// The full provider record stored as JSON blob in the `provider` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRecord {
    pub id: String,
    pub name: String,
    pub category: ProviderCategory,
    pub base_url: String,
    /// Stable handle into the keyring store. Empty when no key has been saved yet.
    pub api_key_id: String,
    /// True when a secret is present in keyring. Kept in sync so the UI can show a
    /// "key set" badge without ever touching the plaintext.
    pub has_api_key: bool,
    #[serde(default)]
    pub models: Vec<String>,
    pub default_model: String,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub is_applied: bool,
    #[serde(default)]
    pub sort_index: i32,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub extra: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Payload for create/update. The Rust side owns id/has_api_key/timestamps.
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderPayload {
    pub name: String,
    pub category: ProviderCategory,
    pub base_url: String,
    /// Empty string = leave existing key untouched. Non-empty = write to keyring.
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub models: Vec<String>,
    pub default_model: String,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// For test_connection / fetch_models — only what's needed, no key plaintext at rest.
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectivityRequest {
    pub api_key_id: String,
    pub base_url: String,
    /// model is optional for fetch_models.
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionTestResult {
    pub ok: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelFetchResult {
    pub ok: bool,
    pub models: Vec<String>,
    pub message: String,
}
