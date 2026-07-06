use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};

use crate::error::{AppError, AppResult};
use crate::translate::assets;
use crate::translate::model::{BabeldocInfo, TranslateEvent, TranslateRequest};
use crate::translate::runner;
use crate::translate::state::TaskRegistry;

/// `^\s*\d+(-\d*)?(\s*,\s*\d+(-\d*)?)*\s*$` — accepts babeldoc's `--pages`
/// format: `1`, `1,2`, `1-3`, `1-,-3`, `1,2-3,-5`. Compiled once.
static PAGES_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
fn pages_re() -> &'static regex::Regex {
    PAGES_RE.get_or_init(|| {
        regex::Regex::new(r"^\s*\d+(-\d*)?(\s*,\s*\d+(-\d*)?)*\s*$")
            .expect("pages regex is a valid literal")
    })
}

/// Conservative cap on `--custom-system-prompt` length. Windows `CreateProcess`
/// has a ~32k total cmdline limit; we cap the prompt well below that so long
/// paths + the rest of the argv still fit. Larger prompts should go through a
/// `--config` toml (known debt, see args.rs header).
const CUSTOM_SYSTEM_PROMPT_MAX_CHARS: usize = 8000;

/// Start a translation. Returns the task_id immediately; progress flows over the
/// `translate://progress` event. This command never blocks on the translation itself.
#[tauri::command]
pub async fn start_translate(app: AppHandle, req: TranslateRequest) -> AppResult<String> {
    if req.pdf_paths.is_empty() {
        return Err(AppError::InvalidInput("至少需要一个 PDF 文件".into()));
    }
    if req.pdf_paths.len() > 1 {
        return Err(AppError::InvalidInput(
            "当前 MVP 仅支持一次翻译一个 PDF 文件".into(),
        ));
    }
    if req.output_dir.trim().is_empty() {
        return Err(AppError::InvalidInput("请选择输出目录".into()));
    }
    if req.lang_in.trim().is_empty() || req.lang_out.trim().is_empty() {
        return Err(AppError::InvalidInput("请选择源语言和目标语言".into()));
    }
    if req.provider.base_url.trim().is_empty() {
        return Err(AppError::InvalidInput("服务商 Base URL 不能为空".into()));
    }
    if req.provider.api_key_id.trim().is_empty() {
        return Err(AppError::InvalidInput("服务商 API Key 未设置".into()));
    }
    if req.provider.model.trim().is_empty() {
        return Err(AppError::InvalidInput("请选择或输入模型".into()));
    }
    if req.qps == 0 {
        return Err(AppError::InvalidInput("QPS 必须大于 0".into()));
    }
    if let Some(ref a) = req.advanced {
        validate_advanced(a)?;
    }
    if !assets::offline_assets_info().installed {
        return Err(AppError::InvalidInput(
            "未检测到 BabelDOC 离线资源包，请先在设置页安装。".into(),
        ));
    }
    let task_id = req
        .task_id
        .clone()
        .unwrap_or_else(|| format!("task_{}", uuid::Uuid::new_v4().simple()));
    let app2 = app.clone();
    let task_id2 = task_id.clone();
    tokio::spawn(async move {
        runner::run_translate(app2, task_id2, req).await;
    });
    Ok(task_id)
}

/// Cancel a running translation by killing the babeldoc subprocess.
#[tauri::command]
pub async fn cancel_translate(app: AppHandle, task_id: String) -> AppResult<bool> {
    let killed = if let Some(reg) = app.try_state::<Arc<TaskRegistry>>() {
        reg.kill(&task_id).await
    } else {
        false
    };
    let _ = app.emit(
        "translate://progress",
        &TranslateEvent::Status {
            task_id: task_id.clone(),
            status: "cancelled".into(),
            output_files: None,
            message: Some("用户已取消".into()),
        },
    );
    Ok(killed)
}

#[tauri::command]
pub async fn get_babeldoc_info(app: AppHandle) -> AppResult<BabeldocInfo> {
    Ok(runner::probe_babeldoc(Some(&app)).await)
}

#[tauri::command]
pub fn get_file_size(path: String) -> AppResult<u64> {
    let metadata = std::fs::metadata(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::NotFound(format!("file not found: {path}"))
        } else {
            AppError::Io(format!("read file metadata for {path}: {e}"))
        }
    })?;
    if !metadata.is_file() {
        return Err(AppError::InvalidInput(format!("not a file: {path}")));
    }
    Ok(metadata.len())
}

/// Helper used by `lib.rs` setup to create the registry state.
pub fn new_registry() -> Arc<TaskRegistry> {
    TaskRegistry::new()
}

/// Validate the advanced-params block. Only fields that are `Some` are checked;
/// `None` fields keep their historical-default semantics (see `args::build_args`).
fn validate_advanced(a: &crate::translate::model::AdvancedParams) -> AppResult<()> {
    if let Some(ref p) = a.pages {
        let p = p.trim();
        if !p.is_empty() && !pages_re().is_match(p) {
            return Err(AppError::InvalidInput(
                "pages 格式无效，例如 1,2,1-3,-3".into(),
            ));
        }
    }
    for (name, val) in [
        ("min_text_length", a.min_text_length),
        ("max_pages_per_part", a.max_pages_per_part),
        ("pool_max_workers", a.pool_max_workers),
        ("term_pool_max_workers", a.term_pool_max_workers),
    ] {
        if let Some(n) = val {
            if n == 0 {
                return Err(AppError::InvalidInput(format!("{name} 必须 ≥ 1")));
            }
        }
    }
    if let Some(ref files) = a.glossary_files {
        for f in files {
            if f.contains(',') {
                return Err(AppError::InvalidInput(format!(
                    "术语表路径不能包含逗号: {f}"
                )));
            }
            if !std::path::Path::new(f).is_file() {
                return Err(AppError::NotFound(format!("术语表文件不存在: {f}")));
            }
        }
    }
    if let Some(ref s) = a.custom_system_prompt {
        if s.chars().count() > CUSTOM_SYSTEM_PROMPT_MAX_CHARS {
            return Err(AppError::InvalidInput(format!(
                "custom_system_prompt 过长（上限 {CUSTOM_SYSTEM_PROMPT_MAX_CHARS} 字符）"
            )));
        }
    }
    Ok(())
}
