use tauri::State;

use crate::domain::{ApiProfile, AppResult, StorageInfo};
use crate::storage::{load_api_profile, save_api_profile as persist_api_profile, AppState};

#[tauri::command]
pub fn get_api_profile(state: State<AppState>) -> AppResult<Option<ApiProfile>> {
    load_api_profile(&state).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn save_api_profile(state: State<AppState>, profile: ApiProfile) -> AppResult<ApiProfile> {
    persist_api_profile(&state, profile).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_storage_info(state: State<AppState>) -> AppResult<StorageInfo> {
    Ok(state.storage_info())
}
