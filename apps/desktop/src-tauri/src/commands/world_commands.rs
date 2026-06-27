use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::time::{sleep, Duration};

use crate::deepseek::{ChatCompletionRequest, ChatMessage, DeepSeekClient, ResponseFormat};
use crate::domain::{
    AppResult, ChoiceOutput, CreateWorldRequest, StoryTurnResult, TurnOutput, WorldRecord,
};
use crate::storage::{
    create_world as persist_world, delete_world as remove_world, export_world_zip as zip_world,
    import_world_zip as unzip_world, list_worlds as query_worlds, load_api_profile, AppState,
};

const WORLD_DRAFT_SCHEMA: &str = r#"{
  "title": "示例标题",
  "description": "A vivid one-paragraph premise with the central conflict, playable role, tone, and opening hook.",
  "genre": "玄幻 / xuanhuan fantasy",
  "target_language": "English",
  "language_level": "B1",
  "narrative_style": "immersive literary prose in the selected target language with interactive choice-driven pacing",
  "characters": [
    {
      "name": "Mira",
      "role": "guide",
      "personality": "observant, guarded, quietly kind",
      "background": "A local ally who understands the world's central conflict.",
      "speaking_style": "clear, concise, with vivid sensory details",
      "relationship_to_player": "new ally",
      "is_player_character": false
    }
  ]
}"#;

