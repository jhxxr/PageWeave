use std::{path::PathBuf, sync::Mutex};

use rusqlite::Connection;

pub mod helpers;
pub mod migrations;
pub mod schema;

/// Holds the SQLite connection + app data dir. Wrapped in Mutex so Tauri commands can share it.
pub struct DbState {
    pub conn: Mutex<Connection>,
    pub app_data_dir: PathBuf,
}

impl DbState {
    /// Open (or create) the database file under `app_data_dir` and run migrations.
    pub fn open(app_data_dir: &std::path::Path) -> crate::error::AppResult<Self> {
        std::fs::create_dir_all(app_data_dir)?;
        let db_path = app_data_dir.join("pageweave.db");
        let conn = Connection::open(&db_path).map_err(|e| {
            crate::error::AppError::Db(format!("open {db_path:?}: {e}"))
        })?;
        // WAL for better concurrency + size; failure is non-fatal.
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        migrations::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            app_data_dir: app_data_dir.to_path_buf(),
        })
    }
}
