use tauri::State;

use crate::domain::{AppResult, StoryTurnInput, StoryTurnPreview, StoryTurnResult};
use crate::storage::{load_api_profile, AppState};
use crate::story_runtime;

#[tauri::command]
pub async fn preview_story_turn(
    state: State<'_, AppState>,
    input: StoryTurnInput,
) -> AppResult<StoryTurnPreview> {
    let profile = load_api_profile(&state)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "Please configure a DeepSeek API profile first.".to_string())?;
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
