use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::domain::{
    ApiProfile, CreateCharacterRequest, CreateWorldRequest, StorageInfo, WorldRecord,
};
use crate::security;

const APP_MIGRATION: &str = include_str!("../../migrations/app/001_init.sql");
const WORLD_MIGRATION: &str = include_str!("../../migrations/world/001_init.sql");
const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";
const DEEPSEEK_BETA_BASE_URL: &str = "https://api.deepseek.com/beta";

pub struct AppState {
    pub data_dir: PathBuf,
    pub app_db_path: PathBuf,
    pub lock: Mutex<()>,
}

impl AppState {
    pub fn initialize(app: &AppHandle) -> Result<Self> {
        let data_dir = app.path().app_data_dir()?;
        fs::create_dir_all(data_dir.join("worlds"))?;
        fs::create_dir_all(data_dir.join("logs"))?;
        let app_db_path = data_dir.join("app.db");
        let conn = Connection::open(&app_db_path)?;
        conn.execute_batch(APP_MIGRATION)?;
        Ok(Self {
            data_dir,
            app_db_path,
            lock: Mutex::new(()),
        })
    }

    pub fn app_conn(&self) -> Result<Connection> {
        Ok(Connection::open(&self.app_db_path)?)
    }

    pub fn world_db_path(&self, world_id: &str) -> PathBuf {
        self.data_dir.join("worlds").join(world_id).join("world.db")
    }

    pub fn open_world_conn(&self, world_id: &str) -> Result<Connection> {
        let path = self.world_db_path(world_id);
        let conn = Connection::open(path)?;
        conn.execute_batch(WORLD_MIGRATION)?;
        Ok(conn)
    }

    pub fn storage_info(&self) -> StorageInfo {
        StorageInfo {
            data_dir: self.data_dir.to_string_lossy().to_string(),
            app_db_path: self.app_db_path.to_string_lossy().to_string(),
            worlds_dir: self.data_dir.join("worlds").to_string_lossy().to_string(),
        }
    }
}

pub fn now() -> String {
    Utc::now().to_rfc3339()
}

