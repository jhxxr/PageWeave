pub mod db;
pub mod error;
pub mod provider;
pub mod secrets;
pub mod settings;
pub mod translate;

use db::DbState;
use provider::commands as pcmd;
use settings::commands as scmd;
use tauri::Manager;
use translate::assets as acmd;
use translate::commands as tcmd;
use translate::history as hcmd;
use translate::state::TaskRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pageweave=debug,info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Open (or create) the SQLite database under the app's data dir.
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("app_data_dir should resolve");
            let db_state = DbState::open(&data_dir).expect("open database");
            app.manage(db_state);

            // Task registry for live translations (cancel support).
            app.manage(TaskRegistry::new());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // provider
            pcmd::list_provider_presets,
            pcmd::list_providers,
            pcmd::get_provider,
            pcmd::create_provider,
            pcmd::update_provider,
            pcmd::delete_provider,
            pcmd::set_default_provider,
            pcmd::reveal_api_key,
            pcmd::test_provider_connection,
            pcmd::fetch_provider_models,
            pcmd::export_providers,
            // translate
            tcmd::start_translate,
            tcmd::cancel_translate,
            tcmd::get_babeldoc_info,
            tcmd::get_file_size,
            hcmd::list_task_records,
            hcmd::delete_task_record,
            acmd::get_offline_assets_info,
            acmd::install_offline_assets_from_file,
            acmd::install_offline_assets_from_release,
            // settings
            scmd::get_settings,
            scmd::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
