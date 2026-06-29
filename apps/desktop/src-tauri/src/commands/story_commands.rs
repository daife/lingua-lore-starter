use tauri::{State, WebviewWindow};

use crate::domain::{AppResult, StoryTurnInput, StoryTurnPreview, StoryTurnResult};
use crate::storage::{official_api_profile, AppState};
use crate::story_runtime;

#[tauri::command]
pub async fn preview_story_turn(
    state: State<'_, AppState>,
    window: WebviewWindow,
    input: StoryTurnInput,
) -> AppResult<StoryTurnPreview> {
    let android_id = crate::official::android_id(&state, &window).map_err(|err| err.to_string())?;
    let profile = official_api_profile(&state, android_id).map_err(|err| err.to_string())?;
    story_runtime::preview_story_turn(&state, profile, input)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn commit_story_turn_preview(
    state: State<'_, AppState>,
    preview: StoryTurnPreview,
) -> AppResult<StoryTurnResult> {
    story_runtime::commit_story_turn_preview(&state, preview).map_err(|err| err.to_string())
}
