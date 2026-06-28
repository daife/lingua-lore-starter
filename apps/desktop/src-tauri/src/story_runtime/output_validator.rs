use anyhow::{bail, Result};
use std::collections::HashSet;

use crate::domain::TurnOutput;

pub fn validate_turn_output(output: &TurnOutput) -> Result<()> {
    if output.narration.trim().is_empty() {
        bail!("narration is required");
    }
    if output.choices.len() != 3 {
        bail!("Turn output must contain exactly 3 choices");
    }
    let labels: Vec<&str> = output.choices.iter().map(|c| c.label.as_str()).collect();
    if labels != ["A", "B", "C"] {
        bail!("Choices must be labeled A, B, C");
    }
    for choice in &output.choices {
        if !matches!(choice.risk.as_str(), "low" | "medium" | "high") {
            bail!("Invalid risk level");
        }
    }
    for update in &output.state_updates {
        if !validate_state_key(&update.key) {
            bail!("state update key is not allowed: {}", update.key);
        }
    }
    if output.new_characters.len() > 3 {
        bail!("new_characters may contain at most 3 characters");
    }
    for character in &output.new_characters {
        if character.name.trim().is_empty()
            || character.role.trim().is_empty()
            || character.personality.trim().is_empty()
            || character.background.trim().is_empty()
            || character.speaking_style.trim().is_empty()
        {
            bail!("new character fields are required");
        }
    }
    for memory in &output.memory_candidates {
        if memory.importance < 1 || memory.importance > 10 {
            bail!("Memory importance must be 1-10");
        }
        if memory.content.trim().is_empty() {
            bail!("Memory content is required");
        }
    }
    let mut relationship_keys = HashSet::new();
    for update in &output.relationship_updates {
        if update.delta < -2 || update.delta > 2 {
            bail!("Relationship delta must be between -2 and 2");
        }
        if !relationship_keys.insert((&update.character_id, &update.dimension)) {
            bail!("duplicate relationship update");
        }
    }
    Ok(())
}

fn validate_state_key(key: &str) -> bool {
    key == "scene.location"
        || key == "scene.mood"
        || key == "scene.current_objective"
        || key.starts_with("story.")
        || key.starts_with("quest.")
        || key.starts_with("flag.")
        || key.starts_with("inventory.")
        || key.starts_with("relationship_hint.")
}
