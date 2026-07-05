use std::time::Instant;

use serde_json::json;

use super::model::{ConnectionTestResult, ConnectivityRequest, ModelFetchResult};
use crate::error::{AppError, AppResult};

/// POST a minimal `chat/completions` request to verify base_url + key + model together.
/// Gateways differ on whether base_url includes `/v1`, so we try `/chat/completions` then
/// `/v1/chat/completions` when base_url doesn't already end in `/v1`.
pub async fn test_connection(
    req: &ConnectivityRequest,
    stored_key: Option<String>,
) -> AppResult<ConnectionTestResult> {
    let key = resolve_probe_key(req, stored_key)?;
    let model = req
        .model
        .clone()
        .ok_or_else(|| AppError::InvalidInput("model is required for connection test".into()))?;
    let base = req.base_url.trim_end_matches('/').to_string();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()?;

    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": "ping"}],
        "max_tokens": 1,
        "stream": false
    });

    let mut urls = vec![format!("{base}/chat/completions")];
    if !base.ends_with("/v1") {
        urls.push(format!("{base}/v1/chat/completions"));
    }

    let start = Instant::now();
    let mut last_msg = String::new();
    for url in &urls {
        match client.post(url).bearer_auth(&key).json(&body).send().await {
            Ok(resp) => {
                let status = resp.status();
                let txt = resp.text().await.unwrap_or_default();
                if status.is_success() {
                    let elapsed = start.elapsed().as_millis() as u64;
                    return Ok(ConnectionTestResult {
                        ok: true,
                        message: "连接成功".into(),
                        latency_ms: Some(elapsed),
                    });
                }
                last_msg = extract_error_message(&txt).unwrap_or(txt);
            }
            Err(e) => last_msg = e.to_string(),
        }
    }
    Ok(ConnectionTestResult {
        ok: false,
        message: if last_msg.is_empty() {
            "无法连接到服务".into()
        } else {
            last_msg
        },
        latency_ms: None,
    })
}

/// Pull `data[].id` from a GET /models response. Tries base_url + /models then + /v1/models.
pub async fn fetch_models(
    req: &ConnectivityRequest,
    stored_key: Option<String>,
) -> AppResult<ModelFetchResult> {
    let key = resolve_probe_key(req, stored_key)?;
    let base = req.base_url.trim_end_matches('/').to_string();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()?;

    let mut urls = vec![format!("{base}/models")];
    if !base.ends_with("/v1") {
        urls.push(format!("{base}/v1/models"));
    }

    let mut last_msg = String::new();
    for url in &urls {
        match client.get(url).bearer_auth(&key).send().await {
            Ok(resp) => {
                let status = resp.status();
                let txt = resp.text().await.unwrap_or_default();
                if status.is_success() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                        if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
                            let models: Vec<String> = arr
                                .iter()
                                .filter_map(|m| {
                                    m.get("id").and_then(|x| x.as_str()).map(|s| s.to_string())
                                })
                                .collect();
                            if !models.is_empty() {
                                return Ok(ModelFetchResult {
                                    ok: true,
                                    models,
                                    message: String::new(),
                                });
                            }
                        }
                    }
                }
                last_msg = extract_error_message(&txt).unwrap_or(txt);
            }
            Err(e) => last_msg = e.to_string(),
        }
    }
    Ok(ModelFetchResult {
        ok: false,
        models: vec![],
        message: if last_msg.is_empty() {
            "无法获取模型列表，请手动填写".into()
        } else {
            last_msg
        },
    })
}

fn resolve_probe_key(req: &ConnectivityRequest, stored_key: Option<String>) -> AppResult<String> {
    if let Some(key) = req
        .api_key
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        return Ok(key.to_string());
    }
    if req.api_key_id.trim().is_empty() {
        return Err(AppError::InvalidInput("API key is missing".into()));
    }
    stored_key.ok_or_else(|| AppError::InvalidInput("API key is missing".into()))
}

fn extract_error_message(txt: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(txt).ok()?;
    // OpenAI-style error envelope.
    if let Some(msg) = v
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
    {
        return Some(msg.to_string());
    }
    // Some gateways put message at top-level.
    if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
        return Some(msg.to_string());
    }
    None
}