#[derive(Debug, Serialize)]
pub struct WorldBootstrap {
    pub world: WorldRecord,
    pub scene_id: String,
    pub turns: Vec<StoryTurnResult>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateWorldDraftRequest {
    pub genre: String,
    pub target_language: String,
}

#[tauri::command]
pub fn list_worlds(state: State<AppState>) -> AppResult<Vec<WorldRecord>> {
    query_worlds(&state).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn create_world(state: State<AppState>, request: CreateWorldRequest) -> AppResult<WorldRecord> {
    persist_world(&state, request).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn delete_world(state: State<AppState>, world_id: String) -> AppResult<()> {
    remove_world(&state, &world_id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn export_world(state: State<AppState>, world_id: String) -> AppResult<Vec<u8>> {
    zip_world(&state, &world_id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn import_world(state: State<AppState>, bytes: Vec<u8>) -> AppResult<WorldRecord> {
    unzip_world(&state, bytes).map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn generate_world_draft(
    state: State<'_, AppState>,
    request: GenerateWorldDraftRequest,
) -> AppResult<CreateWorldRequest> {
    let profile = load_api_profile(&state)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "Please configure a DeepSeek API profile first.".to_string())?;
    let client = DeepSeekClient::new(profile.clone());
    let genre = if request.genre.trim().is_empty() {
        "玄幻"
    } else {
        request.genre.trim()
    };
    let target_language = if request.target_language.trim().is_empty() {
        "English"
    } else {
        request.target_language.trim()
    };
    let request = ChatCompletionRequest {
            model: profile.model,
            messages: vec![
                ChatMessage::system(format!(
                    "You generate concise world-creation drafts for an interactive language-learning novel app.\n\
                     This is not the story-turn generator and you must not use tools.\n\
                     Return valid json only, without markdown or commentary.\n\
                     The json must have exactly these keys and string values:\n{}",
                    WORLD_DRAFT_SCHEMA
                )),
                ChatMessage::user(format!(
                    "Create one original world draft for this selected genre: {genre}.\n\
                     Use this target language for title, description, genre, and narrative_style: {target_language}.\n\
                     Generate 2 to 4 reusable characters. Character text should also use {target_language}, except stable ids are not needed.\n\
                     Keep target_language exactly as {target_language} and language_level as B1.\n\
                     The generated description should be ready to use directly as the world's premise."
                )),
            ],
            tools: None,
            tool_choice: None,
            response_format: Some(ResponseFormat {
                kind: "json_object".to_string(),
            }),
            temperature: 0.9,
            max_tokens: 1200,
            stream: false,
        };
    let response = chat_completion_with_retries(&client, request).await?;

    let content = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .ok_or_else(|| "DeepSeek returned no draft content.".to_string())?;
    let draft: CreateWorldRequest =
        serde_json::from_str(&content).map_err(|err| format!("Invalid world draft json: {err}"))?;
    Ok(normalize_world_draft(draft, genre, target_language))
}

async fn chat_completion_with_retries(
    client: &DeepSeekClient,
    request: ChatCompletionRequest,
) -> AppResult<crate::deepseek::ChatCompletionResponse> {
    let mut last_error = None;
    for attempt in 0..3 {
        match client.chat_completion(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(err) => {
                last_error = Some(err.to_string());
                if attempt < 2 {
                    sleep(Duration::from_millis(400 * (attempt + 1) as u64)).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| "DeepSeek request failed".to_string()))
}

#[tauri::command]
pub fn get_world_bootstrap(state: State<AppState>, world_id: String) -> AppResult<WorldBootstrap> {
    let world = crate::storage::get_world(&state, &world_id).map_err(|err| err.to_string())?;
    let conn = state
        .open_world_conn(&world_id)
        .map_err(|err| err.to_string())?;
    let scene_id = conn
        .query_row(
            "SELECT value FROM story_state WHERE key = 'scene.current'",
            [],
            |row| row.get::<_, String>(0),
        )
        .or_else(|_| {
            conn.query_row(
                "SELECT id FROM scenes ORDER BY created_at ASC LIMIT 1",
                params![],
                |row| row.get::<_, String>(0),
            )
        })
        .map_err(|err| err.to_string())?;
    let turns = load_story_turns(&conn).map_err(|err| err.to_string())?;
    Ok(WorldBootstrap {
        world,
        scene_id,
        turns,
    })
}

fn load_story_turns(conn: &rusqlite::Connection) -> anyhow::Result<Vec<StoryTurnResult>> {
    let mut stmt = conn.prepare(
        "SELECT turns.id, messages.content, turns.raw_output_json
         FROM turns
         JOIN messages ON messages.id = turns.user_message_id
         ORDER BY turns.created_at ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut turns = Vec::new();
    for row in rows {
        let (turn_id, user_input, raw_output_json) = row?;
        let mut output: TurnOutput = serde_json::from_str(&raw_output_json)?;
        output.choices = load_branch_choices(conn, &turn_id)?;
        turns.push(StoryTurnResult {
            turn_id,
            user_input: Some(user_input),
            output,
        });
    }
    Ok(turns)
}

fn load_branch_choices(
    conn: &rusqlite::Connection,
    turn_id: &str,
) -> anyhow::Result<Vec<ChoiceOutput>> {
    let mut stmt = conn.prepare(
        "SELECT id, label, text, intent, risk
         FROM branch_choices
         WHERE turn_id = ?1
         ORDER BY label ASC",
    )?;
    let rows = stmt.query_map(params![turn_id], |row| {
        Ok(ChoiceOutput {
            id: Some(row.get(0)?),
            label: row.get(1)?,
            text: row.get(2)?,
            intent: row.get(3)?,
            risk: row.get(4)?,
        })
    })?;

    let mut choices = Vec::new();
    for row in rows {
        choices.push(row?);
    }
    Ok(choices)
}

fn normalize_world_draft(
    mut draft: CreateWorldRequest,
    selected_genre: &str,
    target_language: &str,
) -> CreateWorldRequest {
    if draft.title.trim().is_empty() {
        draft.title = format!("{selected_genre} World");
    }
    if draft.description.trim().is_empty() {
        draft.description = format!("An original {selected_genre} interactive story world.");
    }
    if draft.genre.trim().is_empty() {
        draft.genre = selected_genre.to_string();
    }
    draft.target_language = target_language.to_string();
    if draft.language_level.trim().is_empty() {
        draft.language_level = "B1".to_string();
    }
    if draft.narrative_style.trim().is_empty() {
        draft.narrative_style =
            format!("immersive literary prose in {target_language} with clear B1-level pacing");
    }
    if draft.characters.is_empty() {
        draft.characters = crate::storage::default_characters(target_language);
    }
    draft
}
