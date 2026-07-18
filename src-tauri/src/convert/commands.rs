use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

use crate::convert::args;
use crate::convert::model::{ConvertEvent, ConvertRequest, MarkitdownInfo};
use crate::convert::runner;
use crate::convert::state::ConvertRegistry;
use crate::error::{AppError, AppResult};

/// Start a document → Markdown conversion. Returns task_id immediately; progress
/// flows over `convert://progress`. Independent from translate concurrency.
#[tauri::command]
pub async fn start_convert(app: AppHandle, req: ConvertRequest) -> AppResult<String> {
    if req.input_path.trim().is_empty() {
        return Err(AppError::InvalidInput("请选择要转换的文件".into()));
    }
    if req.output_dir.trim().is_empty() {
        return Err(AppError::InvalidInput("请选择输出目录".into()));
    }
    if !args::is_local_path(&req.input_path) {
        return Err(AppError::InvalidInput(
            "仅支持本地文件路径，不支持远程 URI".into(),
        ));
    }
    if !args::is_allowed_extension(&req.input_path) {
        return Err(AppError::InvalidInput(format!(
            "不支持的文件类型，仅支持: {}",
            args::ALLOWED_EXTENSIONS
                .iter()
                .map(|e| format!(".{e}"))
                .collect::<Vec<_>>()
                .join(" / ")
        )));
    }
    if !std::path::Path::new(&req.input_path).is_file() {
        return Err(AppError::NotFound(format!(
            "输入文件不存在: {}",
            req.input_path
        )));
    }

    let task_id = req
        .task_id
        .clone()
        .unwrap_or_else(|| format!("convert_{}", uuid::Uuid::new_v4().simple()));

    // Reserve the single convert slot before spawning so concurrent starts cannot
    // both slip past a non-atomic busy check (R12 / AC11).
    let (cancel_tx, cancel_rx) = mpsc::unbounded_channel::<()>();
    if let Some(reg) = app.try_state::<Arc<ConvertRegistry>>() {
        if !reg.try_begin(task_id.clone(), cancel_tx).await {
            return Err(AppError::InvalidInput(
                "已有转换任务在进行中，请等待完成或先取消".into(),
            ));
        }
    }

    let app2 = app.clone();
    let task_id2 = task_id.clone();
    tokio::spawn(async move {
        runner::run_convert(app2, task_id2, req, cancel_rx).await;
    });
    Ok(task_id)
}

/// Cancel a running conversion by killing the markitdown subprocess.
#[tauri::command]
pub async fn cancel_convert(app: AppHandle, task_id: String) -> AppResult<bool> {
    let killed = if let Some(reg) = app.try_state::<Arc<ConvertRegistry>>() {
        reg.kill(&task_id).await
    } else {
        false
    };
    let _ = app.emit(
        "convert://progress",
        &ConvertEvent::Status {
            task_id: task_id.clone(),
            status: "cancelled".into(),
            output_file: None,
            message: Some("用户已取消".into()),
        },
    );
    Ok(killed)
}

#[tauri::command]
pub async fn get_markitdown_info(app: AppHandle) -> AppResult<MarkitdownInfo> {
    Ok(runner::probe_markitdown(Some(&app)).await)
}

/// Helper used by `lib.rs` setup to create the convert registry state.
pub fn new_registry() -> Arc<ConvertRegistry> {
    ConvertRegistry::new()
}
