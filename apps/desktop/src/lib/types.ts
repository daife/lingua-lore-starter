export type RiskLevel = "low" | "medium" | "high";

export interface WorldRecord {
  id: string;
  slug: string;
  title: string;
  description: string;
  storage_path: string;
  target_language: string;
  language_level: string;
  created_at: string;
  updated_at: string;
  last_opened_at?: string | null;
}

export interface CreateWorldRequest {
  title: string;
  description: string;
  genre: string;
  target_language: string;
  language_level: string;
  narrative_style: string;
  characters: CreateCharacterRequest[];
}

export interface CreateCharacterRequest {
  name: string;
  role: string;
  personality: string;
  background: string;
  speaking_style: string;
  relationship_to_player?: string | null;
  is_player_character: boolean;
}

export interface GenerateWorldDraftRequest {
  genre: string;
  target_language: string;
}

export interface ApiProfile {
  id: string;
  name: string;
  base_url: string;
  model: string;
  api_key: string;
  use_strict_tools: boolean;
}

export interface Dialogue {
  speaker: string;
  text: string;
}

export interface SceneStatus {
  location: string;
  mood: string;
  current_objective: string;
}

export interface ChoiceOutput {
  id?: string | null;
  label: "A" | "B" | "C";
  text: string;
  intent: string;
  risk: RiskLevel;
}

export interface StateUpdate {
  key: string;
  value: string;
  reason: string;
}

export interface MemoryCandidate {
  character_id: string;
  content: string;
  importance: number;
  tags: string[];
}

export interface RelationshipUpdate {
  character_id: string;
  dimension: string;
  delta: number;
  reason: string;
}

export interface NewCharacter {
  name: string;
  role: string;
  personality: string;
  background: string;
  speaking_style: string;
  relationship_to_player?: string | null;
}

export interface TurnOutput {
  narration: string;
  dialogues: Dialogue[];
  turn_summary: string;
  scene_status: SceneStatus;
  choices: ChoiceOutput[];
  state_updates: StateUpdate[];
  new_characters: NewCharacter[];
  memory_candidates: MemoryCandidate[];
  relationship_updates: RelationshipUpdate[];
}

export interface StoryTurnResult {
  turn_id: string;
  user_input?: string | null;
  output: TurnOutput;
}

export type StoryTurnInput =
  | { kind: "choice"; choice_id: string }
  | { kind: "free_text"; text: string };

export interface StoryTurnRequest {
  world_id: string;
  scene_id: string;
  input: StoryTurnInput;
}

export interface StoryTurnPreview {
  input: StoryTurnRequest;
  raw_output_json: string;
  output: TurnOutput;
}

export interface WorldBootstrap {
  world: WorldRecord;
  scene_id: string;
  turns: StoryTurnResult[];
}

export interface CheckVersionResult {
  has_update: boolean;
  latest_version: string;
}

export interface TranslationResult {
  source_text: string;
  translated_text: string;
  us_phone: string;
  uk_phone: string;
  related_words: Array<{ key: string; value: string }>;
  phrases: Array<{ key: string; value: string }>;
  example_sentences: string;
  provider: string;
}
