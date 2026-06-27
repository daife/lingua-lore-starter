mod commands;
mod deepseek;
mod domain;
mod security;
mod storage;
mod story_runtime;
mod tool_runtime;
mod translation;
mod turn_commit;

use storage::AppState;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::initialize().expect("failed to initialize Lingua Lore storage"))
        .invoke_handler(tauri::generate_handler![
            commands::world_commands::list_worlds,
            commands::world_commands::create_world,
            commands::world_commands::delete_world,
            commands::world_commands::generate_world_draft,
            commands::world_commands::get_world_bootstrap,
            commands::story_commands::preview_story_turn,
            commands::story_commands::commit_story_turn_preview,
            commands::settings_commands::get_api_profile,
            commands::settings_commands::save_api_profile,
            commands::translation_commands::translate_selection
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lingua Lore");
}
