use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::{helpers, schema, DbState};
use crate::error::{AppError, AppResult};
use crate::secrets;

use super::model::{ConnectivityRequest, ConnectionTestResult, ModelFetchResult, ProviderCategory,
                   ProviderPayload, ProviderRecord};
use super::presets;

#[derive(Serialize)]
pub struct PresetView {
    pub category: ProviderCategory,
    pub label: String,
    pub base_url: String,
    pub models: Vec<String>,
}

#[tauri::command]
pub fn list_provider_presets() -> Vec<PresetView> {
    presets::presets()
        .into_iter()
        .map(|p| PresetView {
            category: p.category,
            label: p.label.to_string(),
            base_url: p.base_url.to_string(),
            models: p.models.iter().map(|s| s.to_string()).collect(),
        })
        .collect()
}

#[tauri::command]
pub fn list_providers(db: State<'_, DbState>) -> AppResult<Vec<ProviderRecord>> {
    let conn = db.conn.lock().unwrap();
    helpers::list::<ProviderRecord>(&conn, schema::TBL_PROVIDER, Some("$.sort_index"))
}

#[tauri::command]
pub fn get_provider(db: State<'_, DbState>, id: String) -> AppResult<Option<ProviderRecord>> {
    let conn = db.conn.lock().unwrap();
    helpers::get::<ProviderRecord>(&conn, schema::TBL_PROVIDER, &id)
}

#[tauri::command]
pub fn create_provider(
    db: State<'_, DbState>,
    payload: ProviderPayload,
) -> AppResult<ProviderRecord> {
    let id = format!("prov_{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Local::now().to_rfc3339();

    let api_key_id = if payload.api_key.is_empty() {
        String::new()
    } else {
        let key_id = format!("key_{}", uuid::Uuid::new_v4().simple());
        secrets::set_secret(&key_id, &payload.api_key)?;
        key_id
    };

    let rec = ProviderRecord {
        id: id.clone(),
        name: payload.name,
        category: payload.category,
        base_url: payload.base_url,
        has_api_key: !api_key_id.is_empty(),
        api_key_id,
        models: payload.models,
        default_model: payload.default_model,
        is_enabled: payload.is_enabled,
        is_applied: false,
        sort_index: next_sort_index(&db)?,
        notes: payload.notes,
        extra: payload.extra,
        created_at: now.clone(),
        updated_at: now,
    };

    let conn = db.conn.lock().unwrap();
    helpers::upsert(&conn, schema::TBL_PROVIDER, &id, &rec)?;
    Ok(rec)
}

#[tauri::command]
pub fn update_provider(
    db: State<'_, DbState>,
    id: String,
    payload: ProviderPayload,
) -> AppResult<ProviderRecord> {
    let conn = db.conn.lock().unwrap();
    let mut rec: ProviderRecord = helpers::get::<ProviderRecord>(&conn, schema::TBL_PROVIDER, &id)?
        .ok_or_else(|| AppError::NotFound(format!("provider {id}")))?;

    rec.name = payload.name;
    rec.category = payload.category;
    rec.base_url = payload.base_url;
    rec.models = payload.models;
    rec.default_model = payload.default_model;
    rec.is_enabled = payload.is_enabled;
    rec.notes = payload.notes;
    rec.extra = payload.extra;

    // Empty key = keep existing. Non-empty = write/replace.
    if !payload.api_key.is_empty() {
        if rec.api_key_id.is_empty() {
            rec.api_key_id = format!("key_{}", uuid::Uuid::new_v4().simple());
        }
        secrets::set_secret(&rec.api_key_id, &payload.api_key)?;
        rec.has_api_key = true;
    }

    rec.updated_at = chrono::Local::now().to_rfc3339();
    helpers::upsert(&conn, schema::TBL_PROVIDER, &id, &rec)?;
    Ok(rec)
}

#[tauri::command]
pub fn delete_provider(db: State<'_, DbState>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().unwrap();
    let rec: Option<ProviderRecord> =
        helpers::get::<ProviderRecord>(&conn, schema::TBL_PROVIDER, &id)?;
    if let Some(r) = &rec {
        if !r.api_key_id.is_empty() {
            let _ = secrets::delete_secret(&r.api_key_id);
        }
    }
    helpers::delete(&conn, schema::TBL_PROVIDER, &id)?;
    Ok(())
}

#[tauri::command]
pub fn set_default_provider(db: State<'_, DbState>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().unwrap();
    // Ensure the id exists.
    let _: ProviderRecord = helpers::get::<ProviderRecord>(&conn, schema::TBL_PROVIDER, &id)?
        .ok_or_else(|| AppError::NotFound(format!("provider {id}")))?;
    helpers::set_single_flag(&conn, schema::TBL_PROVIDER, "$.is_applied", &id)?;
    Ok(())
}

/// User clicked the eye icon. Returns the plaintext key for that one provider.
#[tauri::command]
pub fn reveal_api_key(api_key_id: String) -> AppResult<Option<String>> {
    if api_key_id.is_empty() {
        return Ok(None);
    }
    secrets::get_secret(&api_key_id)
}

#[tauri::command]
pub async fn test_provider_connection(
    req: ConnectivityRequest,
) -> AppResult<ConnectionTestResult> {
    super::connectivity::test_connection(&req).await
}

#[tauri::command]
pub async fn fetch_provider_models(
    req: ConnectivityRequest,
) -> AppResult<ModelFetchResult> {
    super::connectivity::fetch_models(&req).await
}

fn next_sort_index(db: &State<'_, DbState>) -> AppResult<i32> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT COALESCE(MAX(json_extract(data,'$.sort_index')), -1) FROM provider")
        .map_err(|e| AppError::Db(e.to_string()))?;
    let n: i32 = stmt
        .query_row([], |r| r.get(0))
        .map_err(|e| AppError::Db(e.to_string()))?;
    Ok(n + 1)
}

/// Shape returned by import/export flows — never carries the plaintext key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderExport {
    pub id: String,
    pub name: String,
    pub category: ProviderCategory,
    pub base_url: String,
    pub has_api_key: bool,
    pub models: Vec<String>,
    pub default_model: String,
    pub is_enabled: bool,
    pub is_applied: bool,
    pub sort_index: i32,
    pub notes: String,
}

#[derive(Serialize)]
pub struct ExportBundle {
    pub version: u32,
    pub providers: Vec<ProviderExport>,
}

#[tauri::command]
pub fn export_providers(db: State<'_, DbState>) -> AppResult<ExportBundle> {
    let conn = db.conn.lock().unwrap();
    let all: Vec<ProviderRecord> =
        helpers::list::<ProviderRecord>(&conn, schema::TBL_PROVIDER, Some("$.sort_index"))?;
    let providers = all
        .into_iter()
        .map(|r| ProviderExport {
            id: r.id,
            name: r.name,
            category: r.category,
            base_url: r.base_url,
            has_api_key: r.has_api_key,
            models: r.models,
            default_model: r.default_model,
            is_enabled: r.is_enabled,
            is_applied: r.is_applied,
            sort_index: r.sort_index,
            notes: r.notes,
        })
        .collect();
    Ok(ExportBundle {
        version: 1,
        providers,
    })
}
