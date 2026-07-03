use rusqlite::Connection;

use crate::error::{AppError, AppResult};

use super::schema::ensure_schema;

/// Run all migrations. Idempotent (IF NOT EXISTS).
pub fn migrate(conn: &Connection) -> AppResult<()> {
    ensure_schema(conn).map_err(|e| AppError::Db(format!("migrate: {e}")))
}
