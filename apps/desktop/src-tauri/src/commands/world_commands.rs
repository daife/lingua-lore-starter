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

const MAX_MODEL_REQUEST_ATTEMPTS: usize = 4;
const MAX_WORLD_DRAFT_REPAIR_ATTEMPTS: usize = 4;

const WORLD_DRAFT_SCHEMA: &str = r#"{
  "title": "示例标题",
  "description": "一段可直接使用的世界前提，包含核心冲突、玩家位置、氛围和开场钩子。",
  "genre": "玄幻",
  "target_language": "简体中文",
  "language_level": "一般难度",
  "narrative_style": "使用所选目标语言的沉浸式文学叙事，节奏适合互动选择推进",
  "characters": [
    {
      "name": "示例主角",
      "role": "玩家视角角色",
      "personality": "好奇、谨慎、会被玩家选择塑造",
      "background": "玩家将在这个世界中扮演的身份与出发处境。",
      "speaking_style": "自然、直接、符合目标语言难度",
      "relationship_to_player": null,
      "is_player_character": true
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
        "简体中文"
    } else {
        request.target_language.trim()
    };
    let difficulty_label = normal_difficulty_label(target_language);
    let mut messages = vec![
        ChatMessage::system(format!(
            "You generate concise world-creation drafts for an interactive language-learning novel app.\n\
             This is not the story-turn generator and you must not use tools.\n\
             Return valid json only, without markdown or commentary.\n\
             The json object must match this schema exactly, including nested arrays and boolean fields:\n{}",
            WORLD_DRAFT_SCHEMA
        )),
        ChatMessage::user(format!(
             "Create one original world draft for this selected genre: {genre}.\n\
             Use this target language for every visible draft field except target_language itself: {target_language}.\n\
             Fill narrative_style in {target_language}, not English, unless the target language is English.\n\
             Generate exactly one player viewpoint character in characters. Do not generate guides, companions, antagonists, supporting cast, or any other character.\n\
             The player character must fill name, role, personality, background, and speaking_style in {target_language}; set relationship_to_player to null and is_player_character to true.\n\
             Keep target_language exactly as {target_language}.\n\
             Fill language_level in {target_language} with the normal-difficulty meaning of \"一般难度\". Use this exact value when appropriate: {difficulty_label}.\n\
             The generated description should be ready to use directly as the world's premise."
        )),
    ];
    let mut last_error = None;
    for attempt in 0..MAX_WORLD_DRAFT_REPAIR_ATTEMPTS {
        let request = ChatCompletionRequest {
            model: profile.model.clone(),
            messages: messages.clone(),
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
            .unwrap_or_default();
        if content.trim().is_empty() {
            last_error = Some("DeepSeek returned no draft content.".to_string());
        } else {
            match serde_json::from_str::<CreateWorldRequest>(&content) {
                Ok(draft) => match validate_world_draft(&draft) {
                    Ok(()) => return Ok(normalize_world_draft(draft, genre, target_language)),
                    Err(err) => last_error = Some(format!("Invalid world draft json: {err}")),
                },
                Err(err) => last_error = Some(format!("Invalid world draft json: {err}")),
            }
        }

        if attempt + 1 < MAX_WORLD_DRAFT_REPAIR_ATTEMPTS {
            messages.push(ChatMessage::user(format!(
                "The previous draft was invalid: {}. Return a complete valid json object only, matching the exact schema and target_language.",
                last_error
                    .as_deref()
                    .unwrap_or("unknown validation failure")
            )));
        }
    }

    Err(format!(
        "DeepSeek could not produce a valid world draft after retries. Last error: {}",
        last_error.unwrap_or_else(|| "unknown validation failure".to_string())
    ))
}

async fn chat_completion_with_retries(
    client: &DeepSeekClient,
    request: ChatCompletionRequest,
) -> AppResult<crate::deepseek::ChatCompletionResponse> {
    let mut last_error = None;
    for attempt in 0..MAX_MODEL_REQUEST_ATTEMPTS {
        match client.chat_completion(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(err) => {
                last_error = Some(err.to_string());
                if attempt + 1 < MAX_MODEL_REQUEST_ATTEMPTS {
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
    draft.language_level = normal_difficulty_label(target_language);
    if draft.narrative_style.trim().is_empty() {
        draft.narrative_style = normal_narrative_style(target_language);
    }
    if let Some(character) = draft.characters.first_mut() {
        character.relationship_to_player = None;
        character.is_player_character = true;
    }
    draft.characters.truncate(1);
    draft
}

fn validate_world_draft(draft: &CreateWorldRequest) -> AppResult<()> {
    if draft.characters.len() != 1 {
        return Err("world draft must include exactly one player character".to_string());
    }
    let character = &draft.characters[0];
    if !character.is_player_character {
        return Err("world draft character must be the player character".to_string());
    }
    if character.name.trim().is_empty()
        || character.role.trim().is_empty()
        || character.personality.trim().is_empty()
        || character.background.trim().is_empty()
        || character.speaking_style.trim().is_empty()
    {
        return Err("world draft player character fields are required".to_string());
    }
    Ok(())
}

fn normal_difficulty_label(target_language: &str) -> String {
    let normalized = target_language.trim().to_lowercase();
    if normalized.contains("english") {
        return "Normal difficulty".to_string();
    }
    if normalized.contains("日本") {
        return "一般的な難易度".to_string();
    }
    if normalized.contains("한국") {
        return "보통 난이도".to_string();
    }
    "一般难度".to_string()
}

fn normal_narrative_style(target_language: &str) -> String {
    let normalized = target_language.trim().to_lowercase();
    if normalized.contains("english") {
        return "Immersive literary prose with clear interactive pacing".to_string();
    }
    if normalized.contains("日本") {
        return "没入感のある文学的な文体で、選択によって進む明快なテンポ".to_string();
    }
    if normalized.contains("한국") {
        return "선택으로 전개되는 명확한 흐름의 몰입형 문학 서술".to_string();
    }
    "沉浸式文学叙事，节奏清晰，适合互动选择推进".to_string()
}
