mod commands;
mod deepseek;
mod domain;
mod official;
mod storage;
mod story_runtime;
mod tool_runtime;
mod translation;
mod turn_commit;

use storage::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let state = AppState::initialize(app.handle())
                .expect("failed to initialize Lingua Lore storage");
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::account_commands::get_official_account,
            commands::account_commands::detect_registration_phone,
            commands::account_commands::register_official_account,
            commands::account_commands::refresh_quota,
            commands::world_commands::list_worlds,
            commands::world_commands::create_world,
            commands::world_commands::delete_world,
            commands::world_commands::export_world,
            commands::world_commands::import_world,
            commands::world_commands::generate_world_draft,
            commands::world_commands::get_world_bootstrap,
            commands::story_commands::preview_story_turn,
            commands::story_commands::commit_story_turn_preview,
            commands::settings_commands::get_storage_info,
            commands::translation_commands::translate_selection,
            commands::version_commands::check_version,
            commands::version_commands::check_announcement,
            commands::version_commands::quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lingua Lore");
}
