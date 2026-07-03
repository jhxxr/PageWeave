use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};

use crate::error::{AppError, AppResult};
use crate::translate::model::{BabeldocInfo, TranslateEvent, TranslateRequest};
use crate::translate::runner;
use crate::translate::state::TaskRegistry;

/// Start a translation. Returns the task_id immediately; progress flows over the
/// `translate://progress` event. This command never blocks on the translation itself.
#[tauri::command]
pub async fn start_translate(
    app: AppHandle,
    req: TranslateRequest,
) -> AppResult<String> {
    if req.pdf_paths.is_empty() {
        return Err(AppError::InvalidInput("至少需要一个 PDF 文件".into()));
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
pub async fn get_babeldoc_info() -> AppResult<BabeldocInfo> {
    Ok(runner::probe_babeldoc().await)
}

/// Helper used by `lib.rs` setup to create the registry state.
pub fn new_registry() -> Arc<TaskRegistry> {
    TaskRegistry::new()
}
