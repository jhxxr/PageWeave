use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};

use crate::error::{AppError, AppResult};
use crate::translate::assets;
use crate::translate::model::{BabeldocInfo, TranslateEvent, TranslateRequest};
use crate::translate::runner;
use crate::translate::state::TaskRegistry;

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
