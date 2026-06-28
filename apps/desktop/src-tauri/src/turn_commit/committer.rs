use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::domain::{StoryInputKind, StoryTurnInput, StoryTurnResult, TurnOutput};
use crate::storage::now;

pub fn commit_turn(
    conn: &Connection,
    input: &StoryTurnInput,
    mut output: TurnOutput,
    raw_output_json: &str,
) -> Result<StoryTurnResult> {
    let tx = conn.unchecked_transaction()?;
    let created_at = now();
    let turn_id = format!("turn_{}", Uuid::new_v4().simple());
    let user_message_id = format!("msg_{}", Uuid::new_v4().simple());
    let assistant_message_id = format!("msg_{}", Uuid::new_v4().simple());
    let user_content = match &input.input {
        StoryInputKind::FreeText { text } => text.clone(),
        StoryInputKind::Choice { choice_id } => {
            tx.execute(
                "UPDATE branch_choices SET selected = 1 WHERE id = ?1",
                params![choice_id],
            )?;
            tx.query_row(
                "SELECT label || ': ' || text FROM branch_choices WHERE id = ?1",
                params![choice_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| format!("Selected choice {choice_id}"))
        }
    };

    tx.execute(
        "INSERT INTO messages (id, scene_id, turn_id, role, speaker, content, message_kind, created_at)
         VALUES (?1, ?2, ?3, 'user', NULL, ?4, 'user_action', ?5)",
        params![
            &user_message_id,
            &input.scene_id,
            &turn_id,
            &user_content,
            &created_at
        ],
    )?;
    let mut assistant_content = output.narration.clone();
    for dialogue in &output.dialogues {
        assistant_content.push_str(&format!("\n\n{}: {}", dialogue.speaker, dialogue.text));
    }
    tx.execute(
        "INSERT INTO messages (id, scene_id, turn_id, role, speaker, content, message_kind, created_at)
         VALUES (?1, ?2, ?3, 'assistant', NULL, ?4, 'story_turn', ?5)",
        params![
            &assistant_message_id,
            &input.scene_id,
            &turn_id,
            &assistant_content,
            &created_at
        ],
    )?;
    tx.execute(
        "INSERT INTO turns (id, scene_id, user_message_id, assistant_message_id, summary, raw_output_json, selected_choice_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7)",
        params![
            &turn_id,
            &input.scene_id,
            &user_message_id,
            &assistant_message_id,
            &output.turn_summary,
            raw_output_json,
            &created_at
        ],
    )?;

    tx.execute(
        "UPDATE scenes SET location = ?1, mood = ?2, current_objective = ?3, summary = ?4 WHERE id = ?5",
        params![
            &output.scene_status.location,
            &output.scene_status.mood,
            &output.scene_status.current_objective,
            &output.turn_summary,
            &input.scene_id
        ],
    )?;

    for choice in &mut output.choices {
        let choice_id = format!("choice_{}", Uuid::new_v4().simple());
        choice.id = Some(choice_id.clone());
        tx.execute(
            "INSERT INTO branch_choices (id, turn_id, label, text, intent, risk, selected, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7)",
            params![
                &choice_id,
                &turn_id,
                &choice.label,
                &choice.text,
                &choice.intent,
                &choice.risk,
                &created_at
            ],
        )?;
    }

    for update in &output.state_updates {
        let old_value: Option<String> = tx
            .query_row(
                "SELECT value FROM story_state WHERE key = ?1",
                params![&update.key],
                |row| row.get(0),
            )
            .optional()?;
        tx.execute(
            "INSERT OR REPLACE INTO story_state (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![&update.key, &update.value, &created_at],
        )?;
        tx.execute(
            "INSERT INTO state_update_logs (id, turn_id, key, old_value, new_value, reason, applied, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7)",
            params![
                format!("state_log_{}", Uuid::new_v4().simple()),
                &turn_id,
                &update.key,
                &old_value,
                &update.value,
                &update.reason,
                &created_at
            ],
        )?;
    }

    for character in &output.new_characters {
        if character_name_exists(&tx, &character.name)? {
            continue;
        }
        let character_id = format!("char_{}", Uuid::new_v4().simple());
        tx.execute(
            "INSERT INTO characters
             (id, name, role, personality, background, speaking_style, relationship_to_player, is_player_character, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8)",
            params![
                &character_id,
                character.name.trim(),
                character.role.trim(),
                character.personality.trim(),
                character.background.trim(),
                character.speaking_style.trim(),
                character.relationship_to_player.as_deref(),
                &created_at
            ],
        )?;
        tx.execute(
            "INSERT OR REPLACE INTO relationship_state (character_id, dimension, value, updated_at)
             VALUES (?1, 'trust', 0, ?2)",
            params![&character_id, &created_at],
        )?;
    }

    for memory in &output.memory_candidates {
        let accepted = memory.importance >= 7 && character_exists(&tx, &memory.character_id)?;
        tx.execute(
            "INSERT INTO memory_candidates (id, turn_id, character_id, content, importance, tags, accepted, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                format!("memcand_{}", Uuid::new_v4().simple()),
                &turn_id,
                &memory.character_id,
                &memory.content,
                memory.importance,
                serde_json::to_string(&memory.tags)?,
                if accepted { 1 } else { 0 },
                &created_at
            ],
        )?;
        if accepted {
            tx.execute(
                "INSERT INTO memories (id, character_id, memory_type, content, importance, tags, source_turn_id, created_at, last_used_at)
                 VALUES (?1, ?2, 'promoted_candidate', ?3, ?4, ?5, ?6, ?7, NULL)",
                params![
                    format!("mem_{}", Uuid::new_v4().simple()),
                    &memory.character_id,
                    &memory.content,
                    memory.importance,
                    serde_json::to_string(&memory.tags)?,
                    &turn_id,
                    &created_at
                ],
            )?;
        }
    }

    for update in &output.relationship_updates {
        if !character_exists(&tx, &update.character_id)? {
            continue;
        }
        let old_value: Option<i64> = tx
            .query_row(
                "SELECT value FROM relationship_state WHERE character_id = ?1 AND dimension = ?2",
                params![&update.character_id, &update.dimension],
                |row| row.get(0),
            )
            .optional()?;
        let new_value = (old_value.unwrap_or(0) + update.delta).clamp(-100, 100);
        tx.execute(
            "INSERT OR REPLACE INTO relationship_state (character_id, dimension, value, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                &update.character_id,
                &update.dimension,
                new_value,
                &created_at
            ],
        )?;
        tx.execute(
            "INSERT INTO relationship_update_logs (id, turn_id, character_id, dimension, old_value, delta, new_value, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                format!("rel_log_{}", Uuid::new_v4().simple()),
                &turn_id,
                &update.character_id,
                &update.dimension,
                &old_value,
                update.delta,
                new_value,
                &update.reason,
                &created_at
            ],
        )?;
    }

    tx.commit()?;
    Ok(StoryTurnResult {
        turn_id,
        user_input: Some(user_content),
        output,
    })
}

fn character_exists(conn: &Connection, character_id: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM characters WHERE id = ?1",
        params![character_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn character_name_exists(conn: &Connection, name: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM characters WHERE lower(name) = lower(?1)",
        params![name.trim()],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}