pub fn slugify(input: &str) -> String {
    let slug: String = input
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    slug.split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn list_worlds(state: &AppState) -> Result<Vec<WorldRecord>> {
    let conn = state.app_conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, slug, title, description, storage_path, target_language,
                language_level, created_at, updated_at, last_opened_at
         FROM worlds ORDER BY COALESCE(last_opened_at, created_at) DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(WorldRecord {
            id: row.get(0)?,
            slug: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            storage_path: row.get(4)?,
            target_language: row.get(5)?,
            language_level: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            last_opened_at: row.get(9)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub fn create_world(state: &AppState, req: CreateWorldRequest) -> Result<WorldRecord> {
    let _guard = state.lock.lock().expect("storage lock poisoned");
    let id = format!("world_{}", Uuid::new_v4().simple());
    let base_slug = slugify(&req.title);
    let slug = if base_slug.is_empty() {
        id.clone()
    } else {
        base_slug
    };
    let world_dir = state.data_dir.join("worlds").join(&id);
    fs::create_dir_all(&world_dir)?;
    let storage_path = world_dir.to_string_lossy().to_string();
    let created_at = now();

    let conn = state.app_conn()?;
    conn.execute(
        "INSERT INTO worlds
         (id, slug, title, description, storage_path, target_language, language_level, created_at, updated_at, last_opened_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?8)",
        params![
            &id,
            unique_slug(&conn, &slug)?,
            &req.title,
            &req.description,
            &storage_path,
            &req.target_language,
            &req.language_level,
            &created_at
        ],
    )?;

    let world_conn = state.open_world_conn(&id)?;
    seed_world(&world_conn, &id, &req, &created_at)?;
    get_world(state, &id)
}

pub fn default_characters(target_language: &str) -> Vec<CreateCharacterRequest> {
    vec![
        CreateCharacterRequest {
            name: "Player".to_string(),
            role: "player protagonist".to_string(),
            personality: "curious, adaptable, shaped by the user's choices".to_string(),
            background: format!("The user's viewpoint character in this {target_language} story."),
            speaking_style: "direct and natural".to_string(),
            relationship_to_player: None,
            is_player_character: true,
        },
        CreateCharacterRequest {
            name: "Story Guide".to_string(),
            role: "recurring guide".to_string(),
            personality: "observant, patient, and lightly mysterious".to_string(),
            background: "A recurring character who helps the player understand the world without breaking immersion.".to_string(),
            speaking_style: "clear, vivid, and suitable for the target language level".to_string(),
            relationship_to_player: Some("first trusted contact".to_string()),
            is_player_character: false,
        },
    ]
}

pub fn delete_world(state: &AppState, world_id: &str) -> Result<()> {
    let _guard = state.lock.lock().expect("storage lock poisoned");
    let conn = state.app_conn()?;
    let changed = conn.execute("DELETE FROM worlds WHERE id = ?1", params![world_id])?;
    if changed == 0 {
        anyhow::bail!("world not found: {world_id}");
    }

    let world_dir = state.data_dir.join("worlds").join(world_id);
    if world_dir.exists() {
        fs::remove_dir_all(world_dir)?;
    }
    Ok(())
}

fn unique_slug(conn: &Connection, slug: &str) -> Result<String> {
    let mut candidate = slug.to_string();
    let mut suffix = 2;
    loop {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM worlds WHERE slug = ?1",
            params![&candidate],
            |row| row.get(0),
        )?;
        if count == 0 {
            return Ok(candidate);
        }
        candidate = format!("{slug}-{suffix}");
        suffix += 1;
    }
}

pub fn get_world(state: &AppState, world_id: &str) -> Result<WorldRecord> {
    let conn = state.app_conn()?;
    conn.execute(
        "UPDATE worlds SET last_opened_at = ?1 WHERE id = ?2",
        params![now(), world_id],
    )?;
    conn.query_row(
        "SELECT id, slug, title, description, storage_path, target_language,
                language_level, created_at, updated_at, last_opened_at
         FROM worlds WHERE id = ?1",
        params![world_id],
        |row| {
            Ok(WorldRecord {
                id: row.get(0)?,
                slug: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                storage_path: row.get(4)?,
                target_language: row.get(5)?,
                language_level: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                last_opened_at: row.get(9)?,
            })
        },
    )
    .with_context(|| format!("world not found: {world_id}"))
}

fn seed_world(
    conn: &Connection,
    id: &str,
    req: &CreateWorldRequest,
    created_at: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO world_profile
         (id, title, description, genre, target_language, language_level, narrative_style, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            id,
            &req.title,
            &req.description,
            &req.genre,
            &req.target_language,
            &req.language_level,
            &req.narrative_style,
            created_at
        ],
    )?;
    let scene_id = format!("scene_{}", Uuid::new_v4().simple());
    conn.execute(
        "INSERT INTO scenes (id, title, location, mood, summary, current_objective, status, created_at)
         VALUES (?1, 'Opening Scene', 'Unspecified', 'expectant', '', 'Begin the story', 'active', ?2)",
        params![scene_id, created_at],
    )?;
    conn.execute(
        "INSERT INTO story_state (key, value, updated_at) VALUES ('scene.current', ?1, ?2)",
        params![scene_id, created_at],
    )?;
    let characters = if req.characters.is_empty() {
        default_characters(&req.target_language)
    } else {
        req.characters.clone()
    };
    let mut non_player_seen = false;
    for (index, character) in characters.iter().enumerate() {
        let character_id = if character.is_player_character {
            "char_player".to_string()
        } else if !non_player_seen {
            non_player_seen = true;
            "char_guide".to_string()
        } else {
            format!("char_{}", Uuid::new_v4().simple())
        };
        conn.execute(
            "INSERT INTO characters
             (id, name, role, personality, background, speaking_style, relationship_to_player, is_player_character, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                character_id,
                if character.name.trim().is_empty() {
                    format!("Character {}", index + 1)
                } else {
                    character.name.trim().to_string()
                },
                character.role.trim(),
                character.personality.trim(),
                character.background.trim(),
                character.speaking_style.trim(),
                character.relationship_to_player.as_deref(),
                if character.is_player_character { 1 } else { 0 },
                created_at
            ],
        )?;
        if !character.is_player_character {
            conn.execute(
                "INSERT OR REPLACE INTO relationship_state (character_id, dimension, value, updated_at)
                 VALUES (?1, 'trust', 0, ?2)",
                params![character_id, created_at],
            )?;
        }
    }
    Ok(())
}

pub fn load_api_profile(state: &AppState) -> Result<Option<ApiProfile>> {
    let conn = state.app_conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, base_url, model, encrypted_api_key, use_strict_tools
         FROM api_profiles ORDER BY created_at DESC LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let encrypted: String = row.get(4)?;
        Ok(Some(normalize_api_profile(ApiProfile {
            id: row.get(0)?,
            name: row.get(1)?,
            base_url: row.get(2)?,
            model: row.get(3)?,
            api_key: security::decrypt_secret(&encrypted),
            use_strict_tools: row.get::<_, i64>(5)? == 1,
        })))
    } else {
        Ok(None)
    }
}

pub fn save_api_profile(state: &AppState, profile: ApiProfile) -> Result<ApiProfile> {
    let conn = state.app_conn()?;
    let profile = normalize_api_profile(profile);
    let id = if profile.id.trim().is_empty() {
        format!("api_{}", Uuid::new_v4().simple())
    } else {
        profile.id.clone()
    };
    conn.execute(
        "INSERT OR REPLACE INTO api_profiles
         (id, name, base_url, model, encrypted_api_key, use_strict_tools, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            &id,
            &profile.name,
            &profile.base_url,
            &profile.model,
            security::encrypt_secret(&profile.api_key),
            1,
            now()
        ],
    )?;
    Ok(ApiProfile { id, ..profile })
}

fn normalize_api_profile(mut profile: ApiProfile) -> ApiProfile {
    profile.base_url = normalize_deepseek_base_url(&profile.base_url);
    profile.use_strict_tools = true;
    profile
}

fn normalize_deepseek_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed == DEEPSEEK_BASE_URL {
        DEEPSEEK_BETA_BASE_URL.to_string()
    } else {
        trimmed.to_string()
    }
}
