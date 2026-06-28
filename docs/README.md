# Documentation

[**English**](README.md) | [**中文**](README.zh.md)

---

This directory contains the project documentation. The root README is intentionally short; implementation details live here.

## Reading Order

1. [Runtime mechanism](runtime.md): end-to-end flow for world draft generation, world initialization, story preview, tool calls, commit, memory, and quick mode.
2. [Architecture](architecture.md): frontend, Tauri commands, Rust modules, storage boundaries, and runtime responsibilities.
3. [Database layout](database.md): current SQLite databases and table responsibilities.
4. [LLM runtime](llm-runtime.md): DeepSeek-compatible request shape, JSON output, read-only tools, retries, and validation.
5. [Prompt design](prompt-design.md): what the story prompt includes and what it forbids.
6. [Development](development.md): install, local development, checks, and build commands.
7. [Release process](release.md): local release workflow and version files.

## Images

- [Runtime overview](images/runtime-overview.png)
- [Story turn runtime](images/story-turn-runtime.png)

## Document Maintenance Notes

- `deepseek-json-mode.md` and `deepseek-tool-calls.md` were merged into [LLM runtime](llm-runtime.md) because the split docs duplicated behavior and one listed a tool that does not exist in the current code.
- Database and runtime docs should be checked against `apps/desktop/src-tauri/migrations/`, `story_runtime/`, `tool_runtime/`, and `turn_commit/` before updating release-facing claims.
