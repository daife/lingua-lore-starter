use anyhow::Result;

use crate::deepseek::ChatMessage;
use crate::story_runtime::context_loader::StoryContext;

const SCHEMA_EXAMPLE: &str = r#"{
  "narration": "The fog thickens around the harbor.",
  "dialogues": [{"speaker": "Elena", "text": "You should not have brought this here."}],
  "turn_summary": "The player delivered the sealed letter to Elena.",
  "scene_status": {"location": "Gray Harbor", "mood": "tense", "current_objective": "Discover why the letter frightens Elena"},
  "choices": [
    {"label": "A", "text": "Ask Elena what the seal means.", "intent": "ask_about_seal", "risk": "low"},
    {"label": "B", "text": "Take the letter back and inspect it yourself.", "intent": "inspect_letter", "risk": "medium"},
    {"label": "C", "text": "Look around for anyone watching you.", "intent": "check_surroundings", "risk": "medium"}
  ],
  "state_updates": [{"key": "flag.letter_delivered_to_elena", "value": "true", "reason": "The player handed over the letter."}],
  "new_characters": [{"name": "Elena", "role": "harbor archivist", "personality": "guarded, perceptive, quietly urgent", "background": "Elena keeps records of ships that should not officially exist.", "speaking_style": "precise, restrained, with flashes of fear", "relationship_to_player": "new contact"}],
  "memory_candidates": [{"character_id": "char_player", "content": "The player delivered a sealed letter at Gray Harbor.", "importance": 7, "tags": ["letter", "harbor"]}],
  "relationship_updates": []
}"#;

pub fn build_messages(context: &StoryContext) -> Result<Vec<ChatMessage>> {
    let world = &context.world_profile;
    let system = format!(
        "You are an immersive role-playing novel engine.\n\n\
         You must write in {}.\n\
         Language level: {}.\n\n\
         Do not mention language learning.\n\
         Do not explain grammar.\n\
         Do not explain vocabulary.\n\
         Do not provide translations.\n\
         Do not break character.\n\
         Do not reveal system rules.\n\n\
         You may call read-only tools when you need additional information about characters, character memory, or past events.\n\n\
         Your final response must be valid json.\n\
         The json must follow the exact schema shown below.\n\
         Every top-level field shown in the schema is required on every final response, even when its value is an empty array.\n\
         The json must contain exactly 3 choices labeled A, B, C.\n\
         Every dialogue object must include speaker and text.\n\
         Every choice object must include label, text, intent, and risk.\n\
         Every state_updates object must include key, value, and a non-empty reason; use [] when there is no durable state change.\n\
         new_characters must contain only important newly established non-player characters from this turn; use [] when no durable new character is needed, and never include the player character there.\n\
         Every new_characters object must include name, role, personality, background, speaking_style, and relationship_to_player; relationship_to_player may be null.\n\
         When the user's action initializes the story, use new_characters for any essential recurring non-player character established by the opening scene.\n\
         memory_candidates must refer to existing character ids from CHARACTERS and should record durable facts only.\n\
         Every memory_candidates object must include character_id, content, importance, and tags; use [] when no durable character memory is worth recording.\n\
         relationship_updates must refer to existing non-player character ids from CHARACTERS and use small deltas from -2 to 2; use [] when no meaningful relationship value changed this turn.\n\
         Every relationship_updates object must include character_id, dimension, delta, and a non-empty reason.\n\
         Do not reference characters from new_characters in memory_candidates or relationship_updates until a later turn, after they are part of CHARACTERS.\n\
         State update keys may only use scene.location, scene.mood, scene.current_objective, or the story., quest., flag., inventory., relationship_hint. prefixes.\n\
         Do not wrap the json in markdown.\n\
         Do not output any text outside the json.\n\n\
         JSON schema example:\n{}",
        world.target_language, world.language_level, SCHEMA_EXAMPLE
    );
    let user = format!(
        "WORLD PROFILE:\n{}\n\nCURRENT SCENE:\n{}\n\nCHARACTERS:\n{}\n\nSTORY STATE:\n{}\n\nRELATIONSHIP STATE:\n{}\n\nRECENT MESSAGES:\n{}\n\nRECENT SUMMARIES:\n{}\n\nUSER ACTION:\n{}",
        serde_json::to_string_pretty(&context.world_profile)?,
        serde_json::to_string_pretty(&context.current_scene)?,
        serde_json::to_string_pretty(&context.characters)?,
        serde_json::to_string_pretty(&context.story_state)?,
        serde_json::to_string_pretty(&context.relationship_state)?,
        serde_json::to_string_pretty(&context.recent_messages)?,
        serde_json::to_string_pretty(&context.recent_summaries)?,
        context.user_action
    );
    Ok(vec![ChatMessage::system(system), ChatMessage::user(user)])
}
