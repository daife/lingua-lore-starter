use tauri::{State, WebviewWindow};

use crate::domain::{
    AppResult, DetectedPhone, OfficialAccount, QuotaInfo, RegisterOfficialAccountRequest,
};
use crate::official;
use crate::storage::AppState;

#[tauri::command]
pub fn get_official_account(
    state: State<AppState>,
    window: WebviewWindow,
) -> AppResult<OfficialAccount> {
    official::account(&state, &window).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn detect_registration_phone(
    state: State<AppState>,
    window: WebviewWindow,
) -> AppResult<DetectedPhone> {
    official::detect_phone(&state, &window).map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn register_official_account(
    state: State<'_, AppState>,
    window: WebviewWindow,
    request: RegisterOfficialAccountRequest,
) -> AppResult<OfficialAccount> {
    official::register(&state, &window, request)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn refresh_quota(
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> AppResult<QuotaInfo> {
    official::quota(&state, &window)
        .await
        .map_err(|err| err.to_string())
}
