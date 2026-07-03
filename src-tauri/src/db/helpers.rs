use rusqlite::{params, Connection};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::{AppError, AppResult};

/// Upsert a JSON-blob row in a table with (id, data, created_at, updated_at).
pub fn upsert<T: Serialize>(
    conn: &Connection,
    table: &str,
    id: &str,
    data: &T,
) -> AppResult<()> {
    let json = serde_json::to_string(data).map_err(|e| AppError::Db(e.to_string()))?;
    let now = now_iso();
    conn.execute(
        &format!(
            "INSERT INTO {table} (id, data, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at"
        ),
        params![id, json, &now, &now],
    )
    .map_err(|e| AppError::Db(format!("upsert {table}: {e}")))?;
    Ok(())
}

/// Get one row's JSON blob by id.
pub fn get<T: DeserializeOwned>(conn: &Connection, table: &str, id: &str) -> AppResult<Option<T>> {
    let mut stmt = conn
        .prepare(&format!("SELECT data FROM {table} WHERE id = ?1"))
        .map_err(|e| AppError::Db(e.to_string()))?;
    let row = stmt
        .query_row(params![id], |r| r.get::<_, String>(0))
        .ok();
    match row {
        Some(s) => {
            let v: T =
                serde_json::from_str(&s).map_err(|e| AppError::Db(format!("decode {table}: {e}")))?;
            Ok(Some(v))
        }
        None => Ok(None),
    }
}

/// List all rows' JSON blobs, ordered by an optional json_extract path.
pub fn list<T: DeserializeOwned>(
    conn: &Connection,
    table: &str,
    order_by_path: Option<&str>,
) -> AppResult<Vec<T>> {
    let sql = match order_by_path {
        Some(p) => format!(
            "SELECT data FROM {table} ORDER BY json_extract(data, '{p}') ASC"
        ),
        None => format!("SELECT data FROM {table} ORDER BY id ASC"),
    };
    let mut stmt = conn.prepare(&sql).map_err(|e| AppError::Db(e.to_string()))?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| AppError::Db(e.to_string()))?;
    let mut out = Vec::new();
    for r in rows {
        let s = r.map_err(|e| AppError::Db(e.to_string()))?;
        let v: T =
            serde_json::from_str(&s).map_err(|e| AppError::Db(format!("decode {table}: {e}")))?;
        out.push(v);
    }
    Ok(out)
}

pub fn delete(conn: &Connection, table: &str, id: &str) -> AppResult<bool> {
    let n = conn
        .execute(&format!("DELETE FROM {table} WHERE id = ?1"), params![id])
        .map_err(|e| AppError::Db(format!("delete {table}: {e}")))?;
    Ok(n > 0)
}

/// Clear all rows that match a json_extract predicate, then set a flag on one row.
pub fn set_single_flag(
    conn: &Connection,
    table: &str,
    flag_path: &str,
    only_id: &str,
) -> AppResult<()> {
    conn.execute(
        &format!("UPDATE {table} SET data = json_set(data, '{flag_path}', json('false'))"),
        [],
    )
    .map_err(|e| AppError::Db(e.to_string()))?;
    conn.execute(
        &format!("UPDATE {table} SET data = json_set(data, '{flag_path}', json('true')) WHERE id = ?1"),
        params![only_id],
    )
    .map_err(|e| AppError::Db(e.to_string()))?;
    Ok(())
}

/// KV upsert for app_settings (key, value-JSON).
pub fn settings_put<T: Serialize>(conn: &Connection, key: &str, value: &T) -> AppResult<()> {
    let json = serde_json::to_string(value).map_err(|e| AppError::Db(e.to_string()))?;
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, json],
    )
    .map_err(|e| AppError::Db(format!("settings_put {key}: {e}")))?;
    Ok(())
}

pub fn settings_get<T: DeserializeOwned>(conn: &Connection, key: &str) -> AppResult<Option<T>> {
    let mut stmt = conn
        .prepare("SELECT value FROM app_settings WHERE key = ?1")
        .map_err(|e| AppError::Db(e.to_string()))?;
    let row = stmt
        .query_row(params![key], |r| r.get::<_, String>(0))
        .ok();
    match row {
        Some(s) => {
            let v: T = serde_json::from_str(&s)
                .map_err(|e| AppError::Db(format!("settings_get {key}: {e}")))?;
            Ok(Some(v))
        }
        None => Ok(None),
    }
}

fn now_iso() -> String {
    chrono::Local::now().to_rfc3339()
}
