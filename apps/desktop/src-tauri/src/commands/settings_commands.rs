use tauri::State;

use crate::domain::{AppResult, StorageInfo};
use crate::storage::AppState;

#[tauri::command]
pub fn get_storage_info(state: State<AppState>) -> AppResult<StorageInfo> {
    Ok(state.storage_info())
}
