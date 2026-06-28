# Architecture

[**English**](architecture.md) | [**中文**](architecture.zh.md)

---

Lingua Lore is a Tauri application with a React/Vite frontend and a Rust backend. The backend owns persistence, model calls, translation calls, validation, and story-turn commits.

## High-Level Layers

| Layer | Main files | Responsibility |
|---|---|---|
| React UI | `apps/desktop/src/pages/` | World library, reader, settings, selection translation UI |
| Tauri bridge | `apps/desktop/src/lib/tauri.ts` | Typed frontend wrappers around Tauri commands |
| Commands | `apps/desktop/src-tauri/src/commands/` | Public command surface used by the frontend |
| Storage | `apps/desktop/src-tauri/src/storage/mod.rs` | App database, per-world database creation, import/export, API profiles |
| Story runtime | `apps/desktop/src-tauri/src/story_runtime/` | Context loading, prompt building, LLM turn preview, output parsing and validation |
| Tool runtime | `apps/desktop/src-tauri/src/tool_runtime/` | Read-only tool definitions and SQLite-backed tool execution |
| Turn commit | `apps/desktop/src-tauri/src/turn_commit/` | Single-transaction write path for accepted story previews |
| DeepSeek client | `apps/desktop/src-tauri/src/deepseek/` | OpenAI-compatible chat completion request/response types and HTTP client |
| Translation | `apps/desktop/src-tauri/src/translation/` | Independent selection translation through Youdao |

## Runtime Pattern

The story engine follows this pattern:

```text
React input
  -> preview_story_turn
  -> load world context
  -> build prompt
  -> DeepSeek JSON output with optional read-only tools
  -> parse and validate TurnOutput
  -> return preview without writes
  -> commit_story_turn_preview
  -> commit all writes in one SQLite transaction
```

The model never writes directly. It can only return final JSON or ask for one of the read-only tools. Rust validates and applies the final output.

## Frontend Flow

- `WorldLibraryPage.tsx` lists worlds, creates worlds, imports/exports worlds, and calls AI world draft generation.
- `ReaderPage.tsx` starts the opening turn, sends free-text actions, selects branch choices, prefetches choice previews in quick mode, and handles selection translation.
- `SettingsPage.tsx` stores the API profile used by model-backed features.

## Backend Boundaries

- `generate_world_draft` calls the model without tools and returns draft form data.
- `create_world` performs local storage writes and seeds a new `world.db`.
- `preview_story_turn` may call the model and read-only tools, but does not persist the new turn.
- `commit_story_turn_preview` is the only story-turn write path.
- Selection translation is outside the story loop and never mutates world state.
