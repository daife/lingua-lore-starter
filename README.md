# Lingua Lore

[**English**](README.md) | [**中文**](README.zh.md)

---

Lingua Lore Starter is the ready-to-try Android edition of Lingua Lore. It is designed for first-time users who want the official experience without preparing their own model endpoint, API key, or desktop build pipeline.

This repository only targets Android builds. The app uses the official trial service by default so new users can install it, create a world, and start reading immediately.

If you want to use your own API, custom base URL, or self-hosted model relay, please use [daife/lingua-lore](https://github.com/daife/lingua-lore) instead.

## Product Names

| Language | Name |
|---|---|
| English | Lingua Lore |
| Chinese (简体中文) | 语境传说 |
| Japanese (日本語) | 言の葉ロア |

## What It Does

- Works out of the box with the official trial service.
- Focuses on a beginner-friendly Android experience.
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

- Android-only Tauri + Rust backend
- React + Vite frontend
- SQLite storage
- Official Lingua Lore trial API relay
- Youdao public dictionary endpoint for selection translation
