# Lingua Lore

[**English**](README.md) | [**中文**](README.zh.md)

---

Lingua Lore is an immersive foreign-language interactive fiction app built with Tauri, React, Rust, SQLite, and DeepSeek-compatible chat completions. It turns player choices and free-text actions into structured story turns, then persists the world state locally so characters, memories, relationships, and scene status can continue across sessions.

## Product Names

| Language | Name |
|---|---|
| English | Lingua Lore |
| Chinese (简体中文) | 语境传说 |
| Japanese (日本語) | 言の葉ロア |

## What It Does

- Creates reusable story worlds from manual input or AI-assisted world drafts.
- Runs interactive story turns with exactly three branch choices plus free-text player actions.
- Stores every world in its own local SQLite database.
- Uses read-only LLM tools for character profiles, promoted memories, and past events.
- Commits story messages, choices, state updates, new characters, memories, and relationship changes in one transaction.
- Provides independent selection translation through Youdao's public dictionary endpoint.

## Documentation

Start with the docs index:

- [Documentation index](docs/README.md)
- [中文文档索引](docs/README.zh.md)

Core topics:

- [Runtime mechanism](docs/runtime.md)
- [Architecture](docs/architecture.md)
- [Database layout](docs/database.md)
- [LLM runtime](docs/llm-runtime.md)
- [Prompt design](docs/prompt-design.md)
- [Development](docs/development.md)
- [Release process](docs/release.md)

中文主题：

- [运行机制](docs/runtime.zh.md)
- [架构](docs/architecture.zh.md)
- [数据库结构](docs/database.zh.md)
- [LLM 运行时](docs/llm-runtime.zh.md)
- [提示词设计](docs/prompt-design.zh.md)
- [开发](docs/development.zh.md)
- [发布流程](docs/release.zh.md)

## Current Stack

- Tauri + Rust backend
- React + Vite frontend
- SQLite storage
- DeepSeek Chat Completions with an OpenAI-compatible request shape
- Youdao public dictionary endpoint for selection translation
