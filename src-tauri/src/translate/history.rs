use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::db::{schema, DbState};
use crate::error::{AppError, AppResult};

use super::model::{OutputMode, TranslateRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub pdf_paths: Vec<String>,
    pub output_dir: String,
    pub lang_in: String,
    pub lang_out: String,
    pub output_mode: String,
    pub provider_base_url: String,
    pub model: String,
    pub qps: u32,
    pub status: String,
    pub progress: u32,
    pub stage: String,
    pub output_files: Vec<String>,
    pub message: String,
    pub created_at: String,
    pub updated_at: String,
}

impl TaskRecord {
    pub fn from_request(id: String, req: &TranslateRequest) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            id,
            pdf_paths: req.pdf_paths.clone(),
            output_dir: req.output_dir.clone(),
            lang_in: req.lang_in.clone(),
            lang_out: req.lang_out.clone(),
            output_mode: match req.output_mode {
                OutputMode::Mono => "mono",
                OutputMode::Dual => "dual",
                OutputMode::Both => "both",
            }
            .into(),
            provider_base_url: req.provider.base_url.clone(),
            model: req.provider.model.clone(),
            qps: req.qps,
            status: "running".into(),
            progress: 0,
            stage: String::new(),
            output_files: Vec::new(),
            message: String::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[tauri::command]
pub fn list_task_records(db: State<'_, DbState>) -> AppResult<Vec<TaskRecord>> {
    let conn = db.conn.lock().unwrap();
    list_records(&conn)
}

#[tauri::command]
pub fn delete_task_record(db: State<'_, DbState>, id: String) -> AppResult<bool> {
    let conn = db.conn.lock().unwrap();
    let n = conn
        .execute(
            &format!("DELETE FROM {} WHERE id = ?1", schema::TBL_TASK_RECORD),
            params![id],
        )
        .map_err(|e| AppError::Db(format!("delete task_record: {e}")))?;
    Ok(n > 0)
}

pub fn create_record(app: &AppHandle, record: &TaskRecord) -> AppResult<()> {
    let Some(db) = app.try_state::<DbState>() else {
        return Ok(());
    };
    let conn = db.conn.lock().unwrap();
    upsert_record(&conn, record)
}

pub fn update_status(
    app: &AppHandle,
    task_id: &str,
    status: &str,
    progress: Option<u32>,
    stage: Option<String>,
    output_files: Option<Vec<String>>,
    message: Option<String>,
) -> AppResult<()> {
    let Some(db) = app.try_state::<DbState>() else {
        return Ok(());
    };
    let conn = db.conn.lock().unwrap();
    let Some(mut record) = get_record(&conn, task_id)? else {
        return Ok(());
    };
    record.status = status.into();
    if let Some(progress) = progress {
        record.progress = progress;
    }
    if let Some(stage) = stage {
        record.stage = stage;
    }
    if let Some(output_files) = output_files {
        record.output_files = output_files;
    }
    if let Some(message) = message {
        record.message = message;
    }
    record.updated_at = chrono::Local::now().to_rfc3339();
    upsert_record(&conn, &record)
}

fn upsert_record(conn: &Connection, record: &TaskRecord) -> AppResult<()> {
    let json = serde_json::to_string(record).map_err(|e| AppError::Db(e.to_string()))?;
    conn.execute(
        &format!(
            "INSERT INTO {} (id, data, created_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data",
            schema::TBL_TASK_RECORD
        ),
        params![record.id, json, record.created_at],
    )
    .map_err(|e| AppError::Db(format!("upsert task_record: {e}")))?;
    Ok(())
}

fn get_record(conn: &Connection, id: &str) -> AppResult<Option<TaskRecord>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT data FROM {} WHERE id = ?1",
            schema::TBL_TASK_RECORD
        ))
        .map_err(|e| AppError::Db(e.to_string()))?;
    let row = stmt.query_row(params![id], |r| r.get::<_, String>(0)).ok();
    match row {
        Some(s) => serde_json::from_str(&s)
            .map(Some)
            .map_err(|e| AppError::Db(format!("decode task_record: {e}"))),
        None => Ok(None),
    }
}

fn list_records(conn: &Connection) -> AppResult<Vec<TaskRecord>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT data FROM {} ORDER BY created_at DESC",
            schema::TBL_TASK_RECORD
        ))
        .map_err(|e| AppError::Db(e.to_string()))?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| AppError::Db(e.to_string()))?;
    let mut out = Vec::new();
    for row in rows {
        let json = row.map_err(|e| AppError::Db(e.to_string()))?;
        out.push(
            serde_json::from_str(&json)
                .map_err(|e| AppError::Db(format!("decode task_record: {e}")))?,
        );
    }
    Ok(out)
}
