# 语境传说

[**English**](README.md) | [**中文**](README.zh.md)

---

语境传说是一款沉浸式外语互动阅读应用，使用 Tauri、React、Rust、SQLite 和兼容 DeepSeek 的 Chat Completions 构建。它将玩家选择和自由文本行动转化为结构化故事回合，并把世界状态保存在本地，让角色、记忆、关系和场景状态可以跨会话延续。

## 产品名称

| 语言 | 名称 |
|---|---|
| English | Lingua Lore |
| 简体中文 | 语境传说 |
| 日本語 | 言の葉ロア |

## 功能概览

- 通过手动输入或 AI 辅助草稿创建可复用故事世界。
- 支持每回合三个分支选择，也支持玩家自由文本行动。
- 每个世界都保存在独立的本地 SQLite 数据库中。
- 使用只读 LLM 工具查询角色档案、已晋升记忆和历史事件。
- 在一个事务中提交故事消息、选项、状态更新、新角色、记忆和关系变化。
- 通过有道词典公共接口提供独立划词翻译。

## 文档

请从文档索引开始：

- [中文文档索引](docs/README.zh.md)
- [Documentation index](docs/README.md)

中文主题：

- [运行机制](docs/runtime.zh.md)
- [架构](docs/architecture.zh.md)
- [数据库结构](docs/database.zh.md)
- [LLM 运行时](docs/llm-runtime.zh.md)
- [提示词设计](docs/prompt-design.zh.md)
- [开发](docs/development.zh.md)
- [发布流程](docs/release.zh.md)

English topics:

- [Runtime mechanism](docs/runtime.md)
- [Architecture](docs/architecture.md)
- [Database layout](docs/database.md)
- [LLM runtime](docs/llm-runtime.md)
- [Prompt design](docs/prompt-design.md)
- [Development](docs/development.md)
- [Release process](docs/release.md)

## 当前技术栈

- Tauri + Rust 后端
- React + Vite 前端
- SQLite 存储
- DeepSeek Chat Completions（兼容 OpenAI 请求结构）
- 有道词典公共接口（划词翻译）
