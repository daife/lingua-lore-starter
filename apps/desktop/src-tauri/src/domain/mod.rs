use serde::{Deserialize, Serialize};

pub type AppResult<T> = Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldRecord {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub storage_path: String,
    pub target_language: String,
    pub language_level: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorldRequest {
    pub title: String,
    pub description: String,
    pub genre: String,
    pub target_language: String,
    pub language_level: String,
    pub narrative_style: String,
    #[serde(default)]
    pub characters: Vec<CreateCharacterRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCharacterRequest {
    pub name: String,
    pub role: String,
    pub personality: String,
    pub background: String,
    pub speaking_style: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationship_to_player: Option<String>,
    #[serde(default)]
    pub is_player_character: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProfile {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub use_strict_tools: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub data_dir: String,
    pub app_db_path: String,
    pub worlds_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialogue {
    pub speaker: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneStatus {
    pub location: String,
    pub mood: String,
    pub current_objective: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceOutput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub label: String,
    pub text: String,
    pub intent: String,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    pub key: String,
    pub value: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCandidate {
    pub character_id: String,
    pub content: String,
    pub importance: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipUpdate {
    pub character_id: String,
    pub dimension: String,
    pub delta: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnOutput {
    pub narration: String,
    pub dialogues: Vec<Dialogue>,
    pub turn_summary: String,
    pub scene_status: SceneStatus,
    pub choices: Vec<ChoiceOutput>,
    pub state_updates: Vec<StateUpdate>,
    pub memory_candidates: Vec<MemoryCandidate>,
    pub relationship_updates: Vec<RelationshipUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StoryInputKind {
    Choice { choice_id: String },
    FreeText { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryTurnInput {
    pub world_id: String,
    pub scene_id: String,
    pub input: StoryInputKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryTurnResult {
    pub turn_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_input: Option<String>,
    pub output: TurnOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryTurnPreview {
    pub input: StoryTurnInput,
    pub raw_output_json: String,
    pub output: TurnOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub source_text: String,
    pub translated_text: String,
    pub us_phone: String,
    pub uk_phone: String,
    pub related_words: Vec<KeyValue>,
    pub phrases: Vec<KeyValue>,
    pub example_sentences: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}
