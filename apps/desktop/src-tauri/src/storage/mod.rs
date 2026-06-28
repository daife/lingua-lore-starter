use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::domain::{ApiProfile, CreateWorldRequest, StorageInfo, WorldRecord};
use crate::security;

const APP_MIGRATION: &str = include_str!("../../migrations/app/001_init.sql");
const WORLD_MIGRATION: &str = include_str!("../../migrations/world/001_init.sql");
const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";
const DEEPSEEK_BETA_BASE_URL: &str = "https://api.deepseek.com/beta";
const WORLD_EXPORT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct WorldExportManifest {
    version: u32,
    title: String,
    exported_at: String,
}

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

pub fn export_world_zip(state: &AppState, world_id: &str) -> Result<Vec<u8>> {
    let _guard = state.lock.lock().expect("storage lock poisoned");
    let world = get_world(state, world_id)?;
    let db_path = state.world_db_path(world_id);
    let db_bytes = fs::read(&db_path)
        .with_context(|| format!("failed to read world database: {}", db_path.display()))?;
    let manifest = WorldExportManifest {
        version: WORLD_EXPORT_VERSION,
        title: world.title,
        exported_at: now(),
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("manifest.json", options)?;
        zip.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())?;
        zip.start_file("world.db", options)?;
        zip.write_all(&db_bytes)?;
        zip.finish()?;
    }
    Ok(cursor.into_inner())
}

pub fn import_world_zip(state: &AppState, bytes: Vec<u8>) -> Result<WorldRecord> {
    let _guard = state.lock.lock().expect("storage lock poisoned");
    let mut archive = ZipArchive::new(Cursor::new(bytes)).context("invalid world zip")?;
    let mut manifest_json = String::new();
    archive
        .by_name("manifest.json")
        .context("world zip is missing manifest.json")?
        .read_to_string(&mut manifest_json)?;
    let manifest: WorldExportManifest =
        serde_json::from_str(&manifest_json).context("invalid world manifest")?;
    if manifest.version != WORLD_EXPORT_VERSION {
        anyhow::bail!("unsupported world export version: {}", manifest.version);
    }

    let mut db_bytes = Vec::new();
    archive
        .by_name("world.db")
        .context("world zip is missing world.db")?
        .read_to_end(&mut db_bytes)?;

    let id = format!("world_{}", Uuid::new_v4().simple());
    let world_dir = state.data_dir.join("worlds").join(&id);
    fs::create_dir_all(&world_dir)?;
    let db_path = world_dir.join("world.db");
    fs::write(&db_path, db_bytes)?;

    let imported = import_world_record_from_db(state, &id, &world_dir);
    if imported.is_err() {
        let _ = fs::remove_dir_all(&world_dir);
    }
    imported
}

fn import_world_record_from_db(
    state: &AppState,
    world_id: &str,
    world_dir: &PathBuf,
) -> Result<WorldRecord> {
    let world_conn = state.open_world_conn(world_id)?;
    let imported_at = now();
    world_conn.execute(
        "UPDATE world_profile SET id = ?1, updated_at = ?2",
        params![world_id, &imported_at],
    )?;
    let profile = world_conn
        .query_row(
            "SELECT title, description, genre, target_language, language_level, narrative_style
             FROM world_profile LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .context("imported world database is missing world_profile")?;
    let (title, description, _genre, target_language, language_level, _narrative_style) = profile;

    let conn = state.app_conn()?;
    let slug = unique_slug(&conn, &slugify(&title))?;
    let storage_path = world_dir.to_string_lossy().to_string();
    conn.execute(
        "INSERT INTO worlds
         (id, slug, title, description, storage_path, target_language, language_level, created_at, updated_at, last_opened_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?8)",
        params![
            world_id,
            &slug,
            &title,
            &description,
            &storage_path,
            &target_language,
            &language_level,
            &imported_at
        ],
    )?;
    get_world(state, world_id)
}

fn unique_slug(conn: &Connection, slug: &str) -> Result<String> {
    let base = if slug.trim().is_empty() {
        "world"
    } else {
        slug
    };
    let mut candidate = base.to_string();
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
        candidate = format!("{base}-{suffix}");
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
         VALUES (?1, 'Opening Scene', 'Unspecified', 'expectant', '', 'Initialize the story', 'active', ?2)",
        params![scene_id, created_at],
    )?;
    conn.execute(
        "INSERT INTO story_state (key, value, updated_at) VALUES ('scene.current', ?1, ?2)",
        params![scene_id, created_at],
    )?;
    let characters = req.characters.clone();
    if characters
        .iter()
        .filter(|character| character.is_player_character)
        .count()
        != 1
    {
        anyhow::bail!("exactly one player character is required");
    }
    for (index, character) in characters.iter().enumerate() {
        let character_id = if character.is_player_character {
            "char_player".to_string()
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
