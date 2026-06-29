CREATE TABLE IF NOT EXISTS worlds (
  id TEXT PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  storage_path TEXT NOT NULL,
  target_language TEXT NOT NULL,
  language_level TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  last_opened_at TEXT
);

CREATE TABLE IF NOT EXISTS api_profiles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  base_url TEXT NOT NULL,
  model TEXT NOT NULL,
  encrypted_api_key TEXT NOT NULL,
  use_strict_tools INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS global_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS official_accounts (
  android_id TEXT PRIMARY KEY,
  phone TEXT NOT NULL,
  invite_code TEXT NOT NULL,
  user_id TEXT NOT NULL,
  pool_balance INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL
);
