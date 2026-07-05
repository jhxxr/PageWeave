use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};

const KEY_PREFIX: &str = "secret:api-key:";

fn storage_key(id: &str) -> String {
    format!("{KEY_PREFIX}{id}")
}

pub fn set_secret(conn: &Connection, id: &str, value: &str) -> AppResult<()> {
    let key = storage_key(id);
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|e| AppError::Secret(e.to_string()))?;
    Ok(())
}

pub fn get_secret(conn: &Connection, id: &str) -> AppResult<Option<String>> {
    let key = storage_key(id);
    let row = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            params![key],
            |r| r.get::<_, String>(0),
        )
        .ok();
    match row {
        Some(value) => Ok(Some(value)),
        None => Ok(None),
    }
}

pub fn delete_secret(conn: &Connection, id: &str) -> AppResult<()> {
    let key = storage_key(id);
    conn.execute("DELETE FROM app_settings WHERE key = ?1", params![key])
        .map_err(|e| AppError::Secret(e.to_string()))?;
    Ok(())
}
