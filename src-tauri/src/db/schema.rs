use rusqlite::Connection;

/// Table names used across the app. Single source of truth.
pub const TBL_PROVIDER: &str = "provider";
pub const TBL_APP_SETTINGS: &str = "app_settings";
pub const TBL_TASK_RECORD: &str = "task_record";

/// DDL for the three tables. JSON-blob pattern: 4 real columns + a JSON data column.
pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS provider (
  id          TEXT PRIMARY KEY,
  data        TEXT NOT NULL,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_provider_applied ON provider(json_extract(data,'$.is_applied'));
CREATE INDEX IF NOT EXISTS idx_provider_sort    ON provider(json_extract(data,'$.sort_index'));

CREATE TABLE IF NOT EXISTS app_settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_record (
  id          TEXT PRIMARY KEY,
  data        TEXT NOT NULL,
  created_at  TEXT NOT NULL
);
"#;

pub fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA_SQL)
}
