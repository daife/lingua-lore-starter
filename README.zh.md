# 语境传说

[**English**](README.md) | [**中文**](README.zh.md)

---

语境传说 Starter 是语境传说的即开即用 Android 试用版。它面向第一次体验的新手用户，默认接入官方服务，不需要提前准备模型接口、API Key 或桌面端构建环境。

本仓库只构建 Android 版本。安装后即可使用官方试用额度创建世界、开始互动阅读，并体验官方推荐流程。

如果你需要使用自己的 API、自定义 base URL 或自建模型转发服务，请移步 [daife/lingua-lore](https://github.com/daife/lingua-lore)。

## 产品名称

| 语言 | 名称 |
|---|---|
| English | Lingua Lore |
| 简体中文 | 语境传说 |
| 日本語 | 言の葉ロア |

## 功能概览

- 默认接入官方试用服务，即开即用。
- 面向新手试用，优先提供清晰、低门槛的 Android 体验。
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

- Android-only Tauri + Rust 后端
- React + Vite 前端
- SQLite 存储
- 语境传说官方试用 API 转发服务
- 有道词典公共接口（划词翻译）
