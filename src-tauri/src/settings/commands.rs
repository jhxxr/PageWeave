use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::{helpers, DbState};
use crate::error::AppResult;

const SETTINGS_KEY: &str = "app_settings";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: String, // light | dark | system
    #[serde(default = "default_language")]
    pub language: String, // zh | en
    #[serde(default)]
    pub default_output_dir: String,
    #[serde(default = "default_lang_in")]
    pub default_lang_in: String,
    #[serde(default = "default_lang_out")]
    pub default_lang_out: String,
    #[serde(default)]
    pub default_provider_id: String,
    #[serde(default = "default_log_retention")]
    pub log_retention_days: u32,
    #[serde(default)]
    pub cache_dir: String,
}

fn default_theme() -> String {
    "system".into()
}
fn default_language() -> String {
    "zh".into()
}
fn default_lang_in() -> String {
    "en".into()
}
fn default_lang_out() -> String {
    "zh".into()
}
fn default_log_retention() -> u32 {
    7
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            language: default_language(),
            default_output_dir: String::new(),
            default_lang_in: default_lang_in(),
            default_lang_out: default_lang_out(),
            default_provider_id: String::new(),
            log_retention_days: default_log_retention(),
            cache_dir: String::new(),
        }
    }
}

#[tauri::command]
pub fn get_settings(db: State<'_, DbState>) -> AppResult<AppSettings> {
    let conn = db.conn.lock().unwrap();
    Ok(helpers::settings_get::<AppSettings>(&conn, SETTINGS_KEY)?.unwrap_or_default())
}

#[tauri::command]
pub fn save_settings(db: State<'_, DbState>, settings: AppSettings) -> AppResult<AppSettings> {
    let conn = db.conn.lock().unwrap();
    helpers::settings_put::<AppSettings>(&conn, SETTINGS_KEY, &settings)?;
    Ok(settings)
}
